
pub mod motor;

use std::io::Write;
use gilrs;
use ::rov::RovCommand;

pub trait Control {
    fn update(&mut self, input: &gilrs::GamepadState);
    fn write_commands(&self, output: &mut Vec<RovCommand>);
}

