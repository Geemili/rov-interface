
use super::Control;
use gilrs;
use rov::RovCommand;

pub const SERVO_LOW: i16 = 1000;
pub const SERVO_MID: i16 = 1500;
pub const SERVO_HIGH: i16 = 2000;
const SECONDS_TO_CROSS: f64 = 2.0;
pub const DEFAULT_MOVE_SPEED: f64 = (SERVO_HIGH as f64 - SERVO_LOW as f64) / SECONDS_TO_CROSS;

pub struct Servo {
    // info
    pub id: u8,
    increase_button: gilrs::Button,
    decrease_button: gilrs::Button,
    move_speed: f64,
    // state
    pub microseconds: i16,
    pub prev_microseconds: i16,
}

impl Servo {
    pub fn new(id: u8,
               increase_button: gilrs::Button,
               decrease_button: gilrs::Button,
               move_speed: f64)
               -> Self {
        Servo {
            id: id,
            increase_button: increase_button,
            decrease_button: decrease_button,
            move_speed: move_speed,
            microseconds: SERVO_MID,
            prev_microseconds: SERVO_MID,
        }
    }
}

impl Control for Servo {
    fn update(&mut self, input: &gilrs::GamepadState, delta: f64) {
        self.prev_microseconds = self.microseconds;
        let increase = input.is_pressed(self.increase_button);
        let decrease = input.is_pressed(self.decrease_button);

        self.microseconds = match (increase, decrease) {
            (true, false) => self.microseconds + (self.move_speed * delta) as i16,
            (false, true) => self.microseconds - (self.move_speed * delta) as i16,
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
        use rov::RovCommand::ControlServo;
        if self.microseconds != self.prev_microseconds {
            output.push(ControlServo {
                id: self.id,
                microseconds: self.microseconds,
            });
        }
    }
}
