
const TIME_BETWEEN_POLLING_PORTS_MS: i64 = 1_000;

use time::{PreciseTime, Duration};
use serialport;
use screen::{Engine, Screen, Trans};
use screen::control_rov::RovControl;
use rov::Rov;
use ::errors::*;

pub struct PortSelect {
    ports: Vec<serialport::SerialPortInfo>,
    selected: usize,
    last_poll_time: PreciseTime,
}

impl PortSelect {
    pub fn new() -> PortSelect {
        PortSelect {
            ports: vec![],
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
    fn init(&mut self, _engine: &mut Engine) -> Result<()> {
        Ok(())
    }

    fn update(&mut self, engine: &mut Engine, _delta: f64) -> Result<Trans> {
        for (_id, event) in engine.controllers.poll_events() {
            use gilrs::Event::ButtonReleased as Press;
            use gilrs::Button::{DPadUp, DPadDown, South};

            match event {
                Press(DPadDown, _) => self.select_next(),
                Press(DPadUp, _) => self.select_previous(),
                Press(South, _) => {
                    if self.ports.len() > 0 {
                        let port = self.ports[self.selected].port_name.clone().into();
                        let rov = Rov::new(port);
                        let control_screen = Box::new(RovControl::new(rov));
                        return Ok(Trans::Switch(control_screen));
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
                        let port = self.ports[self.selected].port_name.clone().into();
                        let rov = Rov::new(port);
                        let control_screen = Box::new(RovControl::new(rov));
                        return Ok(Trans::Switch(control_screen));
                    }
                }
                Event::Quit { .. } |
                Event::KeyUp { keycode: Some(Keycode::Escape), .. } => return Ok(Trans::Quit),
                _ => (),
            }
        }

        if self.last_poll_time.to(PreciseTime::now()) >=
           Duration::milliseconds(TIME_BETWEEN_POLLING_PORTS_MS) {
            self.ports = serialport::available_ports().chain_err(|| "Couldn't list of ports")?;
            self.last_poll_time = PreciseTime::now();
            if self.selected >= self.ports.len() {
                self.selected = self.ports.len() - 1;
            }
        }

        Ok(Trans::None)
    }

    fn render(&mut self, engine: &mut Engine, delta: f64) -> Result<()> {
        let offset_x = 64.0;
        let height = 64.0;
        let mut y = 0.0;

        for port in self.ports.iter() {
            engine.queue_text(offset_x, y,
                             ::rusttype::Scale::uniform(height),
                             &port.port_name);
            y += height;
        }


        if self.ports.len() > 0 {
            engine.queue_text(0.0, self.selected as f32 * height,
                             ::rusttype::Scale::uniform(height),
                             ">");
        }


        Ok(())
    }
}
