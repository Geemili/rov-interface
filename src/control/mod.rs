
pub mod motor;
pub mod lights;

use gilrs;
use ::rov::RovCommand;

pub const INT_MAX: f32 = 32767.0;

pub trait Control {
    fn update(&mut self, input: &gilrs::GamepadState);
    fn write_commands(&self, output: &mut Vec<RovCommand>);
}

