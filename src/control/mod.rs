
pub mod motor;
pub mod lights;
pub mod master;
pub mod servo;

use gilrs;
use rov::RovCommand;

pub const INT_MAX: f32 = 32767.0;

pub trait Control {
    fn update(&mut self, input: &gilrs::GamepadState, delta: f64);
    fn write_commands(&self, output: &mut Vec<RovCommand>);
}
