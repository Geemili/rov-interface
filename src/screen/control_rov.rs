
use rov::Rov;
use mock::MockRov;
use screen::{Engine, Screen, Trans};
use time::{PreciseTime, Duration};
use sdl2::pixels::Color;
use util::{draw_text, draw_text_ext};
use ::control::Control;

pub struct RovControl {
    controls: Vec<Box<Control>>,
    last_write_time: PreciseTime,
    rov: Rov,
    mock_rov: MockRov,
    renderables: Vec<Box<Renderable>>,
}

impl RovControl {
    pub fn new(rov: Rov) -> RovControl {
        use gilrs;
        RovControl {
            controls: vec![
            Box::new(::control::motor::MotorBuilder::new()
                    .id(0)
                    .position([-1.0,1.0,0.0])
                    .direction([-1.0,0.0,0.0])
                    .build()),
            Box::new(::control::motor::MotorBuilder::new()
                    .id(1)
                    .position([-1.0,-1.0,0.0])
                    .direction([-1.0,0.0,0.0])
                    .build()),
            Box::new(::control::motor::MotorBuilder::new()
                    .id(2)
                    .position([0.0,-1.0,1.0])
                    .direction([0.0,0.0,-1.0])
                    .build()),
            Box::new(::control::motor::MotorBuilder::new()
                    .id(3)
                    .position([0.0,1.0,1.0])
                    .direction([0.0,0.0,-1.0])
                    .build()),
            Box::new(::control::lights::Lights::new(gilrs::Button::North)),
            Box::new(::control::master::MasterPower::new(gilrs::Button::Start)),
            Box::new(::control::servo::Servo::new(0, gilrs::Button::DPadDown, gilrs::Button::DPadUp)),
            Box::new(::control::servo::Servo::new(1, gilrs::Button::DPadRight, gilrs::Button::DPadLeft)),
            ],
            last_write_time: PreciseTime::now(),
            rov: rov,
            mock_rov: MockRov::new(),
            renderables: vec![
                Box::new(MotorRenderable::new(0, [ 30.0,  50.0], [230.0,  50.0])),
                Box::new(MotorRenderable::new(1, [ 30.0, 100.0], [230.0, 100.0])),
                Box::new(MotorRenderable::new(2, [ 75.0, 200.0], [ 75.0, 400.0])),
                Box::new(MotorRenderable::new(3, [185.0, 200.0], [185.0, 400.0])),
                Box::new(ServoRenderable::new(0, [370.0,  20.0], [370.0, 130.0])),
                Box::new(ServoRenderable::new(0, [240.0,  20.0], [240.0, 130.0])),
                Box::new(ServoRenderable::new(1, [250.0, 140.0], [360.0, 140.0])),
                Box::new(ServoRenderable::new(1, [250.0,  10.0], [360.0,  10.0])),
                Box::new(DualServoRenderable::new([1, 0], [250.0,  20.0], [360.0, 130.0])),
            ],
        }
    }
}

impl Screen for RovControl {
    fn update(&mut self, engine: &mut Engine) -> Trans {
        for (_, _controller_event) in engine.controllers.poll_events() {
        }

        for event in engine.event_pump.poll_iter() {
            use sdl2::event::Event;
            use sdl2::keyboard::Keycode;

            match event {
                Event::Quit { .. } |
                Event::KeyUp { keycode: Some(Keycode::Escape), .. } => return Trans::Quit,
                _ => (),
            }
        }

        let now = PreciseTime::now();
        if self.last_write_time.to(now) >= Duration::milliseconds(5) {
            if let Some((_id, gamepad)) = engine.controllers.gamepads().next() {
                let gamepad_state = gamepad.state();
                let mut commands = vec![];
                for control in self.controls.iter_mut() {
                    control.update(&gamepad_state);
                    control.write_commands(&mut commands);
                }

                for command in commands.iter() {
                    self.rov.send_command(command.clone()).expect("Failed to update rov");
                }

                self.last_write_time = now;
            }
        }

        let responses = self.rov.responses();
        self.mock_rov.apply_responses(&responses);
        pint!("\r");
        for r in responses {
            use rov::RovResponse::{Motor,LightsOn,LightsOff};
            let letter = match r {
                Motor {..} => 'm',
                LightsOn => 'L',
                LightsOff => 'l',
                _ => '*',
            };
            print!("{}", letter);
        }
        print!("      ");
        use std::io::{stdout,Write};
        let _ = stdout().flush();

        engine.renderer.set_draw_color(Color::RGB(255, 128, 128));
        engine.renderer.clear();

        engine.renderer.set_draw_color(Color::RGB(255, 255, 255));

        let rect = (30, 450, 70, 70).into();
        if self.mock_rov.robot_is_on {
            engine.renderer.fill_rect(rect).unwrap()
        } else {
            engine.renderer.draw_rect(rect).unwrap()
        }
        draw_text(&mut engine.renderer, &engine.font, "Master", [30, 510]);

        let rect = (120, 450, 50, 50).into();
        if self.mock_rov.light_relay {
            engine.renderer.fill_rect(rect).unwrap()
        } else {
            engine.renderer.draw_rect(rect).unwrap()
        }
        draw_text_ext(&mut engine.renderer,
                      &engine.font,
                      "Lights",
                      (120, 500, 50, 30).into());

        let rect = (180, 450, 50, 50).into();
        if self.mock_rov.sampler_relay {
            engine.renderer.fill_rect(rect).unwrap()
        } else {
            engine.renderer.draw_rect(rect).unwrap()
        }
        draw_text_ext(&mut engine.renderer,
                      &engine.font,
                      "Sampler",
                      (180, 500, 50, 30).into());

        for renderable in self.renderables.iter() {
            renderable.render(&self.mock_rov, engine);
        }

        engine.renderer.present();

        Trans::None
    }
}

trait Renderable {
    fn render(&self, mock: &MockRov, engine: &mut Engine);
}

struct MotorRenderable {
    pub id: u8,
    pub max_pos: [f32; 2],
    pub min_pos: [f32; 2],
}

impl MotorRenderable {
    pub fn new(id: u8, min_pos: [f32; 2], max_pos: [f32; 2]) -> Self {
        MotorRenderable {
            id: id,
            max_pos: max_pos,
            min_pos: min_pos,
        }
    }
}

impl Renderable for MotorRenderable {
    fn render(&self, mock: &MockRov, engine: &mut Engine) {
        use vecmath::{vec2_add, vec2_mul, vec2_normalized, vec2_scale, vec2_sub, vec2_len};
        let motor_start = vec2_add(self.max_pos, self.min_pos);
        let motor_start = vec2_mul(motor_start, [0.5, 0.5]);
        let forward_vector = vec2_sub(self.max_pos, motor_start);
        let motor_direction = vec2_normalized(forward_vector);
        let motor_len = vec2_len(forward_vector);

        let value = mock.motors[self.id as usize] as f32;
        let value = value / (i16::max_value() as f32);

        let amount = motor_len * value;

        let motor_end = vec2_scale(motor_direction, amount);
        let motor_end = vec2_add(motor_start, motor_end);

        engine.renderer
            .draw_line((motor_start[0] as i32, motor_start[1] as i32).into(),
                       (motor_end[0] as i32, motor_end[1] as i32).into())
            .unwrap();
    }
}

struct ServoRenderable {
    pub id: u8,
    pub max_pos: [f32; 2],
    pub min_pos: [f32; 2],
}

impl ServoRenderable {
    pub fn new(id: u8, min_pos: [f32; 2], max_pos: [f32; 2]) -> Self {
        ServoRenderable{
            id: id,
            max_pos: max_pos,
            min_pos: min_pos,
        }
    }
}

impl Renderable for ServoRenderable {
    fn render(&self, mock: &MockRov, engine: &mut Engine) {
        use vecmath::{vec2_add, vec2_mul, vec2_normalized, vec2_scale, vec2_sub, vec2_len};
        let servo_start = vec2_add(self.max_pos, self.min_pos);
        let servo_start = vec2_mul(servo_start, [0.5, 0.5]);
        let forward_vector = vec2_sub(self.max_pos, self.min_pos);
        let servo_direction = vec2_normalized(forward_vector);
        let servo_len = vec2_len(forward_vector);

        let servo_low = ::control::servo::SERVO_LOW as f32;
        let servo_high = ::control::servo::SERVO_HIGH as f32;

        let value = mock.servos[self.id as usize] as f32;
        let value = (value - servo_low) / (servo_high - servo_low);

        let amount = servo_len * value;

        let servo_end = vec2_scale(servo_direction, amount);
        let servo_end = vec2_add(self.min_pos, servo_end);

        engine.renderer
            .draw_line((servo_start[0] as i32, servo_start[1] as i32).into(),
                       (servo_end[0] as i32, servo_end[1] as i32).into())
            .unwrap();
    }
}

struct DualServoRenderable {
    pub ids: [u8; 2],
    pub min_pos: [f32; 2],
    pub max_pos: [f32; 2],
}

impl DualServoRenderable {
    pub fn new(ids: [u8; 2], min_pos: [f32; 2], max_pos: [f32; 2]) -> Self {
        DualServoRenderable{
            ids: ids,
            max_pos: max_pos,
            min_pos: min_pos,
        }
    }
}

impl Renderable for DualServoRenderable {
    fn render(&self, mock: &MockRov, engine: &mut Engine) {
        let servo_low = ::control::servo::SERVO_LOW as f32;
        let servo_high = ::control::servo::SERVO_HIGH as f32;

        let value = mock.servos[self.ids[0] as usize] as f32;
        let x = (value - servo_low) / (servo_high - servo_low);
        let x = x * (self.max_pos[0] - self.min_pos[0]) + self.min_pos[0];

        let value = mock.servos[self.ids[1] as usize] as f32;
        let y = (value - servo_low) / (servo_high - servo_low);
        let y = y * (self.max_pos[1] - self.min_pos[1]) + self.min_pos[1];

        let rect = (x as i32 - 5, y as i32 - 5, 10, 10).into();
        engine.renderer.fill_rect(rect).unwrap();
    }
}

