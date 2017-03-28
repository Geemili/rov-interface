
/// A mock ROV that reflects the state of the ROV.

use rov::RovResponse;

pub struct MockRov {
    pub motors: [i16; 6],
    pub servos: [i16; 2],
    pub robot_is_on: bool,
    pub light_relay: bool,
    pub sampler_relay: bool,
}

impl MockRov {
    pub fn new() -> MockRov {
        MockRov {
            motors: [0; 6],
            servos: [1500; 2], // Start it at the middle
            robot_is_on: true,
            light_relay: false,
            sampler_relay: false,
        }
    }

    pub fn apply_responses(&mut self, commands: &Vec<RovResponse>) {
        for command in commands.iter() {
            self.apply_response(command);
        }
    }

    pub fn apply_response(&mut self, command: &RovResponse) {
        match *command {
            RovResponse::Motor { id, throttle } => {
                if (id as usize) < self.motors.len() {
                    self.motors[id as usize] = throttle;
                }
            }
            RovResponse::CollectingSamples { .. } => self.sampler_relay = true,
            RovResponse::CollectingSamplesNot => self.sampler_relay = false,
            RovResponse::LightsOn => self.light_relay = true,
            RovResponse::LightsOff => self.light_relay = false,
            RovResponse::MasterOn => self.robot_is_on = true,
            RovResponse::MasterOff => self.robot_is_on = false,
            RovResponse::Servo { id, microseconds } => {
                if (id as usize) < self.servos.len() {
                    self.servos[id as usize] = microseconds;
                }
            }
        }
    }

}
