use sdl2::image::LoadSurface;
use sdl2::render::Texture;
use sdl2::render::TextureCreator;
use sdl2::surface::Surface;
use sdl2::video::WindowContext;

use std::path::Path;

pub struct SpriteSheet<'a> {
  pub texture: Texture<'a>,
}

impl<'a> SpriteSheet<'a> {
  pub fn from_file<'txc, P: AsRef<Path>>(
    path: P,
    texture_creator: &'txc TextureCreator<WindowContext>
  ) -> Result<SpriteSheet<'txc>, String> {
    let texture =
      Surface::from_file(path)?
        .as_texture(texture_creator)
        .map_err(|e| format!("{:?}", e))?;

    Ok(SpriteSheet {
      texture,
    })
  }
}