
#![recursion_limit = "1024"]
#![feature(receiver_try_iter)]

extern crate serial;
extern crate sdl2;
extern crate sdl2_ttf;
#[macro_use]
extern crate fomat_macros;
#[macro_use]
extern crate error_chain;
extern crate vecmath;
extern crate time;

mod errors;
mod rov;
mod mock;

use errors::*;
use std::path::Path;
use sdl2::pixels::Color;

fn main() {
    use std;
    let port_name = if let Some(port) = std::env::args().skip(1).next() {
        pintln!("Writing to port "(port));
        String::from(port.trim())
    } else {
        panic!("Port name is required");
    };

    let mut rov = rov::Rov::new(port_name.into());

    let sdl_context = sdl2::init().unwrap();
    let mut game_controller_subsystem = sdl_context.game_controller().unwrap();
    let video = sdl_context.video().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let ttf_context = sdl2_ttf::init().unwrap();

    let window =
        video.window("ROV Interface", 800, 600).position_centered().opengl().build().unwrap();

    let mut renderer = window.renderer().accelerated().build().unwrap();

    load_mappings(&mut game_controller_subsystem).expect("Error loading mappings");
    let font = ttf_context.load_font(Path::new("assets/fonts/NotoSans/NotoSans-Regular.ttf"), 64)
        .unwrap();

    let available = match game_controller_subsystem.num_joysticks() {
        Ok(n) => n,
        Err(e) => panic!("Can't enumerate joysticks. :( {:?}", e),
    };

    pintln!((available)" game controllers available");

    let mut game_controllers = None;

    // Iterate over all available game_controllerss and stop once we manage to
    // open one.
    for id in 0..available {
        if game_controller_subsystem.is_game_controller(id) {
            match game_controller_subsystem.open(id) {
                Ok(c) => {
                    pintln!("Success: opened \""(c.name())"\".");
                    game_controllers = Some(c);
                    break;
                }
                Err(e) => pintln!("failed: "[e]),
            }
        } else {
            pintln!("Controller "(id)" has no mapping.");
        }
    }

    if game_controllers.is_none() {
        panic!("Couldn't open any joystick");
    };

    let mut control_state = ControlState::new();
    let mut prev_control_state = control_state.clone();
    let mut last_write_time = time::PreciseTime::now();
    let mut mock_rov = mock::MockRov::new();

    'main: loop {
        for event in event_pump.poll_iter() {
            use sdl2::event::Event;
            use sdl2::keyboard::Keycode;
            use sdl2::controller::Axis;
            use sdl2::controller::Button;

            match event {
                Event::ControllerAxisMotion { axis: Axis::LeftY, value: val, .. } => {
                    // Axis motion is an absolute value in the range
                    // [-32768, 32767]. Let's simulate a very rough dead
                    // zone to ignore spurious events.
                    let dead_zone = 10000;
                    control_state.horizontal_thrust = if val > dead_zone || val < -dead_zone {
                        val as f64 / 32768.0
                    } else {
                        0.0
                    }
                }
                Event::ControllerAxisMotion { axis: Axis::LeftX, value: val, .. } => {
                    let dead_zone = 10000;
                    control_state.sideways_thrust = if val > dead_zone || val < -dead_zone {
                        val as f64 / 32768.0
                    } else {
                        0.0
                    }
                }
                Event::ControllerButtonDown { button: Button::Y, .. } => {
                    control_state.power_lights = !control_state.power_lights
                }
                Event::Quit { .. } |
                Event::KeyUp { keycode: Some(Keycode::Escape), .. } => break 'main,
                _ => (),
            }
        }

        let now = time::PreciseTime::now();
        if last_write_time.to(now) >= time::Duration::milliseconds(5) {
            let mut buffer = vec![];
            control_state.generate_commands_diff(&prev_control_state, &mut buffer);
            mock_rov.apply_commands(&buffer);
            for command in buffer.iter() {
                rov.send_command(command.clone()).expect("Failed to update rov");
            }

            prev_control_state = control_state.clone();
            last_write_time = now;
        }

        for response in rov.responses().iter() {
            pintln!([response]);
        }

        renderer.set_draw_color(Color::RGB(255, 128, 128));
        renderer.clear();

        renderer.set_draw_color(Color::RGB(255, 255, 255));
        let surface = font.render(&fomat!("Horizontal: "(control_state.horizontal_thrust)))
            .solid(Color::RGB(255, 255, 255))
            .unwrap();
        let texture = renderer.create_texture_from_surface(&surface).unwrap();
        let mut dest = surface.rect();
        dest.set_y(0);
        renderer.copy(&texture, None, Some(dest)).unwrap();

        let surface = font.render(&fomat!("Sideways: "(control_state.sideways_thrust)))
            .solid(Color::RGB(255, 255, 255))
            .unwrap();
        let texture = renderer.create_texture_from_surface(&surface).unwrap();
        let mut dest = surface.rect();
        dest.set_y(64);
        renderer.copy(&texture, None, Some(dest)).unwrap();

        let surface = font.render(&fomat!("Lights"))
            .solid(Color::RGB(255, 255, 255))
            .unwrap();
        let texture = renderer.create_texture_from_surface(&surface).unwrap();
        let mut dest = surface.rect();
        dest.set_y(150);
        renderer.copy(&texture, None, Some(dest)).unwrap();
        let rect = (dest.x() + dest.width() as i32 + 20, dest.y(), dest.height(), dest.height())
            .into();
        if control_state.power_lights {
            renderer.fill_rect(rect).unwrap()
        } else {
            renderer.draw_rect(rect).unwrap()
        }

        renderer.present();
    }

    rov.quit().unwrap();
}

fn load_mappings(game_controller_subsystem: &mut sdl2::GameControllerSubsystem) -> Result<()> {
    use std::fs::OpenOptions;
    use std::io::{BufReader, BufRead};
    let file = OpenOptions::new().read(true)
        .open("assets/controller_mappings.csv")
        .chain_err(|| "Unable to load controller mappings")?;
    let reader = BufReader::new(&file);
    for line in reader.lines() {
        let l = line.chain_err(|| "Error reading line")?;
        if l == "" {
            continue;
        }
        match game_controller_subsystem.add_mapping(l.trim()) {
            Ok(_) => {}
            Err(e) => pintln!("Error parsing mapping: "[e]),
        }
    }
    Ok(())
}

#[derive(Clone)]
struct ControlState {
    pub horizontal_thrust: f64,
    pub vertical_thrust: f64,
    pub sideways_thrust: f64,
    pub release_one: bool,
    pub release_all: bool,
    pub power_lights: bool,
}

// Corresponding to the BlueROV Vectored ROV configuration
const MOTOR_1: u8 = 1;
const MOTOR_2: u8 = 2;
const MOTOR_3: u8 = 3;
const MOTOR_4: u8 = 4;
const MOTOR_5: u8 = 5;
const MOTOR_6: u8 = 6;

impl ControlState {
    pub fn new() -> ControlState {
        ControlState {
            horizontal_thrust: 0.0,
            vertical_thrust: 0.0,
            sideways_thrust: 0.0,
            release_one: false,
            release_all: false,
            power_lights: false,
        }
    }

    pub fn generate_commands_diff(&self, other: &ControlState, buffer: &mut Vec<rov::RovCommand>) {
        // Horizontal movement
        if self.horizontal_thrust != other.horizontal_thrust ||
           self.sideways_thrust != other.sideways_thrust {
            // TODO: Research doing this with ints.
            let control_vector = [self.horizontal_thrust, self.sideways_thrust];
            let motor_1_vector = [0.5, -0.5];
            let motor_2_vector = [-0.5, -0.5];
            let motor_3_vector = [0.5, 0.5];
            let motor_4_vector = [-0.5, 0.5];
            let motor_1_throttle = vecmath::vec2_dot(control_vector, motor_1_vector);
            let motor_2_throttle = vecmath::vec2_dot(control_vector, motor_2_vector);
            let motor_3_throttle = vecmath::vec2_dot(control_vector, motor_3_vector);
            let motor_4_throttle = vecmath::vec2_dot(control_vector, motor_4_vector);

            buffer.push(rov::RovCommand::ControlMotor {
                id: MOTOR_1,
                throttle: (motor_1_throttle * std::i16::MAX as f64) as i16,
            });
            buffer.push(rov::RovCommand::ControlMotor {
                id: MOTOR_2,
                throttle: (motor_2_throttle * std::i16::MAX as f64) as i16,
            });
            buffer.push(rov::RovCommand::ControlMotor {
                id: MOTOR_3,
                throttle: (motor_3_throttle * std::i16::MAX as f64) as i16,
            });
            buffer.push(rov::RovCommand::ControlMotor {
                id: MOTOR_4,
                throttle: (motor_4_throttle * std::i16::MAX as f64) as i16,
            });
        }

        // Vertical movement
        if self.vertical_thrust != other.vertical_thrust {
            buffer.push(rov::RovCommand::ControlMotor {
                id: MOTOR_5,
                throttle: (self.vertical_thrust * std::i16::MAX as f64) as i16,
            });
            buffer.push(rov::RovCommand::ControlMotor {
                id: MOTOR_6,
                throttle: (self.vertical_thrust * std::i16::MAX as f64) as i16,
            });
        }

        // Lights
        match (self.power_lights, other.power_lights) {
            (true, false) => {
                buffer.push(rov::RovCommand::LightsOn);
            }
            (false, true) => {
                buffer.push(rov::RovCommand::LightsOff);
            }
            _ => {
                // The mode didn't change; there is no need to send a command
            }
        }

        // TODO: Add in releasing sediment
    }
}
