use crate::sprite_sheet::SpriteKey;
use crate::dimensions::{
  WorldCoord as Coord,
  WorldPoint as Point,
  WorldRect as Rect,
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
        Unit::new(Coord(300.), Coord(250.), Coord(16.), newt.clone()),
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

        let tiles: Vec<_> = self.map.tiles_overlapping_rect(unit.bounding_box_at(next_pos)).collect();
        let collision = tiles.iter()
          .any(|item| item.tile == GridTile::Obstacle);
        if collision {
          // TODO: Step up to the wall, but not through it.
        } else {
          unit.pos = next_pos;
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

  fn bounding_box(&self) -> Rect { self.bounding_box_at(self.pos) }
  fn bounding_box_at(&self, p: Point) -> Rect {
    let top_left  = p - Point::new(self.rad, self.rad);
    Rect {
      top_left,
      width:  self.rad*Coord(2.),
      height: self.rad*Coord(2.),
    }
  }
}

pub struct Map {
  // Width and height are measured in grid units.
  pub width:  u32,
  pub height: u32,

  pub grid_tiles: Vec<GridTile>,
}

// A tile is square with side length L:
//
// p1--p2
// |    |
// p3--p4
//
// The tile at tile coordinates (0,0) has p1 = (0,0), p4 = (L-ε, L-ε).
//
// Yes okay fine, to you mathematicians out there: p2, p3, and p4 are not
// actually in the tile. The tile is left- and up- inclusive and right-
// and down- exclusive.
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

  pub fn tiles_overlapping_rect<'a>(&'a self, rect: Rect) -> MapTileRectIterator<'a> {
    let bounds = self.bounds();
    if !bounds.intersects(&rect) {
      return MapTileRectIterator::empty(&self);
    }

    let top_left  = rect.top_left.clamp(&bounds);
    let bot_right = rect.top_left + Point::new(rect.width, rect.height);
    let (top_left_x, top_left_y) =
      self.tile_coords_at_unchecked(top_left.clamp(&bounds));
    let (bot_right_x, bot_right_y) =
      self.tile_coords_at_unchecked(bot_right.clamp(&bounds));
    let width  = bot_right_x - top_left_x + 1; // +1 to include the cur.
    let height = bot_right_y - top_left_y + 1; // +1 to include the cur.

    MapTileRectIterator {
      next_x: top_left_x,
      next_y: top_left_y,

      top_left_x,
      top_left_y,
      width,
      height,
      map: &self,
    }
  }

  fn bounds(&self) -> Rect {
    Rect {
      top_left: Point::new(Coord(0.), Coord(0.)),
      width:  Coord((self.width  * TILE_WIDTH) as f32),
      height: Coord((self.height * TILE_WIDTH) as f32),
    }
  }

  // Returns (x, y), a tuple with the coordinates of the tile at this point.
  // May return None if the point is out of bounds.
  fn tile_coords_at(&self, point: Point) -> Option<(u32, u32)> {
    let (Coord(px), Coord(py)) = (point.x, point.y);
    if px < 0. || py < 0. { return None }
    let x = px as u32 / TILE_WIDTH;
    let y = py as u32 / TILE_WIDTH;
    Some((x, y))
  }

  fn tile_coords_at_unchecked(&self, point: Point) -> (u32, u32) {
    let (Coord(px), Coord(py)) = (point.x, point.y);
    let x = px as u32 / TILE_WIDTH;
    let y = py as u32 / TILE_WIDTH;
    (x, y)
  }

  pub fn get_tile_at(&self, point: Point) -> Option<GridTile> {
    self.tile_coords_at(point)
      .map(|(x, y)| self.get_tile(x, y))
      .flatten()
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

pub struct MapTileRectIterator<'a> {
  // x,y pointer to the next tile.
  next_x: u32,
  next_y: u32,

  // Tile coordinates of the top-left point of the rect
  // that we're iterating through.
  top_left_x: u32,
  top_left_y: u32,
  
  // Width and height of the rect.
  // This iterator outputs tiles in [top_left_x, top_left_x + width].
  // Corresponding for y axis.
  //
  // Struct creators must observe these constraints:
  width: u32,  // top_left_x + width  <= map.width
  height: u32, // top_left_y + height <= map.height
  map: &'a Map,
}

impl<'a> MapTileRectIterator<'a> {
  // An iterator which produces no tiles.
  pub fn empty(map: &'a Map) -> MapTileRectIterator<'a> {
    MapTileRectIterator {
      // If we set the target rect to be empty...
      top_left_x: 0,
      top_left_y: 0,
      width: 0,
      height: 0,

      // ...and the next point to be well outside the target,
      // then the iterator should immediately terminate.
      next_x: 10,
      next_y: 10,

      map,
    }
  }
}

impl Iterator for MapTileRectIterator<'_> {
  type Item = MapTileIteratorItem;

  fn next(&mut self) -> Option<MapTileIteratorItem> {
    if self.next_y >= self.top_left_y + self.height { return None }
    let tile = MapTileIteratorItem {
      x: self.next_x,
      y: self.next_y,
      tile: self.map.get_tile_unchecked(self.next_x, self.next_y),
    };

    self.next_x += 1;
    if self.next_x >= self.top_left_x + self.width {
      self.next_x = self.top_left_x;
      self.next_y += 1;
    }

    Some(tile)
  }
}

#[derive(Clone, Copy, PartialEq)]
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
