use std::ops::Add;

use crate::dimensions::{WorldCoord as Coord, WorldPoint as Point, WorldRect as Rect};

pub const TILE_WIDTH: u32 = 64;
pub const TILE_WIDTH_F32: f32 = 64.;

pub struct Map {
  // Width and height are measured in grid units.
  pub width: u32,
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
  // Reads a Map from a file.
  //
  // The format is like so:
  // WIDTH
  // HEIGHT
  // TILES
  //
  // TILES is a grid of WIDTH by HEIGHT tile entries. Each entry is either X for
  // a wall or O for open.
  pub fn from_file(path: &str) -> Result<Map, String> {
    let file = std::fs::read_to_string(path).map_err(|e| format!("err reading file: {:?}", e))?;
    let mut lines = file.lines();

    let width: u32 = lines
      .next()
      .ok_or("map missing WIDTH")?
      .parse()
      .or(Err("failed to parse WIDTH"))?;
    let height: u32 = lines
      .next()
      .ok_or("map missing HEIGHT")?
      .parse()
      .or(Err("failed to parse WIDTH"))?;
    let mut grid_tiles = Vec::new();
    grid_tiles.reserve(width as usize * height as usize);
    for i in 0..height {
      let row = lines.next().ok_or(format!(
        "map ends at row {} of expected HEIGHT={}",
        i, height
      ))?;
      if row.len() == width as usize {
        for c in row.chars() {
          match c {
            'X' => grid_tiles.push(GridTile::Obstacle),
            'O' => grid_tiles.push(GridTile::Empty),
            _ => {}
          }
        }
      } else {
        return Err(format!(
          "row {} has length {}, when it should be WIDTH={}",
          i,
          row.len(),
          width
        ));
      }
    }

    Ok(Map {
      width,
      height,
      grid_tiles,
    })
  }

  pub fn get_tile(&self, p: TilePoint) -> Option<GridTile> {
    let TilePoint { x, y } = p;
    if self.width <= x || self.height <= y {
      return None;
    }
    let index = (x + y * self.width) as usize;
    if self.grid_tiles.len() <= index {
      return None;
    }
    Some(self.grid_tiles[index])
  }

  fn get_tile_unchecked(&self, x: u32, y: u32) -> GridTile {
    self.grid_tiles[(x + y * self.width) as usize]
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

    let top_left = rect.top_left.clamp(&bounds);
    let bot_right = rect.top_left + Point::new(rect.width, rect.height);
    let (top_left_x, top_left_y) = self.tile_coords_at_unchecked(top_left.clamp(&bounds));
    let (bot_right_x, bot_right_y) = self.tile_coords_at_unchecked(bot_right.clamp(&bounds));
    let width = bot_right_x - top_left_x + 1; // +1 to include the cur.
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
      width: Coord((self.width * TILE_WIDTH) as f32),
      height: Coord((self.height * TILE_WIDTH) as f32),
    }
  }

  // Returns (x, y), a tuple with the coordinates of the tile at this point.
  // May return None if the point is out of bounds.
  fn tile_coords_at(&self, point: Point) -> Option<(u32, u32)> {
    let (Coord(px), Coord(py)) = (point.x, point.y);
    if px < 0. || py < 0. {
      return None;
    }
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
    self
      .tile_coords_at(point)
      .map(|(x, y)| self.get_tile(TilePoint { x, y }))
      .flatten()
  }
}

pub struct MapTileIterator<'a> {
  x: u32,
  y: u32,
  map: &'a Map,
}

pub struct MapTileIteratorItem {
  pub pos: TilePoint,
  pub tile: GridTile,
}

impl Iterator for MapTileIterator<'_> {
  type Item = MapTileIteratorItem;

  fn next(&mut self) -> Option<MapTileIteratorItem> {
    if self.y >= self.map.height {
      return None;
    }
    let out = MapTileIteratorItem {
      pos: TilePoint {
        x: self.x,
        y: self.y,
      },
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
    if self.next_y >= self.top_left_y + self.height {
      return None;
    }
    let tile = MapTileIteratorItem {
      pos: TilePoint {
        x: self.next_x,
        y: self.next_y,
      },
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

#[derive(Eq, PartialEq, Hash, Clone, Copy)]
pub struct TilePoint {
  x: u32,
  y: u32,
}

impl TilePoint {
  pub fn new(x: u32, y: u32) -> TilePoint {
    TilePoint { x, y }
  }

  // TODO: Optimize to be a custom iterator. That way no malloc needed.
  pub fn neighbors4(&self, map: &Map) -> Vec<TilePoint> {
    let (x, y) = (self.x, self.y);
    let mut out = vec![];
    out.reserve_exact(4);
    if x > 0 {
      out.push(TilePoint { x: x - 1, y });
    }
    if y > 0 {
      out.push(TilePoint { x, y: y - 1 });
    }
    if x + 1 < map.width {
      out.push(TilePoint { x: x + 1, y });
    }
    if y + 1 < map.height {
      out.push(TilePoint { x, y: y + 1 });
    }
    out
  }

  pub fn tile_center(self) -> Point {
    Point {
      x: Coord((self.x as f32 + 0.5) * TILE_WIDTH_F32),
      y: Coord((self.y as f32 + 0.5) * TILE_WIDTH_F32),
    }
  }

  // Converts tile coordinates to world coordinates.
  pub fn to_world_point(self) -> Point {
    Point {
      x: Coord((self.x * TILE_WIDTH) as f32),
      y: Coord((self.y * TILE_WIDTH) as f32),
    }
  }

  pub fn center_to_world_point(self) -> Point {
    Point {
      x: Coord((self.x as f32 + 0.5) * TILE_WIDTH_F32),
      y: Coord((self.y as f32 + 0.5) * TILE_WIDTH_F32),
    }
  }
}

pub trait ToTilePoint {
  fn to_tile_point(self) -> TilePoint;
}

impl ToTilePoint for Point {
  fn to_tile_point(self) -> TilePoint {
    // TODO: Add checks to this and other conversions to ensure
    // the f32 is inside the range of allowable u32s.
    TilePoint {
      x: self.x.0 as u32 / TILE_WIDTH,
      y: self.y.0 as u32 / TILE_WIDTH,
    }
  }
}

impl Add for TilePoint {
  type Output = Self;
  fn add(self, other: Self) -> Self {
    TilePoint {
      x: self.x + other.x,
      y: self.y + other.y,
    }
  }
}
