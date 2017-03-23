
const TIME_BETWEEN_POLLING_PORTS_MS: i64 = 1_000;

use time::{PreciseTime, Duration};
use serial_enumerate;
use util::draw_text;
use screen::{Engine, Screen, Trans};
use screen::control_rov::RovControl;
use rov::Rov;

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

    fn select_next(&mut self) {
        if self.ports.len() > 0 && self.selected < self.ports.len() - 1 {
            self.selected += 1;
        }
    }

    fn select_previous(&mut self) {
        if self.ports.len() > 0 && self.selected > 0 {
            self.selected -= 1;
        }
    }
}

impl Screen for PortSelect {
    fn update(&mut self, engine: &mut Engine) -> Trans {
        for (_id, event) in engine.controllers.poll_events() {
            use gilrs::Event::ButtonReleased as Press;
            use gilrs::Button::{DPadUp, DPadDown, South};

            match event {
                Press(DPadDown, _) => self.select_next(),
                Press(DPadUp, _) => self.select_previous(),
                Press(South, _) => {
                    if self.ports.len() > 0 {
                        let ref port_name = self.ports[self.selected];
                        let rov = Rov::new(port_name.into());
                        let control_screen = Box::new(RovControl::new(rov));
                        return Trans::Switch(control_screen);
                    }
                }
                _ => (),
            }
        }
        for event in engine.event_pump.poll_iter() {
            use sdl2::event::Event;
            use sdl2::keyboard::Keycode;

            match event {
                Event::KeyDown { keycode: Some(Keycode::Down), .. } => self.select_next(),
                Event::KeyDown { keycode: Some(Keycode::Up), .. } => self.select_previous(),
                Event::KeyDown { keycode: Some(Keycode::Return), .. } => {
                    if self.ports.len() > 0 {
                        let ref port_name = self.ports[self.selected];
                        let rov = Rov::new(port_name.into());
                        let control_screen = Box::new(RovControl::new(rov));
                        return Trans::Switch(control_screen);
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
            if self.selected >= self.ports.len() {
                self.selected = self.ports.len() - 1;
            }
        }

        engine.renderer.clear();

        let offset_x = 64;
        let height = 64;
        let mut y = 0;

        for port in self.ports.iter() {
            draw_text(&mut engine.renderer, &engine.font, port, [offset_x, y]);
            y += height;
        }


        if self.ports.len() > 0 {
            draw_text(&mut engine.renderer,
                      &engine.font,
                      ">",
                      [0, self.selected as i32 * height]);
        }

        engine.renderer.present();

        Trans::None
    }
}
