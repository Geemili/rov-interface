
/// A mock ROV that reflects the state of the ROV.

use rov::RovCommand;

pub struct MockRov {
    pub motors: [i16; 6],
    pub light_relay: bool,
}

impl MockRov {
    pub fn new() -> MockRov {
        MockRov {
            motors: [0; 6],
            light_relay: false,
        }
    }

    pub fn apply_commands(&mut self, commands: &Vec<RovCommand>) {
        for command in commands.iter() {
            self.apply_command(command);
        }
    }

    pub fn apply_command(&mut self, command: &RovCommand) {
        match *command {
            RovCommand::ControlMotor { id, throttle } => {
                if (id as usize) < self.motors.len() {
                    self.motors[id as usize] = throttle;
                }
            }
            RovCommand::CollectSamples { .. } => {
                unimplemented!();
            }
            RovCommand::LightsOn => self.light_relay = true,
            RovCommand::LightsOff => self.light_relay = false,
        }
    }
}
