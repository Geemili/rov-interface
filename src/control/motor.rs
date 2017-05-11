
use super::Control;
use gilrs;
use rov::RovCommand;
use vecmath;

pub struct MotorBuilder {
    pub id: Option<u8>,
    pub position: Option<[f32; 3]>,
    pub direction: Option<[f32; 3]>,
}

impl MotorBuilder {
    pub fn new() -> Self {
        MotorBuilder {
            id: None,
            position: None,
            direction: None,
        }
    }

    pub fn id(mut self, id: u8) -> Self {
        self.id = Some(id);
        self
    }

    pub fn position(mut self, position: [f32; 3]) -> Self {
        self.position = Some(position);
        self
    }

    pub fn direction(mut self, direction: [f32; 3]) -> Self {
        self.direction = Some(direction);
        self
    }

    pub fn build(self) -> Motor {
        let direction = self.direction.unwrap_or([1.0, 0.0, 0.0]);
        let angle = (direction[1]/direction[0]).tan();
        let rotation_coefficient = match (self.position, angle.is_nan()) {
            (_,true) => 0.0,
            (Some(pos), _) if pos == [0.0;3] => 0.0,
            (Some(pos), _) => {
                let angle_from_origin = (pos[1]/pos[0]).tan();
                use std::f32::consts;
                ((angle-angle_from_origin)/consts::PI).sin()
            }
            _ => 0.0,
        };
        Motor {
            id: self.id.unwrap_or(0),
            direction: direction,
            rotation_coefficient: rotation_coefficient,
            thrust: 0,
            prev_thrust: 0,
        }
    }
}

pub struct Motor {
    // info
    pub id: u8,
    pub direction: [f32; 3],
    pub rotation_coefficient: f32,
    // state
    pub thrust: i16,
    pub prev_thrust: i16,
}

impl Control for Motor {
    fn update(&mut self, input: &gilrs::GamepadState, _delta: f64) {
        self.prev_thrust = self.thrust;
        let forward = input.value(gilrs::Axis::LeftStickY);
        let sideways = input.value(gilrs::Axis::LeftStickX);
        let ascent = input.value(gilrs::Axis::LeftTrigger2);
        let descent = input.value(gilrs::Axis::RightTrigger2);
        let rotational = input.value(gilrs::Axis::RightStickX);

        let control_vector = [forward, sideways, ascent - descent];
        let thrust = vecmath::vec3_dot(control_vector, self.direction);
        let thrust = thrust + (rotational * self.rotation_coefficient);
        let thrust = thrust.max(-1.0).min(1.0);
        self.thrust = (thrust * super::INT_MAX) as i16;
    }

    fn write_commands(&self, output: &mut Vec<RovCommand>) {
        use ::rov::RovCommand::ControlMotor;
        if self.thrust != self.prev_thrust {
            output.push(ControlMotor {
                id: self.id,
                throttle: self.thrust,
            });
        }
    }
}

