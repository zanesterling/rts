use crate::sprite_sheet::SpriteKey;
use crate::units::{
  WorldCoord as Coord,
  WorldPoint as Point,
};

pub const TILE_WIDTH: u32 = 64;

pub struct State {
  pub units: Vec<Unit>,
  pub map: Map,
}

impl State {
  pub fn new() -> State {
    let o = GridTile::Empty;
    let l = GridTile::Obstacle;
    let newt = "newt_gingrich".to_string();
    State {
      units: vec![
        Unit::new(Coord(300.), Coord(200.), Coord(16.), newt.clone()),
        Unit::new(Coord(350.), Coord(300.), Coord(16.), newt.clone()),
        Unit::new(Coord(450.), Coord(230.), Coord(16.), newt.clone()),
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
      // If the unit is moving, move it.
      if let Some(target) = &unit.move_target {
        let to_target = *target - unit.pos;
        let speed = unit.speed();
        let (next_pos, is_last_step) = if to_target.magnitude() < speed {
          (*target, true)
        } else {
          (unit.pos + to_target.normalized()*speed, false)
        };

        if let Some(GridTile::Empty) = self.map.get_tile_at(next_pos) {
          unit.pos = next_pos;
        } else {
          // TODO: Step up to the wall, but not through it.
        }
        if is_last_step { unit.move_target = None; }
      }
    }
  }
}

pub struct Unit {
  pub pos: Point,
  pub rad: Coord,
  pub selected: bool,
  pub move_target: Option<Point>,
  pub base_speed: Coord,
  pub sprite_key: SpriteKey,
}

impl Unit {
  pub fn new(x: Coord, y: Coord, rad: Coord, sprite_key: SpriteKey) -> Unit {
    Unit {
      pos: Point::new(x, y),
      rad,
      selected: false,
      move_target: None,
      base_speed: Coord(1.),
      sprite_key,
    }
  }

  pub fn speed(&self) -> Coord { self.base_speed }
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

  pub fn get_tile_at(&self, point: Point) -> Option<GridTile> {
    let (Coord(px), Coord(py)) = (point.x, point.y);
    if px < 0. || py < 0. { return None }
    let x = px as u32 / TILE_WIDTH;
    let y = py as u32 / TILE_WIDTH;
    self.get_tile(x, y)
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

// Converts tile coordinates to world coordinates.
pub fn tile_pos(x: u32, y: u32) -> Point {
  Point {
    x: Coord((x * TILE_WIDTH) as f32),
    y: Coord((y * TILE_WIDTH) as f32),
  }
}