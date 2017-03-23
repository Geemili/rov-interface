
#![recursion_limit = "1024"]

extern crate serial;
extern crate sdl2;
extern crate sdl2_ttf;
#[macro_use]
extern crate fomat_macros;
#[macro_use]
extern crate error_chain;
extern crate vecmath;
extern crate time;
extern crate serial_enumerate;
extern crate gilrs;

mod errors;
mod rov;
mod mock;
mod control_state;
mod util;
mod screen;
mod control;

use std::path::Path;
use std::env;

fn main() {
    let serialport_path = env::args().skip(1).next();
    let sdl_context = sdl2::init().unwrap();
    let gilrs = gilrs::Gilrs::new();
    let video = sdl_context.video().unwrap();
    let event_pump = sdl_context.event_pump().unwrap();
    let ttf_context = sdl2_ttf::init().unwrap();

    let window =
        video.window("ROV Interface", 800, 600).position_centered().opengl().build().unwrap();

    let font = ttf_context.load_font(Path::new("assets/fonts/NotoSans/NotoSans-Regular.ttf"), 64)
        .unwrap();

    let renderer = window.renderer().accelerated().build().unwrap();

    let mut engine = screen::Engine {
        event_pump: event_pump,
        controllers: gilrs,
        renderer: renderer,
        font: font,
    };

    use screen::Screen;
    let mut screen: Box<Screen> = match serialport_path {
        Some(path) => {
            use screen::control_rov::RovControl;
            use rov::Rov;
            let rov = Rov::new(path.into());
            Box::new(RovControl::new(rov))
        }
        None => Box::new(screen::port_select::PortSelect::new()),
    };

    loop {
        let current_screen = match screen.update(&mut engine) {
            screen::Trans::Quit => break,
            screen::Trans::None => screen,
            screen::Trans::Switch(new_screen) => new_screen,
        };
        screen = current_screen;
    }
}

