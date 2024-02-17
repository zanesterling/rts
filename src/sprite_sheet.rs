use sdl2::image::LoadSurface;
use sdl2::render::Texture;
use sdl2::render::TextureCreator;
use sdl2::surface::Surface;
use sdl2::video::WindowContext;

use std::path::Path;
use std::str::FromStr;

pub struct SpriteSheet<'a> {
  pub texture: Texture<'a>,
  pub sprite_map: Vec<SpriteRef>,
}

impl<'a> SpriteSheet<'a> {
  pub fn from_file<'txc, P: AsRef<Path>>(
    sprite_map_path: P,
    texture_creator: &'txc TextureCreator<WindowContext>
  ) -> Result<SpriteSheet<'txc>, String> {
    let mut sprite_map = vec![];
    let file = std::fs::read_to_string(sprite_map_path)
      .map_err(|e| format!("err reading file: {:?}", e))?;
    let mut lines = file.lines();

    // Read the image_path from the map file, then load it and create a texture.
    let image_path = lines.next().ok_or("sprite sheet missing img path")?;
    let texture =
      // TODO: handle directory-local scoping of paths
      Surface::from_file(Path::new("media").join(image_path))?
        .as_texture(texture_creator)
        .map_err(|e| format!("err making texture: {:?}", e))?;

    // Read the sprite map from the map file.
    let n_sprites = u32::from_str_radix(
      lines.next().ok_or("sprite sheet missing n_sprites")?, 10)
      .map_err(|e| format!("err parsing n_sprites: {:?}", e))?;
    for _ in 0..n_sprites {
      let line = lines.next().ok_or("sprite sheet has too few sprites")?;
      sprite_map.push(line.parse()?);
    }

    Ok(SpriteSheet {
      texture,
      sprite_map,
    })
  }
}

pub struct SpriteRef {
  pub name:     String, // Must not have spaces.
  pub offset_x: u32,
  pub offset_y: u32,
  pub width:    u32,
  pub height:   u32,
}

impl FromStr for SpriteRef {
  type Err = String;

  // Parses a string of the form "NAME OFF_X OFF_Y WIDTH HEIGHT"
  // into an instance of FromStr. NAME must not have spaces.
  fn from_str(line: &str) -> Result<Self, Self::Err> {
    let elts: Vec<_> = line.split(' ').collect();
    if elts.len() != 5 {
      return Err("FromStr line has wrong number of elements".to_string());
    }
    Ok(SpriteRef {
      name:     elts[0].to_string(),
      offset_x: u32::from_str(elts[1]).map_err(|e| format!("{:?}", e))?,
      offset_y: u32::from_str(elts[2]).map_err(|e| format!("{:?}", e))?,
      width:    u32::from_str(elts[3]).map_err(|e| format!("{:?}", e))?,
      height:   u32::from_str(elts[4]).map_err(|e| format!("{:?}", e))?,
    })
  }
}