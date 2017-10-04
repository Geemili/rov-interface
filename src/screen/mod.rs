
pub mod control_rov;
pub mod port_select;

use ::errors::*;

pub enum Trans {
    Quit,
    None,
    Switch(Box<Screen>),
}

pub trait Screen {
    fn init(&mut self, engine: &mut Engine) -> Result<()>;
    fn update(&mut self, engine: &mut Engine, delta: f64) -> Result<Trans>;
    fn render(&mut self, engine: &mut Engine, delta: f64) -> Result<()>;
}

use sdl2::EventPump;
use sdl2::render::{Renderer, Texture};
use rusttype::{Font, PositionedGlyph};
use rusttype::gpu_cache::Cache;
use gilrs;
use config::Config;

pub struct Engine<'renderer> {
    pub event_pump: EventPump,
    pub controllers: gilrs::Gilrs,
    pub renderer: Renderer<'renderer>,
    pub rfont: Font<'renderer>,
    pub cache: Cache,
    pub glyphs: Vec<PositionedGlyph<'renderer>>,
    pub cache_texture: Texture,
    pub config: Config,
}

        use rusttype::Scale;
impl<'a> Engine<'a> {

    pub fn queue_text(&mut self, x: f32, y: f32, scale: Scale, text: &str) {
        use unicode_normalization::UnicodeNormalization;
        use rusttype::Point;

        let y = y + scale.y;
        let mut caret = Point {x, y};
        for c in text.nfc() {
            let base_glyph = if let Some(glyph) = self.rfont.glyph(c) {
                glyph
            } else {
                continue;
            };
            let glyph = base_glyph.scaled(scale).positioned(caret);
            caret.x += glyph.unpositioned().h_metrics().advance_width;
            self.glyphs.push(glyph.standalone());
        }
    }

    pub fn render_text(&mut self) {
        use sdl2::rect::Rect as SdlRect;

        for glyph in &self.glyphs {
            self.cache.queue_glyph(0, glyph.clone());
        }
        let mut to_cache = vec![];
        self.cache.cache_queued(|rect,data| {
                          let rect = SdlRect::new(rect.min.x as i32,
                                                 rect.min.y as i32,
                                                 rect.width(),
                                                 rect.height());
                          let mut pixel_data = vec![];
                          // Assuming the cache texture is in RGBA8888
                          for p in data {
                              let fill = 0xFF;
                              pixel_data.push(*p);
                              pixel_data.push(fill);
                              pixel_data.push(fill);
                              pixel_data.push(fill);
                          }
                          to_cache.push((rect, pixel_data));
        }).expect("render_text queue character");

        // This for loop is used to avoid double borrowing in the closure
        for (rect, pixel_data) in to_cache {
            self.cache_texture.update(
                Some(rect),
                &pixel_data,
                rect.width() as usize * 4)
                .expect("Error updating font cache");
        }

        let (cache_width, cache_height) = self.cache.dimensions();
        let (cache_width, cache_height) = (cache_width as f32, cache_height as f32);
        for glyph in &self.glyphs {
            let cache_result = self.cache.rect_for(0, glyph)
                .expect("Glyph not in cache");
            if let Some((src, dest)) = cache_result {
                let cache_rect = SdlRect::new(
                    (src.min.x * cache_width) as i32,
                    (src.min.y * cache_height) as i32,
                    (src.width() * cache_width) as u32,
                    (src.height() * cache_height) as u32);
                let dest_rect = SdlRect::new(
                    dest.min.x as i32,
                    dest.min.y as i32,
                    dest.width() as u32,
                    dest.height() as u32);
                self.renderer.copy(
                    &self.cache_texture,
                    Some(cache_rect),
                    Some(dest_rect))
                    .expect("Error rendering glyph to screen");
            }
        }

        self.glyphs.clear();
    }
}
