
#![recursion_limit = "1024"]
#![feature(receiver_try_iter)]

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

mod errors;
mod rov;
mod mock;
mod control_state;
mod util;
mod screen;

use errors::*;
use std::path::Path;
use sdl2::pixels::Color;
use control_state::{ControlState, ThrustMode, SamplerReleaseMode};

fn main() {
    use std;
    let port_name = if let Some(port) = std::env::args().skip(1).next() {
        pintln!("Writing to port "(port));
        String::from(port.trim())
    } else {
        panic!("Port name is required");
    };

    for device in serial_enumerate::enumerate_serial_ports().unwrap() {
        println!("{}", device);
    }

    let mut rov = rov::Rov::new(port_name.into());

    let sdl_context = sdl2::init().unwrap();
    let mut game_controller_subsystem = sdl_context.game_controller().unwrap();
    let video = sdl_context.video().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let ttf_context = sdl2_ttf::init().unwrap();

    let window =
        video.window("ROV Interface", 800, 600).position_centered().opengl().build().unwrap();

    let mut renderer = window.renderer().accelerated().build().unwrap();

    match load_mappings(&mut game_controller_subsystem) {
        Ok(_) => {}
        Err(_) => pintln!("Couldn't load mappings"),
    }
    let font = ttf_context.load_font(Path::new("assets/fonts/NotoSans/NotoSans-Regular.ttf"), 64)
        .unwrap();

    let available = match game_controller_subsystem.num_joysticks() {
        Ok(n) => n,
        Err(e) => panic!("Can't enumerate joysticks. :( {:?}", e),
    };

    pintln!((available)" game controllers available");

    let mut game_controllers = None;

    // Iterate over all available game_controllerss and stop once we manage to
    // open one.
    for id in 0..available {
        if game_controller_subsystem.is_game_controller(id) {
            match game_controller_subsystem.open(id) {
                Ok(c) => {
                    pintln!("Success: opened \""(c.name())"\".");
                    game_controllers = Some(c);
                    break;
                }
                Err(e) => pintln!("failed: "[e]),
            }
        } else {
            pintln!("Controller "(id)" has no mapping.");
        }
    }

    if game_controllers.is_none() {
        panic!("Couldn't open any joystick");
    };

    let mut engine = screen::Engine {
        event_pump: event_pump,
        controllers: game_controller_subsystem,
        renderer: renderer,
        font: font,
    };

    use screen::Screen;
    let mut screen: Box<Screen> = Box::new(screen::port_select::PortSelect::new());

    loop {
        let current_screen = match screen.update(&mut engine) {
            screen::Trans::Quit => break,
            screen::Trans::None => screen,
            screen::Trans::Switch(mut new_screen) => new_screen,
        };
        screen = current_screen;
    }
}

fn load_mappings(game_controller_subsystem: &mut sdl2::GameControllerSubsystem) -> Result<()> {
    use std::fs::OpenOptions;
    use std::io::{BufReader, BufRead};
    let file = OpenOptions::new().read(true)
        .open("assets/controller_mappings.csv")
        .chain_err(|| "Unable to load controller mappings")?;
    let reader = BufReader::new(&file);
    for line in reader.lines() {
        let l = line.chain_err(|| "Error reading line")?;
        if l == "" {
            continue;
        }
        match game_controller_subsystem.add_mapping(l.trim()) {
            Ok(_) => {}
            Err(e) => pintln!("Error parsing mapping: "[e]),
        }
    }
    Ok(())
}
