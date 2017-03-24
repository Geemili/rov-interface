
use super::Control;
use gilrs;
use rov::RovCommand;
use vecmath;

pub struct Servo {
    // info
    pub id: u8,
    increase_button: gilrs::Button,
    decrease_button: gilrs::Button,
    // state
    pub microseconds: i16,
    pub prev_microseconds: i16,
}

impl Servo {
    pub fn new(id: u8, increase_button: gilrs::Button, decrease_button: gilrs::Button) -> Self {
        Servo {
            id: id,
            increase_button: increase_button,
            decrease_button: decrease_button,
            microseconds: 0,
            prev_microseconds: 0,
        }
    }
}

impl Control for Servo {
    fn update(&mut self, input: &gilrs::GamepadState) {
        self.prev_microseconds = self.microseconds;
        let increase = input.is_pressed(self.increase_button);
        let decrease = input.is_pressed(self.decrease_button);

        match (increase, decrease) {
            (true, false) => self.microseconds += 10,
            (false, true) => self.microseconds -= 10,
            _ => {}
        }
    }

    fn write_commands(&self, output: &mut Vec<RovCommand>) {
        use ::rov::RovCommand::ControlServo;
        if self.microseconds != self.prev_microseconds {
            output.push(ControlServo {
                id: self.id,
                microseconds: self.microseconds,
            });
        }
    }
}

