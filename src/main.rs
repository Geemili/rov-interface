
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
#[macro_use(o, kv, slog_b, slog_kv,
           slog_record, slog_record_static,
           slog_log, slog_info, slog_error, slog_trace, slog_warn)]
extern crate slog;
extern crate slog_term;
extern crate slog_async;
extern crate slog_json;
#[macro_use]
extern crate slog_scope;
extern crate rusttype;
extern crate unicode_normalization;

pub mod errors;
mod rov;
mod mock;
mod util;
mod screen;
mod control;
mod config;

use errors::*;
use rusttype::gpu_cache::Cache;

fn main() {
    use slog::Drain;

    let decorator = slog_term::TermDecorator::new().build();
    let term_drain = slog_term::FullFormat::new(decorator).build().fuse();
    let term_drain = slog_async::Async::new(term_drain).build().fuse();

    let log_file = ::std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open("log.json")
        .expect("Couldn't open log file!");

    let json_drain = ::std::sync::Mutex::new(slog_json::Json::default(log_file).fuse());

    let root_drain = slog::Duplicate(json_drain, term_drain).fuse();

    let root_logger = slog::Logger::root(root_drain, o!("version" => env!("CARGO_PKG_VERSION")));

    let _guard = slog_scope::set_global_logger(root_logger);

    info!("Application started"; "started_at" => format!("{}", time::now().rfc3339()));

    if let Err(ref e) = run() {
        let error_trace = util::get_error_trace(e);

        // If there is a backtrace, print it.
        let backtrace = format!("{:?}", e.backtrace());
        error!("An error was returned to main.";
              "error_trace" => error_trace,
              "backtrace" => backtrace);

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

    let rfont = load_font().chain_err(|| "Failed to load r font")?;

    let renderer =
        window.renderer().accelerated().build().chain_err(|| "Failed to accelerate renderer")?;

    let config = match util::load_config_from_file("config.toml") {
        Ok(config) => config,
        Err(ref e) => {
            let error_trace = util::get_error_trace(e);
            info!("Error loading config file, using default configuration."; "error_trace" => error_trace);
            ::config::Config::default()
        }
    };

    let (cache_width, cache_height) = (512, 512);
    let cache = Cache::new(cache_width, cache_height, 0.1, 0.1);

    use sdl2::pixels::PixelFormatEnum;
    const PIXEL_FORMAT: PixelFormatEnum = PixelFormatEnum::RGBA8888;

    let cache_texture = renderer.create_texture_target(PIXEL_FORMAT, cache_width, cache_height)
        .chain_err(|| "Failed to create texture for font cache")?;

    let mut engine = screen::Engine {
        event_pump: event_pump,
        controllers: gilrs,
        renderer: renderer,
        font: font,
        rfont: rfont,
        cache: cache,
        glyphs: vec![],
        cache_texture: cache_texture,
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

        let trans = screen.update(&mut engine, delta).chain_err(|| "Failed to update screen")?;

        use sdl2::pixels::Color;
        engine.renderer.set_draw_color(Color::RGB(0, 0, 0));
        engine.renderer.clear();
        engine.renderer.set_draw_color(Color::RGB(255, 255, 255));
        screen.render(&mut engine, delta).chain_err(|| "Failed to render screen")?;
        engine.render_text();
        engine.renderer.present();

        let current_screen = match trans {
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

use rusttype::{FontCollection, Font};
use std::fs::File;
use std::io::Read;

fn load_font<'a>() -> Result<Font<'a>> {
    let path = Path::new("assets/fonts/NotoSans/NotoSans-Regular.ttf");
    let mut file = File::open(path).chain_err(|| "Font file not found")?;
    let mut bytes = vec![];
    file.read_to_end(&mut bytes).chain_err(|| "Failed to read font file")?;
    let collection = FontCollection::from_bytes(bytes);
    collection.into_font()
        .ok_or("File contained no fonts!".into())
}
