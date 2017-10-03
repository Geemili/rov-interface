
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
    fn update(&mut self, engine: &mut Engine, delta: f64) -> Result<Trans>;
}

use sdl2::EventPump;
use sdl2::render::{Renderer, Texture};
use sdl2_ttf::Font as SdlFont;
use rusttype::Font;
use rusttype::gpu_cache::Cache;
use gilrs;
use config::Config;

pub struct Engine<'renderer> {
    pub event_pump: EventPump,
    pub controllers: gilrs::Gilrs,
    pub renderer: Renderer<'renderer>,
    pub font: SdlFont<'renderer>,
    pub rfont: Font<'renderer>,
    pub cache: Cache,
    pub cache_texture: Texture,
    pub config: Config,
}
