
use super::Control;
use gilrs;
use rov::RovCommand::{self, LightsOn, LightsOff};

pub struct Lights {
    // info
    button: gilrs::Button,
    // state
    lights_state: bool,
    was_pressed: bool,
}

impl Lights {
    pub fn new(button: gilrs::Button) -> Self {
        Lights {
            button: button,
            lights_state: false,
            was_pressed: false,
        }
    }
}

impl Control for Lights {
    fn update(&mut self, input: &gilrs::GamepadState) {
        let is_pressed = input.is_pressed(self.button);
        match (self.was_pressed, is_pressed) {
            (false, true) => {
                self.lights_state = !self.lights_state;
            }
            _ => {}
        }
        self.was_pressed = is_pressed;
    }

    fn write_commands(&self, output: &mut Vec<RovCommand>) {
        output.push(
            if self.lights_state {
                LightsOn
            } else {
                LightsOff
            }
            );
    }
}

