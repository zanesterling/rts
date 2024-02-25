pub use sdl2::rect::Point as ScreenPoint;

use std::ops::{Neg, Add, Sub, Mul, Div, SubAssign};
use std::cmp::{Ordering, PartialOrd};

const PIXELS_PER_WORLD: f32 = 1.;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldCoord(pub f32);
impl Neg for WorldCoord {
  type Output = Self;
  fn neg(self) -> Self { WorldCoord(-self.0) }
}
impl Add for WorldCoord {
  type Output = Self;
  fn add(self, rhs: Self) -> Self { WorldCoord(self.0 + rhs.0) }
}
impl Sub for WorldCoord {
  type Output = Self;
  fn sub(self, rhs: Self) -> Self { WorldCoord(self.0 - rhs.0) }
}
impl Mul for WorldCoord {
  type Output = Self;
  fn mul(self, rhs: Self) -> Self { WorldCoord(self.0 * rhs.0) }
}
impl Div for WorldCoord {
  type Output = Self;
  fn div(self, rhs: Self) -> Self { WorldCoord(self.0 / rhs.0) }
}
impl SubAssign for WorldCoord {
  fn sub_assign(&mut self, rhs: WorldCoord) {
    *self = *self - rhs;
  }
}
impl PartialOrd for WorldCoord {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    f32::partial_cmp(&self.0, &other.0)
  }
}

impl WorldCoord {
  pub fn clamp(self, lower: WorldCoord, upper: WorldCoord) -> WorldCoord {
    WorldCoord(self.0.clamp(lower.0, upper.0))
  }
}

#[derive(Clone, Copy, Debug)]
pub struct WorldPoint { pub x: WorldCoord, pub y: WorldCoord }
impl WorldPoint {
  pub fn new(x: WorldCoord, y: WorldCoord) -> WorldPoint {
    WorldPoint { x, y }
  }

  pub fn magnitude(self) -> WorldCoord {
    let (x, y) = (self.x, self.y);
    WorldCoord(f32::sqrt((x*x + y*y).0))
  }

  pub fn normalized(self) -> WorldPoint {
    let (x, y) = (self.x, self.y);
    if x == WorldCoord(0.) && y == WorldCoord(0.) { return self }
    let magnitude = self.magnitude();
    self / magnitude
  }

  pub fn to_screen(self, camera: WorldPoint) -> ScreenPoint {
    let offset = self - camera;
    ScreenPoint::new(
      (offset.x.0 * PIXELS_PER_WORLD) as i32,
      (offset.y.0 * PIXELS_PER_WORLD) as i32,
    )
  }

  // Clamps point p to the rectangle.
  pub fn clamp(self, r: &WorldRect) -> WorldPoint {
    WorldPoint {
      x: self.x.clamp(r.top_left.x, r.top_left.x + r.width),
      y: self.y.clamp(r.top_left.y, r.top_left.y + r.height),
    }
  }
}

impl Add for WorldPoint {
  type Output = Self;
  fn add(self, other: Self) -> Self {
    WorldPoint {
      x: self.x + other.x,
      y: self.y + other.y,
    }
  }
}
impl Sub for WorldPoint {
  type Output = Self;
  fn sub(self, other: WorldPoint) -> Self {
    WorldPoint {
      x: self.x - other.x,
      y: self.y - other.y,
    }
  }
}
impl Mul<WorldCoord> for WorldPoint {
  type Output = Self;
  fn mul(self, mag: WorldCoord) -> Self {
    WorldPoint { x: self.x * mag, y: self.y * mag, }
  }
}
impl Div<WorldCoord> for WorldPoint {
  type Output = Self;
  fn div(self, mag: WorldCoord) -> Self {
    WorldPoint { x: self.x / mag, y: self.y / mag, }
  }
}
impl SubAssign for WorldPoint {
  fn sub_assign(&mut self, rhs: Self) {
    *self = *self - rhs;
  }
}

pub trait ToWorld {
  fn to_world(self, camera: WorldPoint) -> WorldPoint;
}

impl ToWorld for ScreenPoint {
  fn to_world(self, camera: WorldPoint) -> WorldPoint {
    WorldPoint {
      x: WorldCoord(self.x() as f32 / PIXELS_PER_WORLD),
      y: WorldCoord(self.y() as f32 / PIXELS_PER_WORLD),
    } + camera
  }
}

#[derive(Debug)]
pub struct WorldRect {
  pub top_left: WorldPoint,
  pub width:    WorldCoord,
  pub height:   WorldCoord,
}

impl WorldRect {
  pub fn contains(&self, p: WorldPoint) -> bool {
    self.top_left.x <= p.x &&
    p.x <= self.top_left.x + self.width &&
    self.top_left.y <= p.y &&
    p.y <= self.top_left.y + self.height
  }

  pub fn intersects(&self, other: &WorldRect) -> bool {
    let (p1, p2, p3, p4) = self.points();
    let (q1, q2, q3, q4) = other.points();
    other.contains(p1) || other.contains(p2) ||
    other.contains(p3) || other.contains(p4) ||
    self.contains(q1) || self.contains(q2) ||
    self.contains(q3) || self.contains(q4)
  }

  fn points(&self) -> (WorldPoint, WorldPoint, WorldPoint, WorldPoint) {
    (
      self.top_left,
      self.top_left + WorldPoint::new(self.width,     WorldCoord(0.)),
      self.top_left + WorldPoint::new(WorldCoord(0.), self.height),
      self.top_left + WorldPoint::new(self.width,     self.height),
    )
  }
}