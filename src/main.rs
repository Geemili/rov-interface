
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
mod control_state;

use errors::*;
use std::path::Path;
use sdl2::pixels::Color;
use control_state::{ControlState, ThrustMode, SamplerReleaseMode};

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

    match load_mappings(&mut game_controller_subsystem) {
        Ok(_) => {}
        Err(_) => pintln!("Couldn't load mappings"),
    }
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
                    control_state.forward_thrust = val as f64 / 32768.0
                }
                Event::ControllerAxisMotion { axis: Axis::LeftX, value: val, .. } => {
                    control_state.sideways_thrust = val as f64 / 32768.0
                }
                Event::ControllerAxisMotion { axis: Axis::RightX, value: val, .. } => {
                    control_state.rotational_thrust = val as f64 / 32768.0
                }
                Event::ControllerAxisMotion { axis: Axis::TriggerLeft, value: val, .. } => {
                    control_state.ascent_thrust = val as f64 / 32768.0;
                }
                Event::ControllerAxisMotion { axis: Axis::TriggerRight, value: val, .. } => {
                    control_state.descent_thrust = val as f64 / 32768.0;
                }
                Event::ControllerButtonDown { button: Button::Y, .. } => {
                    control_state.power_lights = !control_state.power_lights
                }
                Event::ControllerButtonUp { button: Button::RightShoulder, .. } => {
                    control_state.thrust_mode = ThrustMode::Normal
                }
                Event::ControllerButtonDown { button: Button::RightShoulder, .. } => {
                    control_state.thrust_mode = ThrustMode::Emergency
                }
                Event::ControllerButtonDown { button: Button::Start, .. } => {
                    control_state.power_master = !control_state.power_master
                }
                Event::ControllerButtonDown { button: Button::B, .. } => {
                    control_state.sampler_release = true
                }
                Event::ControllerButtonUp { button: Button::LeftShoulder, .. } => {
                    control_state.sampler_release_mode = SamplerReleaseMode::One
                }
                Event::ControllerButtonDown { button: Button::LeftShoulder, .. } => {
                    control_state.sampler_release_mode = SamplerReleaseMode::All
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
            control_state.sampler_release = false;
            last_write_time = now;
        }
        mock_rov.update();

        for response in rov.responses().iter() {
            pintln!([response]);
        }

        renderer.set_draw_color(Color::RGB(255, 128, 128));
        renderer.clear();

        renderer.set_draw_color(Color::RGB(255, 255, 255));
        let surface = font.render(&fomat!("Horizontal: "(control_state.forward_thrust)))
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

        let rect = (60, 450, 50, 50).into();
        if mock_rov.robot_is_on {
            renderer.fill_rect(rect).unwrap()
        } else {
            renderer.draw_rect(rect).unwrap()
        }

        let rect = (120, 450, 50, 50).into();
        if mock_rov.light_relay {
            renderer.fill_rect(rect).unwrap()
        } else {
            renderer.draw_rect(rect).unwrap()
        }

        let rect = (180, 450, 50, 50).into();
        if mock_rov.sampler_relay {
            renderer.fill_rect(rect).unwrap()
        } else {
            renderer.draw_rect(rect).unwrap()
        }

        {
            use control_state::{MOTOR_1, MOTOR_2, MOTOR_3, MOTOR_4, MOTOR_5, MOTOR_6};
            // Render mock rov
            let motor_1_start = [430.0, 260.0];
            let motor_2_start = [370.0, 260.0];
            let motor_3_start = [430.0, 340.0];
            let motor_4_start = [370.0, 340.0];
            let motor_5_start = [500.0, 300.0];
            let motor_6_start = [560.0, 300.0];

            let multiplier = 60.0 / i16::max_value() as f64;
            let motor_1_vector = vecmath::vec2_scale([-0.5, -0.5],
                                                     mock_rov.motors[MOTOR_1 as usize] as f64 *
                                                     multiplier);
            let motor_2_vector = vecmath::vec2_scale([0.5, -0.5],
                                                     mock_rov.motors[MOTOR_2 as usize] as f64 *
                                                     multiplier);
            let motor_3_vector = vecmath::vec2_scale([-0.5, 0.5],
                                                     mock_rov.motors[MOTOR_3 as usize] as f64 *
                                                     multiplier);
            let motor_4_vector = vecmath::vec2_scale([0.5, 0.5],
                                                     mock_rov.motors[MOTOR_4 as usize] as f64 *
                                                     multiplier);
            let motor_5_vector = vecmath::vec2_scale([0.0, 1.0],
                                                     mock_rov.motors[MOTOR_5 as usize] as f64 *
                                                     multiplier);
            let motor_6_vector = vecmath::vec2_scale([0.0, 1.0],
                                                     mock_rov.motors[MOTOR_6 as usize] as f64 *
                                                     multiplier);
            let motor_1_end = vecmath::vec2_add(motor_1_start, motor_1_vector);
            let motor_2_end = vecmath::vec2_add(motor_2_start, motor_2_vector);
            let motor_3_end = vecmath::vec2_add(motor_3_start, motor_3_vector);
            let motor_4_end = vecmath::vec2_add(motor_4_start, motor_4_vector);
            let motor_5_end = vecmath::vec2_add(motor_5_start, motor_5_vector);
            let motor_6_end = vecmath::vec2_add(motor_6_start, motor_6_vector);

            renderer.draw_line((motor_1_start[0] as i32, motor_1_start[1] as i32).into(),
                           (motor_1_end[0] as i32, motor_1_end[1] as i32).into())
                .unwrap();
            renderer.draw_line((motor_2_start[0] as i32, motor_2_start[1] as i32).into(),
                           (motor_2_end[0] as i32, motor_2_end[1] as i32).into())
                .unwrap();
            renderer.draw_line((motor_3_start[0] as i32, motor_3_start[1] as i32).into(),
                           (motor_3_end[0] as i32, motor_3_end[1] as i32).into())
                .unwrap();
            renderer.draw_line((motor_4_start[0] as i32, motor_4_start[1] as i32).into(),
                           (motor_4_end[0] as i32, motor_4_end[1] as i32).into())
                .unwrap();
            renderer.draw_line((motor_5_start[0] as i32, motor_5_start[1] as i32).into(),
                           (motor_5_end[0] as i32, motor_5_end[1] as i32).into())
                .unwrap();
            renderer.draw_line((motor_6_start[0] as i32, motor_6_start[1] as i32).into(),
                           (motor_6_end[0] as i32, motor_6_end[1] as i32).into())
                .unwrap();
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
