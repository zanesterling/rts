use crate::sprite_sheet::SpriteKey;
use crate::units::WorldPoint;

pub const TILE_WIDTH: u32 = 64;

pub struct State {
  pub units: Vec<Unit>,
  pub map: Map,
}

impl State {
  pub fn new() -> State {
    let o = GridTile::Empty;
    let l = GridTile::Obstacle;
    State {
      units: vec![
        Unit::new(300., 200., "newt_gingrich".to_string()),
        Unit::new(350., 300., "newt_gingrich".to_string()),
        Unit::new(450., 230., "newt_gingrich".to_string()),
      ],

      map: Map {
        width:  10,
        height: 10,
        grid_tiles: vec![
          o,o,o,o,o,o,o,o,o,o,
          o,o,o,o,o,o,o,o,o,o,
          o,o,l,l,l,l,l,l,o,o,
          o,o,o,o,o,o,o,l,o,o,
          o,o,o,o,o,l,o,l,o,o,
          o,o,l,l,l,l,o,l,o,o,
          o,o,l,o,o,o,o,l,o,o,
          o,o,l,l,l,l,l,l,o,o,
          o,o,o,o,o,o,o,o,o,o,
          o,o,o,o,o,o,o,o,o,o,
        ],
      },
    }
  }

  pub fn tick(&mut self) {
    for unit in self.units.iter_mut() {
      if let Some(target) = &unit.move_target {
        let to_target = target.minus(unit.pos);
        let speed = unit.speed();
        if to_target.magnitude() < speed {
          unit.pos = *target;
          unit.move_target = None;
        } else {
          let vel = to_target.normalized().scaled(unit.speed());
          unit.pos = unit.pos.plus(vel);
        }
      }
    }
  }
}

pub struct Unit {
  pub pos: WorldPoint,
  pub selected: bool,
  pub move_target: Option<WorldPoint>,
  pub base_speed: f32,
  pub sprite_key: SpriteKey,
}

impl Unit {
  pub fn new(x: f32, y: f32, sprite_key: SpriteKey) -> Unit {
    Unit {
      pos: WorldPoint::new(x, y),
      selected: false,
      move_target: None,
      base_speed: 1.,
      sprite_key,
    }
  }

  pub fn speed(&self) -> f32 { self.base_speed }
}

pub struct Map {
  // Width and height are measured in grid units.
  pub width:  u32,
  pub height: u32,

  pub grid_tiles: Vec<GridTile>,
}

impl Map {
  pub fn get_tile(&self, x: u32, y: u32) -> Option<GridTile> {
    if self.width <= x || self.height <= y { return None }
    let index = (x + y*self.width) as usize;
    if self.grid_tiles.len() <= index { return None }
    Some(self.grid_tiles[index])
  }

  fn get_tile_unchecked(&self, x: u32, y: u32) -> GridTile {
    self.grid_tiles[(x + y*self.width) as usize]
  }

  pub fn tiles<'a>(&'a self) -> MapTileIterator<'a> {
    MapTileIterator {
      x: 0,
      y: 0,
      map: self,
    }
  }
}

pub struct MapTileIterator<'a> {
  x: u32,
  y: u32,
  map: &'a Map,
}

pub struct MapTileIteratorItem {
  pub x: u32,
  pub y: u32,
  pub tile: GridTile,
}

impl Iterator for MapTileIterator<'_> {
  type Item = MapTileIteratorItem;

  fn next(&mut self) -> Option<MapTileIteratorItem> {
    if self.y >= self.map.height { return None }
    let out = MapTileIteratorItem {
      x: self.x,
      y: self.y,
      tile: self.map.get_tile_unchecked(self.x, self.y),
    };
    if self.x >= self.map.width - 1 {
      self.x = 0;
      self.y += 1;
    } else {
      self.x += 1;
    }
    Some(out)
  }
}

#[derive(Clone, Copy)]
pub enum GridTile {
  Empty,
  Obstacle,
}
