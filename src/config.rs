
#[derive(Deserialize, Clone, Debug, Default)]
pub struct Config {
    pub control: Controls,
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct Controls {
    pub servo_pan: Servo,
    pub servo_tilt: Servo,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Servo {
    pub speed: f64,
}

use std::default::Default;

impl Default for Servo {
    fn default() -> Servo {
        Servo { speed: ::control::servo::DEFAULT_MOVE_SPEED }
    }
}
