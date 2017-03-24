
use super::Control;
use gilrs;
use rov::RovCommand::{self, MasterOn, MasterOff};

pub struct MasterPower {
    // info
    button: gilrs::Button,
    // state
    master_power: bool,
    was_pressed: bool,
    should_send: bool,
}

impl MasterPower {
    pub fn new(button: gilrs::Button) -> Self {
        MasterPower {
            button: button,
            master_power: false,
            was_pressed: false,
            should_send: false,
        }
    }
}

impl Control for MasterPower {
    fn update(&mut self, input: &gilrs::GamepadState) {
        self.should_send = false;
        let is_pressed = input.is_pressed(self.button);
        match (self.was_pressed, is_pressed) {
            (false, true) => {
                self.master_power = !self.master_power;
                self.should_send = true;
            }
            _ => {}
        }
        self.was_pressed = is_pressed;
    }

    fn write_commands(&self, output: &mut Vec<RovCommand>) {
        if self.should_send {
            output.push(
                if self.master_power {
                    MasterOn
                } else {
                    MasterOff
                }
                );
        }
    }
}

