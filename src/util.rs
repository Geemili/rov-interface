
use sdl2_ttf::Font;
use sdl2::pixels::Color;
use sdl2::render::Renderer;

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
