
pub mod control_rov;
pub mod port_select;

use ::errors::*;

pub enum Trans {
    Quit,
    None,
    Switch(Box<Screen>),
}

pub trait Screen {
    fn init(&mut self, engine: &mut Engine) -> Result<()>;
    fn update(&mut self, engine: &mut Engine, delta: f64) -> Trans;
}

use sdl2::EventPump;
use sdl2::render::Renderer;
use sdl2_ttf::Font;
use gilrs;
use config::Config;

pub struct Engine<'renderer> {
    pub event_pump: EventPump,
    pub controllers: gilrs::Gilrs,
    pub renderer: Renderer<'renderer>,
    pub font: Font<'renderer>,
    pub config: Config,
}
