use sdl2::image::LoadSurface;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::render::RenderTarget;
use sdl2::render::Texture;
use sdl2::render::TextureCreator;
use sdl2::surface::Surface;
use sdl2::video::WindowContext;

use std::path::Path;
use std::str::FromStr;

pub struct SpriteSheet<'texture> {
  pub texture: Texture<'texture>,
  pub sprite_map: Vec<SpriteRef>,
}

impl<'texture> SpriteSheet<'texture> {
  pub fn from_file<'txc>(
    sprite_map_path: &str,
    texture_creator: &'txc TextureCreator<WindowContext>,
  ) -> Result<SpriteSheet<'txc>, String> {
    let mut sprite_map = vec![];
    let file =
      std::fs::read_to_string(sprite_map_path).map_err(|e| format!("err reading file: {:?}", e))?;
    let mut lines = file.lines();

    // Read the image_path from the map file, then load it and create a texture.
    let image_path = lines.next().ok_or("sprite sheet missing img path")?;
    let parent_dir = Path::new(&sprite_map_path).parent().ok_or(format!(
      "sprite sheet file has no parent dir, so could not open the png file"
    ))?;
    let texture = Surface::from_file(parent_dir.join(image_path))?
      .as_texture(texture_creator)
      .map_err(|e| format!("err making texture: {:?}", e))?;

    // Read the sprite map from the map file.
    let n_sprites = u32::from_str_radix(lines.next().ok_or("sprite sheet missing n_sprites")?, 10)
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

  pub fn blit_sprite_to_rect<Ctx: RenderTarget>(
    &self,
    sprite_id: &str,
    canvas: &mut Canvas<Ctx>,
    dst_rect: Rect,
  ) -> Result<(), String> {
    for sprite_ref in self.sprite_map.iter() {
      if sprite_ref.name == sprite_id {
        let src_rect = sprite_ref.rect();
        canvas.copy(&self.texture, src_rect, dst_rect)?;
        return Ok(());
      }
    }
    Err(format!("sprite \"{}\" not found", sprite_id))
  }

  pub fn blit_sprite<Ctx: RenderTarget>(
    &self,
    sprite_id: &str,
    canvas: &mut Canvas<Ctx>,
    x_off: u32,
    y_off: u32,
    mag_factor: u32,
  ) -> Result<Rect, String> {
    for sprite_ref in self.sprite_map.iter() {
      if sprite_ref.name == sprite_id {
        let src_rect = sprite_ref.rect();
        let dst_rect = Rect::new(
          x_off as i32,
          y_off as i32,
          mag_factor * src_rect.width(),
          mag_factor * src_rect.height(),
        );
        canvas.copy(&self.texture, src_rect, dst_rect)?;
        return Ok(dst_rect);
      }
    }
    Err(format!("sprite \"{}\" not found", sprite_id))
  }
}

pub struct SpriteRef {
  pub name: SpriteKey,
  pub offset_x: u32,
  pub offset_y: u32,
  pub width: u32,
  pub height: u32,
}

pub type SpriteKey = String; // Must not have spaces.

impl SpriteRef {
  fn rect(&self) -> Rect {
    Rect::new(
      self.offset_x as i32,
      self.offset_y as i32,
      self.width,
      self.height,
    )
  }
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
      name: elts[0].to_string(),
      offset_x: u32::from_str(elts[1]).map_err(|e| format!("{:?}", e))?,
      offset_y: u32::from_str(elts[2]).map_err(|e| format!("{:?}", e))?,
      width: u32::from_str(elts[3]).map_err(|e| format!("{:?}", e))?,
      height: u32::from_str(elts[4]).map_err(|e| format!("{:?}", e))?,
    })
  }
}
