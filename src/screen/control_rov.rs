
use rov::Rov;
use mock::MockRov;
use screen::{Engine, Screen, Trans};
use time::{PreciseTime, Duration};
use vecmath;
use sdl2::pixels::Color;
use util::{draw_text, draw_text_ext};
use ::control::Control;

pub struct RovControl {
    controls: Vec<Box<Control>>,
    last_write_time: PreciseTime,
    rov: Rov,
    mock_rov: MockRov,
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
                    .id(4)
                    .position([0.0,-1.0,1.0])
                    .direction([0.0,0.0,-1.0])
                    .build()),
            Box::new(::control::motor::MotorBuilder::new()
                    .id(5)
                    .position([0.0,1.0,1.0])
                    .direction([0.0,0.0,-1.0])
                    .build()),
            Box::new(::control::lights::Lights::new(gilrs::Button::North)),
            ],
            last_write_time: PreciseTime::now(),
            rov: rov,
            mock_rov: MockRov::new(),
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

        self.mock_rov.apply_responses(&self.rov.responses());

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
                                                     self.mock_rov.motors[MOTOR_1 as usize] as f64 *
                                                     multiplier);
            let motor_2_vector = vecmath::vec2_scale([0.5, -0.5],
                                                     self.mock_rov.motors[MOTOR_2 as usize] as f64 *
                                                     multiplier);
            let motor_3_vector = vecmath::vec2_scale([-0.5, 0.5],
                                                     self.mock_rov.motors[MOTOR_3 as usize] as f64 *
                                                     multiplier);
            let motor_4_vector = vecmath::vec2_scale([0.5, 0.5],
                                                     self.mock_rov.motors[MOTOR_4 as usize] as f64 *
                                                     multiplier);
            let motor_5_vector = vecmath::vec2_scale([0.0, 1.0],
                                                     self.mock_rov.motors[MOTOR_5 as usize] as f64 *
                                                     multiplier);
            let motor_6_vector = vecmath::vec2_scale([0.0, 1.0],
                                                     self.mock_rov.motors[MOTOR_6 as usize] as f64 *
                                                     multiplier);
            let motor_1_end = vecmath::vec2_add(motor_1_start, motor_1_vector);
            let motor_2_end = vecmath::vec2_add(motor_2_start, motor_2_vector);
            let motor_3_end = vecmath::vec2_add(motor_3_start, motor_3_vector);
            let motor_4_end = vecmath::vec2_add(motor_4_start, motor_4_vector);
            let motor_5_end = vecmath::vec2_add(motor_5_start, motor_5_vector);
            let motor_6_end = vecmath::vec2_add(motor_6_start, motor_6_vector);

            engine.renderer
                .draw_line((motor_1_start[0] as i32, motor_1_start[1] as i32).into(),
                           (motor_1_end[0] as i32, motor_1_end[1] as i32).into())
                .unwrap();
            engine.renderer
                .draw_line((motor_2_start[0] as i32, motor_2_start[1] as i32).into(),
                           (motor_2_end[0] as i32, motor_2_end[1] as i32).into())
                .unwrap();
            engine.renderer
                .draw_line((motor_3_start[0] as i32, motor_3_start[1] as i32).into(),
                           (motor_3_end[0] as i32, motor_3_end[1] as i32).into())
                .unwrap();
            engine.renderer
                .draw_line((motor_4_start[0] as i32, motor_4_start[1] as i32).into(),
                           (motor_4_end[0] as i32, motor_4_end[1] as i32).into())
                .unwrap();
            engine.renderer
                .draw_line((motor_5_start[0] as i32, motor_5_start[1] as i32).into(),
                           (motor_5_end[0] as i32, motor_5_end[1] as i32).into())
                .unwrap();
            engine.renderer
                .draw_line((motor_6_start[0] as i32, motor_6_start[1] as i32).into(),
                           (motor_6_end[0] as i32, motor_6_end[1] as i32).into())
                .unwrap();
        }

        engine.renderer.present();

        Trans::None
    }
}
