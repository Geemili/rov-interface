
const TIME_BETWEEN_POLLING_PORTS_MS: i64 = 1_000;

use time::{PreciseTime, Duration};
use serial_enumerate;
use util::draw_text;
use screen::{Engine, Screen, Trans};

pub struct PortSelect {
    ports: Vec<String>,
    selected: usize,
    last_poll_time: PreciseTime,
}

impl PortSelect {
    pub fn new() -> PortSelect {
        PortSelect {
            ports: serial_enumerate::enumerate_serial_ports().unwrap(),
            selected: 0,
            last_poll_time: PreciseTime::now(),
        }
    }
}

impl Screen for PortSelect {
    fn update(&mut self, engine: &mut Engine) -> Trans {
        for event in engine.event_pump.poll_iter() {
            use sdl2::event::Event;
            use sdl2::keyboard::Keycode;
            use sdl2::controller::Button;

            match event {
                Event::ControllerButtonUp { button: Button::DPadDown, .. } |
                Event::KeyDown { keycode: Some(Keycode::Down), .. } => {
                    if self.ports.len() > 0 && self.selected < self.ports.len() - 1 {
                        self.selected += 1;
                    }
                }
                Event::ControllerButtonDown { button: Button::DPadUp, .. } |
                Event::KeyDown { keycode: Some(Keycode::Up), .. } => {
                    if self.ports.len() > 0 && self.selected > 0 {
                        self.selected -= 1;
                    }
                }
                Event::Quit { .. } |
                Event::KeyUp { keycode: Some(Keycode::Escape), .. } => return Trans::Quit,
                _ => (),
            }
        }

        if self.last_poll_time.to(PreciseTime::now()) >=
           Duration::milliseconds(TIME_BETWEEN_POLLING_PORTS_MS) {
            self.ports = serial_enumerate::enumerate_serial_ports().unwrap();
            self.last_poll_time = PreciseTime::now();
        }

        engine.renderer.clear();

        let offset_x = 64;
        let height = 64;
        let mut y = 0;

        for port in self.ports.iter() {
            draw_text(&mut engine.renderer, &engine.font, port, [offset_x, y]);
            y += height;
        }

        draw_text(&mut engine.renderer,
                  &engine.font,
                  ">",
                  [0, self.selected as i32 * height]);

        engine.renderer.present();

        Trans::None
    }
}
