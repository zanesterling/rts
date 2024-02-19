pub type WorldCoord = f32;

#[derive(Clone, Copy, Debug)]
pub struct WorldPoint { pub x: WorldCoord, pub y: WorldCoord }
impl WorldPoint {
  pub fn new(x: f32, y: f32) -> WorldPoint {
    WorldPoint { x, y }
  }

  pub fn magnitude(self) -> f32 {
    let (x, y) = (self.x, self.y);
    f32::sqrt(x*x + y*y)
  }

  pub fn normalized(self) -> WorldPoint {
    let (x, y) = (self.x, self.y);
    if x == 0. && y == 0. { return self }
    let magnitude = self.magnitude();
    WorldPoint { x: x / magnitude, y: y / magnitude }
  }

  pub fn scaled(self, mag: f32) -> WorldPoint {
    WorldPoint { x: self.x*mag, y: self.y*mag }
  }

  pub fn plus(self, other: WorldPoint) -> WorldPoint {
    WorldPoint {
      x: self.x + other.x,
      y: self.y + other.y,
    }
  }

  pub fn minus(self, other: WorldPoint) -> WorldPoint {
    WorldPoint {
      x: self.x - other.x,
      y: self.y - other.y,
    }
  }
}
