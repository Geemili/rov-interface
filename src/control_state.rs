
use std;
use rov::RovCommand;
use vecmath;

// Corresponding to the BlueROV Vectored ROV configuration
pub const MOTOR_1: u8 = 0;
pub const MOTOR_2: u8 = 1;
pub const MOTOR_3: u8 = 2;
pub const MOTOR_4: u8 = 3;
pub const MOTOR_5: u8 = 4;
pub const MOTOR_6: u8 = 5;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum ThrustMode {
    Normal,
    Emergency,
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum SamplerReleaseMode {
    One,
    All,
}

#[derive(PartialEq, Clone)]
pub struct ControlState {
    pub thrust_mode: ThrustMode,
    pub forward_thrust: f64,
    pub sideways_thrust: f64,
    pub rotational_thrust: f64,
    pub ascent_thrust: f64,
    pub descent_thrust: f64,

    pub sampler_release_mode: SamplerReleaseMode,
    pub sampler_release: bool,
    pub sampler_release_latch: bool,

    pub power_master: bool,
    pub power_lights: bool,
}


const MOTOR_1_VEC: [f64; 2] = [0.5, 0.5];
const MOTOR_2_VEC: [f64; 2] = [0.5, -0.5];
const MOTOR_3_VEC: [f64; 2] = [-0.5, 0.5];
const MOTOR_4_VEC: [f64; 2] = [-0.5, -0.5];

impl ControlState {
    pub fn new() -> ControlState {
        ControlState {
            thrust_mode: ThrustMode::Normal,
            forward_thrust: 0.0,
            sideways_thrust: 0.0,
            rotational_thrust: 0.0,
            ascent_thrust: 0.0,
            descent_thrust: 0.0,

            sampler_release_mode: SamplerReleaseMode::One,
            sampler_release: false,
            sampler_release_latch: false,

            power_master: true,
            power_lights: false,
        }
    }

    pub fn generate_commands_diff(&self, other: &ControlState, buffer: &mut Vec<RovCommand>) {
        // Horizontal movement
        if self.forward_thrust != other.forward_thrust ||
           self.sideways_thrust != other.sideways_thrust ||
           self.thrust_mode != other.thrust_mode {
            match self.thrust_mode {
                ThrustMode::Normal => {
                    // TODO: Research doing this with ints.
                    let control_vector = [self.forward_thrust, self.sideways_thrust];

                    // Find out the magnitude of all the motors
                    let motor_1_throttle = vecmath::vec2_dot(control_vector, MOTOR_1_VEC);
                    let motor_2_throttle = vecmath::vec2_dot(control_vector, MOTOR_2_VEC);
                    let motor_3_throttle = vecmath::vec2_dot(control_vector, MOTOR_3_VEC);
                    let motor_4_throttle = vecmath::vec2_dot(control_vector, MOTOR_4_VEC);

                    buffer.push(RovCommand::ControlMotor {
                        id: MOTOR_1,
                        throttle: (motor_1_throttle * std::i16::MAX as f64) as i16,
                    });
                    buffer.push(RovCommand::ControlMotor {
                        id: MOTOR_2,
                        throttle: (motor_2_throttle * std::i16::MAX as f64) as i16,
                    });
                    buffer.push(RovCommand::ControlMotor {
                        id: MOTOR_3,
                        throttle: (motor_3_throttle * std::i16::MAX as f64) as i16,
                    });
                    buffer.push(RovCommand::ControlMotor {
                        id: MOTOR_4,
                        throttle: (motor_4_throttle * std::i16::MAX as f64) as i16,
                    });
                }
                ThrustMode::Emergency => {
                    buffer.push(RovCommand::ControlMotor {
                        id: MOTOR_1,
                        throttle: (self.forward_thrust * std::i16::MAX as f64) as i16,
                    });
                    buffer.push(RovCommand::ControlMotor {
                        id: MOTOR_2,
                        throttle: (self.forward_thrust * std::i16::MAX as f64) as i16,
                    });
                    buffer.push(RovCommand::ControlMotor {
                        id: MOTOR_3,
                        throttle: (-self.forward_thrust * std::i16::MAX as f64) as i16,
                    });
                    buffer.push(RovCommand::ControlMotor {
                        id: MOTOR_4,
                        throttle: (-self.forward_thrust * std::i16::MAX as f64) as i16,
                    });
                }
            }
        }

        // Vertical movement
        if self.ascent_thrust != other.ascent_thrust ||
           self.descent_thrust != other.descent_thrust {
            buffer.push(RovCommand::ControlMotor {
                id: MOTOR_5,
                throttle: ((self.ascent_thrust - other.descent_thrust) *
                           std::i16::MAX as f64) as i16,
            });
            buffer.push(RovCommand::ControlMotor {
                id: MOTOR_6,
                throttle: ((self.ascent_thrust - other.descent_thrust) *
                           std::i16::MAX as f64) as i16,
            });
        }

        // Master power
        match (self.power_master, other.power_master) {
            (true, false) => {
                buffer.push(RovCommand::MasterOn);
            }
            (false, true) => {
                buffer.push(RovCommand::MasterOff);
            }
            _ => {
                // The mode didn't change; there is no need to send a command
            }
        }

        // Lights
        match (self.power_lights, other.power_lights) {
            (true, false) => {
                buffer.push(RovCommand::LightsOn);
            }
            (false, true) => {
                buffer.push(RovCommand::LightsOff);
            }
            _ => {
                // The mode didn't change; there is no need to send a command
            }
        }

        // Sampler
        match (self.sampler_release, other.sampler_release) {
            (true, false) => {
                let amount = match self.sampler_release_mode {
                    SamplerReleaseMode::One => 1,
                    SamplerReleaseMode::All => 6,
                };
                buffer.push(RovCommand::CollectSamples { amount: amount });
            }
            _ => {}
        }
    }
}
