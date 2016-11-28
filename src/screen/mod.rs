
pub mod control_rov;

pub enum Trans {
    Quit,
    None,
    Switch(Box<Screen>),
}

pub trait Screen {
    fn update(&mut self, engine: &mut Engine) -> Trans;
}

use sdl2::{EventPump, GameControllerSubsystem};
use sdl2::render::Renderer;
use sdl2_ttf::Font;

pub struct Engine<'renderer> {
    pub event_pump: EventPump,
    pub controllers: GameControllerSubsystem,
    pub renderer: Renderer<'renderer>,
    pub font: Font<'renderer>,
}
