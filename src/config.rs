
#[derive(Deserialize, Clone, Debug, Default)]
pub struct Config {
    control: Controls
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct Controls {
    servo_pan: Servo,
    servo_tilt: Servo,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Servo {
    speed: f64,
}

use std::default::Default;

impl Default for Servo {
    fn default() -> Servo {
        Servo {
            speed: 40.0,
        }
    }
}

