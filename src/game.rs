pub struct State {
  pub units: Vec<Unit>,
}

impl State {
  pub fn new() -> State {
    State {
      units: vec![
        Unit::new(300, 200),
        Unit::new(350, 300),
        Unit::new(450, 230),
      ],
    }
  }
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
