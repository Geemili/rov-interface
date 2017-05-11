
use sdl2_ttf::Font;
use sdl2::pixels::Color;
use sdl2::render::Renderer;
use sdl2::rect::Rect;
use ::errors::*;

pub fn draw_text(renderer: &mut Renderer, font: &Font, text: &str, pos: [i32; 2]) {
    let surface = font.render(text)
        .solid(Color::RGB(255, 255, 255))
        .unwrap();
    let texture = renderer.create_texture_from_surface(&surface).unwrap();
    let mut dest = surface.rect();
    dest.set_x(pos[0]);
    dest.set_y(pos[1]);
    renderer.copy(&texture, None, Some(dest)).unwrap();
}

pub fn draw_text_ext(renderer: &mut Renderer, font: &Font, text: &str, dest: Rect) {
    let surface = font.render(text)
        .solid(Color::RGB(255, 255, 255))
        .unwrap();
    let texture = renderer.create_texture_from_surface(&surface).unwrap();
    renderer.copy(&texture, None, Some(dest)).unwrap();
}

use config::Config;
pub fn load_config_from_file(path: &str) -> Result<Config> {
    use std::fs::File;
    use std::io::Read;

    let mut file = File::open(path).chain_err(|| "Failed to open config file")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).chain_err(|| "Failed to read file")?;

    let config = ::toml::de::from_str(&contents).chain_err(|| "Failed to deserialize config")?;

    Ok(config)
}
