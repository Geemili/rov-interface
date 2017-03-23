
pub mod motor;

use std::io::Write;
use gilrs;
use ::rov::RovCommand;

pub const INT_MAX: f32 = 32768.0;

pub trait Control {
    fn update(&mut self, input: &gilrs::GamepadState);
    fn write_commands(&self, output: &mut Vec<RovCommand>);
}

