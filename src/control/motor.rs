
use super::Control;
use gilrs;
use std::io::Write;
use rov::RovCommand;
use vecmath;

// TODO: Make this into a builder
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
        let forward = input.value(gilrs::Axis::LeftStickY);
        let sideways = input.value(gilrs::Axis::LeftStickX);
        let ascent = input.value(gilrs::Axis::LeftTrigger);
        let descent = input.value(gilrs::Axis::RightTrigger);
        let _rotational = input.value(gilrs::Axis::RightStickX);
        // let motor_rotation_sign = {
            // TODO: calculate from position and direction
            // Find how well a motor fits on an imaginary circle
        // };

        let control_vector = [forward, sideways, ascent - descent];
        let thrust = vecmath::vec3_dot(control_vector, self.info.direction);
        let thrust = thrust.max(-1.0).min(1.0);
        self.thrust = (thrust * super::INT_MAX) as i16;
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

