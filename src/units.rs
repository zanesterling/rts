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

#[derive(Clone, Copy, Debug)]
pub struct ScreenCoord(pub i32);

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
    ScreenPoint {
      x: ScreenCoord((offset.x.0 * PIXELS_PER_WORLD) as i32),
      y: ScreenCoord((offset.y.0 * PIXELS_PER_WORLD) as i32),
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

#[derive(Clone, Copy)]
pub struct ScreenPoint { pub x: ScreenCoord, pub y: ScreenCoord }
impl ScreenPoint {
  pub fn to_world(self, camera: WorldPoint) -> WorldPoint {
    WorldPoint {
      x: WorldCoord(self.x.0 as f32 / PIXELS_PER_WORLD),
      y: WorldCoord(self.y.0 as f32 / PIXELS_PER_WORLD),
    } + camera
  }
}
