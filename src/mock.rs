
/// A mock ROV that reflects the state of the ROV.

use rov::RovCommand;
use time::{self, Tm, Duration};

pub struct MockRov {
    pub motors: [i16; 6],
    pub robot_is_on: bool,
    pub light_relay: bool,
    pub sampler_relay: bool,
    pub turn_off_motor: Option<Tm>,
}

impl MockRov {
    pub fn new() -> MockRov {
        MockRov {
            motors: [0; 6],
            robot_is_on: true,
            light_relay: false,
            sampler_relay: false,
            turn_off_motor: None,
        }
    }

    pub fn apply_commands(&mut self, commands: &Vec<RovCommand>) {
        for command in commands.iter() {
            self.apply_command(command);
        }
    }

    pub fn apply_command(&mut self, command: &RovCommand) {
        match *command {
            RovCommand::MasterOn => {
                self.master_on();
                return;
            }
            _ => {}
        }
        if !self.robot_is_on {
            return;
        }
        match *command {
            RovCommand::ControlMotor { id, throttle } => {
                if (id as usize) < self.motors.len() {
                    self.motors[id as usize] = throttle;
                }
            }
            RovCommand::CollectSamples { amount } => {
                self.sampler_relay = true;
                self.turn_off_motor = Some(time::now() +
                                           Duration::milliseconds(2000 * amount as i64));
            }
            RovCommand::LightsOn => self.light_relay = true,
            RovCommand::LightsOff => self.light_relay = false,
            RovCommand::MasterOn => {
                unreachable!();
            }
            RovCommand::MasterOff => {
                self.robot_is_on = false;
                self.light_relay = false;
                for motor in self.motors.iter_mut() {
                    *motor = 0;
                }
            }
        }
    }

    pub fn update(&mut self) {
        if let Some(turn_off_time) = self.turn_off_motor {
            if time::now() >= turn_off_time {
                self.sampler_relay = false;
                self.turn_off_motor = None;
            }
        }
    }

    fn master_on(&mut self) {
        self.robot_is_on = true;
        self.light_relay = false;
        for motor in self.motors.iter_mut() {
            *motor = 0;
        }
    }
}
