use std::collections::HashMap;
use std::collections::VecDeque;

use crate::ability::{Ability, AbilityBuild, AbilityTrain};
use crate::dimensions::{WorldCoord as Coord, WorldPoint as Point, WorldRect as Rect};
use crate::map::{GridTile, Map, TilePoint, ToTilePoint};
use crate::sprite_sheet::SpriteKey;

const TICKS_PER_SEC: u32 = 120; // TODO: Drop to 24 when fps and tps differ.

#[derive(Clone, Copy, PartialEq)]
pub struct GameDur {
  pub ticks: u32,
}
impl GameDur {
  pub fn from_secs(secs: u32) -> GameDur {
    GameDur {
      ticks: secs * TICKS_PER_SEC,
    }
  }
}

// UIDs are used to refer uniquely to buildings or units.
pub type UID = u32;

pub struct State {
  pub units: Vec<Unit>,
  pub unit_types: Vec<UnitType>,
  pub buildings: Vec<Building>,
  pub building_types: Vec<BuildingType>,
  pub map: Map,
  pub next_uid: UID,
}

impl State {
  pub fn blank() -> State {
    State {
      units: vec![],
      unit_types: vec![],
      buildings: vec![],
      building_types: vec![],

      map: Map::from_file("media/test-map.txt").expect("couldn't load the map"),

      next_uid: 0,
    }
  }

  pub fn level1() -> State {
    let mut state = State::blank();
    let newt_type = UnitType {
      name: "Newt",
      sprite_key: "newt_gingrich".to_string(),
      radius: Coord(16.),
      base_speed: Coord(1.),
    };
    state.unit_types.push(newt_type.clone());
    let town_hall_type = BuildingType {
      name: "Town Hall",
      width: 1,
      height: 1,
    };
    state.building_types.push(town_hall_type.clone());

    state.make_unit(newt_type, Point::new(Coord(300.), Coord(250.)));
    state.make_building(town_hall_type, TilePoint::new(1, 1));
    state
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
          (unit.pos + to_target.normalized() * speed, false)
        };

        let tiles: Vec<_> = self
          .map
          .tiles_overlapping_rect(unit.bounding_box_at(next_pos))
          .collect();
        let collision = tiles.iter().any(|item| item.tile == GridTile::Obstacle);
        if collision {
          // TODO: Step up to the wall, but not through it.
        } else {
          unit.pos = next_pos;
        }
        if is_last_step {
          unit.waypoints.pop_front();
        }
      }
    }

    let mut units_to_make = vec![];
    for building in self.buildings.iter_mut() {
      let mut done_training = false;
      if let Some(training) = building.train_queue.front_mut() {
        training.dur_left.ticks -= 1;
        done_training = training.dur_left.ticks == 0;
        if done_training {
          units_to_make.push((training.unit_type.clone(), building.spawn_location()));
        }
      }
      if done_training {
        building.train_queue.pop_front();
      }
    }
    for (unit, pos) in units_to_make {
      self.make_unit(unit, pos);
    }
  }

  fn next_uid(&mut self) -> UID {
    let uid = self.next_uid;
    if self.next_uid == UID::MAX {
      println!("error: ran out of UIDs!");
    }
    self.next_uid += 1;
    uid
  }

  pub fn make_unit(&mut self, unit_type: UnitType, pos: Point) {
    let uid = self.next_uid();
    self.units.push(Unit {
      uid,
      pos,
      unit_type,
      selected: false,
      waypoints: VecDeque::new(),
      // TODO: Make settable by unit type
      abilities: vec![AbilityBuild::new(uid, self.building_types[0].clone())],
    });
  }

  fn get_unit(&mut self, uid: UID) -> Option<&mut Unit> {
    self.units.iter_mut().find(|u| u.uid == uid)
  }

  pub fn make_building(&mut self, building_type: BuildingType, top_left_pos: TilePoint) {
    let uid = self.next_uid();
    self.buildings.push(Building {
      uid,

      top_left_pos,
      building_type,

      selected: false,

      train_queue: VecDeque::new(),
      train_queue_max_len: 5,

      abilities: vec![AbilityTrain::new(
        uid,
        // TODO: Avoid this array access. One way to do it is to define units in
        // a text file and validate them on load.
        self.unit_types[0].clone(),
      )],
    });
  }

  pub fn get_building(&mut self, uid: UID) -> Option<&mut Building> {
    self.buildings.iter_mut().find(|b| b.uid == uid)
  }
}

pub struct Unit {
  pub uid: UID,
  pub pos: Point,
  pub unit_type: UnitType,
  pub selected: bool,
  pub waypoints: VecDeque<Point>,
  pub abilities: Vec<Ability>,
}

impl Unit {
  fn speed(&self) -> Coord {
    self.unit_type.base_speed
  }

  fn rad(&self) -> Coord {
    self.unit_type.radius
  }

  fn bounding_box(&self) -> Rect {
    self.bounding_box_at(self.pos)
  }
  fn bounding_box_at(&self, p: Point) -> Rect {
    let top_left = p - Point::new(self.rad(), self.rad());
    Rect {
      top_left,
      width: self.rad() * Coord(2.),
      height: self.rad() * Coord(2.),
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
    let src = if self.waypoints.is_empty() {
      self.pos
    } else {
      self.waypoints[self.waypoints.len() - 1]
    };
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
      path_cost: 0,
    });
    while !to_visit.is_empty() {
      let point = to_visit.pop_front().unwrap();
      if visited.contains_key(&point.here) {
        continue;
      }
      visited.insert(point.here, point);
      if point.here == dest {
        break;
      }
      for p in point.here.neighbors4(&map) {
        let tile_blocked = map
          .get_tile(p)
          .map(|t| t != GridTile::Empty)
          .unwrap_or(true);
        if tile_blocked {
          continue;
        }
        to_visit.push_back(BackPath {
          here: p,
          best_source: point.here,
          path_cost: point.path_cost + 1,
        });
      }
    }
    if !visited.contains_key(&dest) {
      return false;
    }

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
    self.rad().0 as u32
  }
}

pub struct Building {
  pub uid: UID,

  pub top_left_pos: TilePoint,
  pub building_type: BuildingType,

  pub selected: bool,

  pub train_queue: VecDeque<UnitTraining>,
  pub train_queue_max_len: usize,

  pub abilities: Vec<Ability>,
}

impl Building {
  // Width in tiles.
  pub fn width(&self) -> u32 {
    self.building_type.width
  }
  // Height in tiles.
  pub fn height(&self) -> u32 {
    self.building_type.height
  }

  fn spawn_location(&self) -> Point {
    let tile_pos = self.top_left_pos + TilePoint::new(0, self.height());
    tile_pos.center_to_world_point()
  }
}

// Sort of a factory for units. Stores some properties of the unit so that one
// can make more of a type without closures.
//
// TODO: For both UnitTypes and BuildingTypes, add a list of the abilities they
// get. To do this, add a registry of Abilities and make them generically
// constructable.
#[derive(Clone)]
pub struct UnitType {
  pub name: &'static str,
  pub sprite_key: SpriteKey,
  pub radius: Coord,
  pub base_speed: Coord,
}

#[derive(Clone)]
pub struct BuildingType {
  pub name: &'static str,
  // TODO: Add sprite key

  // Width and height measured in tiles.
  pub width: u32,
  pub height: u32,
}

pub struct UnitTraining {
  pub unit_type: UnitType,
  pub dur_total: GameDur,
  pub dur_left: GameDur,
}
