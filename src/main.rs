
#![recursion_limit = "1024"]

extern crate serial;
extern crate sdl2;
extern crate sdl2_ttf;
#[macro_use]
extern crate fomat_macros;
#[macro_use]
extern crate error_chain;

mod errors;

use std::path::Path;
use sdl2::pixels::Color;
use errors::*;

fn main() {
    use std;
    if let Some(gamepad) = std::env::args().skip(2).next() {
        let sdl_context = sdl2::init().unwrap();
        let joystick_subsystem = sdl_context.joystick().unwrap();
        let video = sdl_context.video().unwrap();
        let ttf_context = sdl2_ttf::init().unwrap();

        let window =
            video.window("ROV Interface", 800, 600).position_centered().opengl().build().unwrap();

        let mut renderer = window.renderer().accelerated().build().unwrap();

        let mut font =
            ttf_context.load_font(Path::new("assets/fonts/NotoSans/NotoSans-Regular.ttf"), 64)
                .unwrap();

        let available = match joystick_subsystem.num_joysticks() {
            Ok(n) => n,
            Err(e) => panic!("Can't enumerate joysticks. :( "),
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

        let mut surface =
            font.render(&fomat!("Axis 0: "(axis[0]))).solid(Color::RGB(255, 255, 255)).unwrap();
        let mut texture = renderer.create_texture_from_surface(&surface).unwrap();

        for event in sdl_context.event_pump().unwrap().wait_iter() {
            use sdl2::event::Event;

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
                Event::Quit { .. } => break,
                _ => (),
            }

            renderer.set_draw_color(Color::RGB(255, 128, 128));
            renderer.clear();

            renderer.set_draw_color(Color::RGB(255, 255, 255));
            surface = font.render(&format!("Axis 0: {}", axis[0]))
                .solid(Color::RGB(255, 255, 255))
                .unwrap();
            texture = renderer.create_texture_from_surface(&surface).unwrap();
            let mut dest = surface.rect();
            dest.set_y(0);
            renderer.copy(&texture, None, Some(dest)).unwrap();

            surface = font.render(&format!("Axis 1: {}", axis[1]))
                .solid(Color::RGB(255, 255, 255))
                .unwrap();
            texture = renderer.create_texture_from_surface(&surface).unwrap();
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
    }
    if let Some(port) = std::env::args().skip(1).next() {
        pintln!("Writing to port "(port));
        run(String::from(port.trim())).expect("Error running");
    }
}

fn run(port_name: String) -> Result<()> {
    use std::io::{Write, Read};
    use std::thread;
    use std::time::Duration;
    use serial::{SerialPort, SerialPortSettings};

    // Open port
    let mut port = serial::open(&port_name).chain_err(|| "Couldn't open file")?;

    let mut settings = serial::PortSettings::default();
    settings.set_char_size(serial::CharSize::Bits8);
    settings.set_parity(serial::Parity::ParityNone);
    settings.set_stop_bits(serial::StopBits::Stop1);
    settings.set_baud_rate(serial::BaudRate::Baud9600).chain_err(|| "Couldn't set baud rate")?;
    port.configure(&settings).chain_err(|| "Couldn't configure port")?;

    port.set_timeout(Duration::from_millis(5000)).chain_err(|| "Couldn't set duration")?;

    // Wait for a few milliseconds
    thread::sleep(Duration::from_millis(1000));

    let mut buffer = [0u8; 256];

    loop {
        // Write bytes
        write_message(&mut port, &[0xF0, 'h' as u8]);
        // 0000_0001 << 4 = 0001_0000
        // 0000_0111
        // 0b0001 == 1
        // 0b0010 == 2
        write_message(&mut port, &[0x10, 0b0001_0111, 24, 25]);
        write_message(&mut port, &[0x11, 0b0001_0001, 100]);

        // Read bytes
        let bytes_read = port.read(&mut buffer).expect("Couldn't read");
        for i in 0..bytes_read {
            pint!((buffer[i] as char)" ");
        }
        pintln!("");
    }

    Ok(())
}

fn write_message<S>(port: &mut S, message: &[u8])
    where S: serial::SerialPort
{
    let parity = message.iter().skip(1).fold(message[0], |acc, i| acc ^ i);
    port.write(message).expect("Couldn't write message");
    port.write(&[parity]).expect("Couldn't write parity");
}
