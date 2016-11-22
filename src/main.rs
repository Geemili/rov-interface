
#![recursion_limit = "1024"]
#![feature(receiver_try_iter)]

extern crate serial;
extern crate sdl2;
extern crate sdl2_ttf;
#[macro_use]
extern crate fomat_macros;
#[macro_use]
extern crate error_chain;

mod errors;
mod rov;

use std::path::Path;
use sdl2::pixels::Color;

fn main() {
    use std;
    let port_name = if let Some(port) = std::env::args().skip(1).next() {
        pintln!("Writing to port "(port));
        String::from(port.trim())
    } else {
        panic!("Port name is required");
    };

    let mut rov = rov::Rov::new(port_name.into());

    let sdl_context = sdl2::init().unwrap();
    let joystick_subsystem = sdl_context.joystick().unwrap();
    let video = sdl_context.video().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let ttf_context = sdl2_ttf::init().unwrap();

    let window =
        video.window("ROV Interface", 800, 600).position_centered().opengl().build().unwrap();

    let mut renderer = window.renderer().accelerated().build().unwrap();

    let font = ttf_context.load_font(Path::new("assets/fonts/NotoSans/NotoSans-Regular.ttf"), 64)
        .unwrap();

    let available = match joystick_subsystem.num_joysticks() {
        Ok(n) => n,
        Err(e) => panic!("Can't enumerate joysticks. :( {:?}", e),
    };

    pintln!((available)" joysticks available");

    let mut joystick = None;

    // Iterate over all available joysticks and stop once we manage to
    // open one.
    for id in 0..available {
        match joystick_subsystem.open(id) {
            Ok(c) => {
                pintln!("Success: opened \""(c.name())"\"");
                joystick = Some(c);
                break;
            }
            Err(e) => pintln!("failed: "[e]),
        }
    }

    if joystick.is_none() {
        panic!("Couldn't open any joystick");
    };

    let mut axis = [0; 2];
    let mut buttons = [false; 9];

    'main: loop {
        for event in event_pump.poll_iter() {
            use sdl2::event::Event;
            use sdl2::keyboard::Keycode;

            match event {
                Event::JoyAxisMotion { axis_idx, value: val, .. } => {
                    // Axis motion is an absolute value in the range
                    // [-32768, 32767]. Let's simulate a very rough dead
                    // zone to ignore spurious events.
                    let dead_zone = 10000;
                    axis[axis_idx as usize] = if val > dead_zone || val < -dead_zone {
                        val
                    } else {
                        0
                    }
                }
                Event::JoyButtonDown { button_idx, .. } => buttons[button_idx as usize] = true,
                Event::JoyButtonUp { button_idx, .. } => buttons[button_idx as usize] = false,
                Event::JoyHatMotion { hat_idx, state, .. } => {
                    pintln!("Hat "(hat_idx)" moved to "[state])
                }
                Event::Quit { .. } |
                Event::KeyUp { keycode: Some(Keycode::Escape), .. } => break 'main,
                _ => (),
            }
        }

        for response in rov.responses().iter() {
            pintln!([response]);
        }

        renderer.set_draw_color(Color::RGB(255, 128, 128));
        renderer.clear();

        renderer.set_draw_color(Color::RGB(255, 255, 255));
        let surface = font.render(&format!("Axis 0: {}", axis[0]))
            .solid(Color::RGB(255, 255, 255))
            .unwrap();
        let texture = renderer.create_texture_from_surface(&surface).unwrap();
        let mut dest = surface.rect();
        dest.set_y(0);
        renderer.copy(&texture, None, Some(dest)).unwrap();

        let surface = font.render(&format!("Axis 1: {}", axis[1]))
            .solid(Color::RGB(255, 255, 255))
            .unwrap();
        let texture = renderer.create_texture_from_surface(&surface).unwrap();
        let mut dest = surface.rect();
        dest.set_y(64);
        renderer.copy(&texture, None, Some(dest)).unwrap();

        let button_width = 800 / buttons.len();
        for i in 0..buttons.len() {
            let rect = (i as i32 * button_width as i32, 150, button_width as u32, 64).into();
            if buttons[i as usize] {
                renderer.fill_rect(rect).unwrap()
            } else {
                renderer.draw_rect(rect).unwrap()
            }
        }

        renderer.present();
    }

    rov.quit().unwrap();
}
