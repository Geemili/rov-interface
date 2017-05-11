
#![recursion_limit = "1024"]

extern crate sdl2;
extern crate sdl2_ttf;
#[macro_use]
extern crate fomat_macros;
#[macro_use]
extern crate error_chain;
extern crate vecmath;
extern crate time;
extern crate serialport;
extern crate gilrs;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate toml;

pub mod errors;
mod rov;
mod mock;
mod util;
mod screen;
mod control;
mod config;

use errors::*;

fn main() {
    if let Err(ref e) = run() {
        println!("Error: {}", e);

        for e in e.iter().skip(1) {
            println!("Caused by: {}", e);
        }

        // If there is a backtrace, print it.
        if let Some(backtrace) = e.backtrace() {
            println!("backtrace: {:?}", backtrace);
        }

        ::std::process::exit(1);
    }
}

use std::path::Path;
use std::env;

fn run() -> Result<()> {
    let serialport_path = env::args().skip(1).next();
    let sdl_context = sdl2::init().map_err(|msg| Error::from_kind(ErrorKind::SdlMsg(msg)))
        .chain_err(|| "Failed to initialize SDL context")?;
    let gilrs = gilrs::Gilrs::new();
    let video = sdl_context.video()
        .map_err(|msg| Error::from_kind(ErrorKind::SdlMsg(msg)))
        .chain_err(|| "Failed to get video context")?;
    let event_pump = sdl_context.event_pump()
        .map_err(|msg| Error::from_kind(ErrorKind::SdlMsg(msg)))
        .chain_err(|| "Failed to get event pump")?;
    let ttf_context =
        sdl2_ttf::init().map_err(|err| Error::from_kind(ErrorKind::SdlMsg(format!("{:?}", err))))
            .chain_err(|| "Failed to get font context")?;

    let window = video.window("ROV Interface", 800, 600)
        .position_centered()
        .opengl()
        .build()
        .chain_err(|| "Failed to build SDL window")?;

    let font = ttf_context.load_font(Path::new("assets/fonts/NotoSans/NotoSans-Regular.ttf"), 64)
        .map_err(|font_error| Error::from_kind(ErrorKind::SdlMsg(format!("{:?}", font_error))))
        .chain_err(|| "Failed to load font")?;

    let renderer =
        window.renderer().accelerated().build().chain_err(|| "Failed to accelerate renderer")?;

    let config = match util::load_config_from_file("config.toml") {
        Ok(config) => config,
        // TODO: Log error
        Err(_e) => ::config::Config::default(),
    };

    let mut engine = screen::Engine {
        event_pump: event_pump,
        controllers: gilrs,
        renderer: renderer,
        font: font,
        config: config,
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

    let mut prev_time = ::std::time::Instant::now();
    loop {
        let elapsed = prev_time.elapsed();
        let delta = (elapsed.as_secs() as f64) + (elapsed.subsec_nanos() as f64 / 1_000_000_000.0);
        prev_time = ::std::time::Instant::now();

        let current_screen = match screen.update(&mut engine, delta) {
            screen::Trans::Quit => break,
            screen::Trans::None => screen,
            screen::Trans::Switch(mut new_screen) => {
                new_screen.init(&mut engine).chain_err(|| "Failed to initialize screen")?;
                new_screen
            }
        };
        screen = current_screen;
    }

    Ok(())
}
