pub struct State {
  pub units: Vec<Unit>,
}

pub struct Unit {
  pub world_x: i32,
  pub world_y: i32,
  pub selected: bool,
}

impl Unit {
  pub fn new(x: i32, y: i32) -> Unit {
    Unit {
      world_x: x,
      world_y: y,
      selected: false,
    }
  }
}
