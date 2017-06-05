
use super::Control;
use gilrs;
use rov::RovCommand::{self, LightsOn, LightsOff};

pub struct Lights {
    // info
    button: gilrs::Button,
    // state
    lights_state: bool,
    was_pressed: bool,
    need_to_write: bool,
}

impl Lights {
    pub fn new(button: gilrs::Button) -> Self {
        Lights {
            button: button,
            lights_state: false,
            was_pressed: false,
            need_to_write: true,
        }
    }
}

impl Control for Lights {
    fn update(&mut self, input: &gilrs::GamepadState, _delta: f64) {
        let is_pressed = input.is_pressed(self.button);
        match (self.was_pressed, is_pressed) {
            (false, true) => {
                self.lights_state = !self.lights_state;
                self.need_to_write = true;
            }
            _ => {
                self.need_to_write = false;
            }
        }
        self.was_pressed = is_pressed;
    }

    fn write_commands(&self, output: &mut Vec<RovCommand>) {
        if self.need_to_write {
            output.push(if self.lights_state {
                LightsOn
            } else {
                LightsOff
            });
        }
    }
}
