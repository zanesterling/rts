use crate::sprite_sheet::SpriteKey;

pub struct State {
  pub units: Vec<Unit>,
}

impl State {
  pub fn new() -> State {
    State {
      units: vec![
        Unit::new(300., 200., "newt_gingrich".to_string()),
        Unit::new(350., 300., "newt_gingrich".to_string()),
        Unit::new(450., 230., "newt_gingrich".to_string()),
      ],
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
  pub pos: Point,
  pub selected: bool,
  pub move_target: Option<Point>,
  pub base_speed: f32,
  pub sprite_key: SpriteKey,
}

impl Unit {
  pub fn new(x: f32, y: f32, sprite_key: SpriteKey) -> Unit {
    Unit {
      pos: Point::new(x, y),
      selected: false,
      move_target: None,
      base_speed: 1.,
      sprite_key,
    }
  }

  pub fn speed(&self) -> f32 { self.base_speed }
}

#[derive(Clone, Copy, Debug)]
pub struct Point { pub x: f32, pub y: f32 }
impl Point {
  pub fn new(x: f32, y: f32) -> Point {
    Point { x, y }
  }

  pub fn magnitude(self) -> f32 {
    let (x, y) = (self.x, self.y);
    f32::sqrt(x*x + y*y)
  }

  pub fn normalized(self) -> Point {
    let (x, y) = (self.x, self.y);
    if x == 0. && y == 0. { return self }
    let magnitude = self.magnitude();
    Point { x: x / magnitude, y: y / magnitude }
  }

  pub fn scaled(self, mag: f32) -> Point {
    Point { x: self.x*mag, y: self.y*mag }
  }

  pub fn plus(self, other: Point) -> Point {
    Point {
      x: self.x + other.x,
      y: self.y + other.y,
    }
  }

  pub fn minus(self, other: Point) -> Point {
    Point {
      x: self.x - other.x,
      y: self.y - other.y,
    }
  }
}