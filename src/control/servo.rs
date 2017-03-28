
use super::Control;
use gilrs;
use rov::RovCommand;

pub const SERVO_LOW: i16 = 1000;
pub const SERVO_MID: i16 = 1500;
pub const SERVO_HIGH: i16 = 2000;

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
            microseconds: SERVO_MID,
            prev_microseconds: SERVO_MID,
        }
    }
}

impl Control for Servo {
    fn update(&mut self, input: &gilrs::GamepadState) {
        self.prev_microseconds = self.microseconds;
        let increase = input.is_pressed(self.increase_button);
        let decrease = input.is_pressed(self.decrease_button);

        self.microseconds = match (increase, decrease) {
            (true, false) => self.microseconds.saturating_add(10),
            (false, true) => self.microseconds.saturating_sub(10),
            _ => self.microseconds,
        };
        if self.microseconds < SERVO_LOW {
            self.microseconds = SERVO_LOW;
        }
        if self.microseconds > SERVO_HIGH {
            self.microseconds = SERVO_HIGH;
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

