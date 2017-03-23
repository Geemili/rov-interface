
use super::Control;
use gilrs;
use std::io::Write;
use rov::RovCommand;

pub struct MotorInfo {
    pub id: u8,
    pub position: [f32; 3],
    pub direction: [f32; 3],
}

pub struct Motor {
    pub info: MotorInfo,
    pub thrust: i16,
    pub prev_thrust: i16,
}

impl Control for Motor {
    fn update(&mut self, input: &gilrs::GamepadState) {
        self.prev_thrust = self.thrust;
        self.thrust = input.value(gilrs::Axis::LeftStickY) as i16;
    }

    fn write_commands(&self, output: &mut Vec<RovCommand>) {
        use ::rov::RovCommand::ControlMotor;
        if self.thrust != self.prev_thrust {
            output.push(ControlMotor {
                id: self.info.id,
                throttle: self.thrust,
            });
        }
    }
}

