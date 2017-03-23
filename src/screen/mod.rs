
pub mod control_rov;
pub mod port_select;

pub enum Trans {
    Quit,
    None,
    Switch(Box<Screen>),
}

pub trait Screen {
    fn update(&mut self, engine: &mut Engine) -> Trans;
}

use sdl2::EventPump;
use sdl2::render::Renderer;
use sdl2_ttf::Font;
use gilrs;

pub struct Engine<'renderer> {
    pub event_pump: EventPump,
    pub controllers: gilrs::Gilrs,
    pub renderer: Renderer<'renderer>,
    pub font: Font<'renderer>,
}
