use sdl2::keyboard::Keycode;

use std::collections::HashMap;
use std::collections::VecDeque;

use crate::dimensions::{
  WorldCoord as Coord,
  WorldPoint as Point,
  WorldRect as Rect,
};
use crate::sprite_sheet::SpriteKey;
use crate::map::{Map, GridTile, TilePoint, ToTilePoint};

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
        Unit::new(Point::new(Coord(300.), Coord(250.)),
          Coord(16.), newt.clone()),
      ],

      map: Map {
        width:  14,
        height: 10,
        grid_tiles: vec![
          o,o,o,o,o,o,o,o,o,o,o,o,o,o,
          o,o,o,o,o,o,o,o,o,o,o,o,o,o,
          o,o,o,o,o,l,l,l,l,l,l,o,o,o,
          o,o,o,o,o,o,o,o,o,o,l,o,o,o,
          o,o,o,o,o,o,o,o,l,o,l,o,o,o,
          o,o,o,o,o,l,l,l,l,o,l,o,o,o,
          o,o,o,o,o,l,o,o,o,o,l,o,o,o,
          o,o,o,o,o,l,l,l,l,l,l,o,o,o,
          o,o,o,o,o,o,o,o,o,o,o,o,o,o,
          o,o,o,o,o,o,o,o,o,o,o,o,o,o,
        ],
      },
    }
  }

  pub fn tick(&mut self) {
    for unit in self.units.iter_mut() {
      // If the unit is moving, move it.
      if let Some(target) = unit.waypoints.front() {
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
        if is_last_step { unit.waypoints.pop_front(); }
      }
    }
  }
}

pub struct Unit {
  pub pos: Point,
  pub rad: Coord,
  pub selected: bool,
  pub waypoints: VecDeque<Point>,
  pub base_speed: Coord,
  pub sprite_key: SpriteKey,
  pub abilities: Vec<Box<dyn Ability>>,
}

impl Unit {
  pub fn new(pos: Point, rad: Coord, sprite_key: SpriteKey) -> Unit {
    Unit {
      pos,
      rad,
      selected: false,
      waypoints: VecDeque::new(),
      base_speed: Coord(1.),
      sprite_key,
      abilities: vec![],
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

  pub fn queue_move(&mut self, p: Point) {
    self.waypoints.push_back(p);
  }

  pub fn move_queued(&self) -> bool {
    self.waypoints.len() > 0
  }

  // Find a path to dest, and enqueue that path in the waypoints. Returns true
  // if a path was found, and false otherwise.
  //
  // As a first pass, this is implemented as BFS.
  // TODO: Implement A*.
  pub fn pathfind(&mut self, map: &Map, dest: Point) -> bool {
    let src = if self.waypoints.is_empty()
      { self.pos }
      else { self.waypoints[self.waypoints.len() - 1] };
    let src = src.to_tile_point();
    let dest = dest.to_tile_point();

    #[derive(Clone, Copy)]
    struct BackPath {
      here: TilePoint,
      best_source: TilePoint,
      path_cost: u32,
    }

    // Find a path. (incidentally, finds the best path)
    let mut visited: HashMap<TilePoint, BackPath> = HashMap::new();
    let mut to_visit: VecDeque<BackPath> = VecDeque::new();
    to_visit.push_front(BackPath {
      here: src,
      best_source: src, // Just need to put some value here.
      path_cost: 0
    });
    while !to_visit.is_empty() {
      let point = to_visit.pop_front().unwrap();
      if visited.contains_key(&point.here) { continue }
      visited.insert(point.here, point);
      if point.here == dest { break }
      for p in point.here.neighbors4(&map) {
        let tile_blocked = map.get_tile(p)
          .map(|t| t != GridTile::Empty)
          .unwrap_or(true);
        if tile_blocked { continue }
        to_visit.push_back(BackPath {
          here: p,
          best_source: point.here,
          path_cost: point.path_cost + 1,
        });
      }
    }
    if !visited.contains_key(&dest) { return false; }

    // Make waypoints for the path found.
    let mut path_reverse = vec![];
    let mut current = visited.get(&dest).unwrap();
    while current.here != src {
      path_reverse.push(current.here);
      current = visited.get(&current.best_source).unwrap();
    }
    self.waypoints.push_back(src.tile_center());
    for p in path_reverse.iter().rev() {
      self.waypoints.push_back(p.tile_center());
    }
    true
  }

  pub fn window_rad(&self) -> u32 {
    self.rad.0 as u32
  }
}

pub trait Ability {
  fn keycode(&self) -> Keycode;
  fn cast(&self, state: &mut State, target: Point);
}

pub struct AbilityBuild {}

impl Ability for AbilityBuild {
  fn keycode(&self) -> Keycode { Keycode::B }

  fn cast(&self, state: &mut State, target: Point) {
    state.units.push(Unit::new(
      target, Coord(16.), "newt_gingrich".to_string()));
  }
}
