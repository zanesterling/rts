use crate::dimensions::WorldPoint as Point;
use crate::game::{GameDur, State, UnitTraining, UnitType, UID};

use sdl2::keyboard::Keycode;

use std::rc::Rc;

#[derive(Clone)]
pub enum Ability {
  NonTargeted(Rc<dyn NonTargetedAbility>),
  PointTargeted(Rc<dyn PointTargetedAbility>),
  // TODO: Add unit- and building- targeted abilities.
}

pub trait AbilityCommon {
  fn keycode(&self) -> Keycode;
  fn name(&self) -> &'static str;
  // TODO: When units can die we should use this to stop trying to cast
  // active abilities from the dead unit.
  fn caster(&self) -> UID;
}

impl AbilityCommon for Ability {
  fn keycode(&self) -> Keycode {
    match self {
      Ability::NonTargeted(ab) => ab.keycode(),
      Ability::PointTargeted(ab) => ab.keycode(),
    }
  }

  fn name(&self) -> &'static str {
    match self {
      Ability::NonTargeted(ab) => ab.name(),
      Ability::PointTargeted(ab) => ab.name(),
    }
  }

  fn caster(&self) -> UID {
    match self {
      Ability::NonTargeted(ab) => ab.caster(),
      Ability::PointTargeted(ab) => ab.caster(),
    }
  }
}

pub trait NonTargetedAbility: AbilityCommon {
  fn cast(&self, state: &mut State);
}

pub trait PointTargetedAbility: AbilityCommon {
  fn cast(&self, state: &mut State, target: Point);
}

pub struct AbilityBuild {
  caster: UID,
}

impl AbilityBuild {
  pub fn new(caster: UID) -> Ability {
    Ability::PointTargeted(Rc::new(AbilityBuild { caster }))
  }
}

impl AbilityCommon for AbilityBuild {
  fn keycode(&self) -> Keycode {
    Keycode::B
  }
  fn name(&self) -> &'static str {
    "Build"
  }
  fn caster(&self) -> UID {
    self.caster
  }
}

impl PointTargetedAbility for AbilityBuild {
  fn cast(&self, state: &mut State, target: Point) {
    state.make_unit(state.unit_types[0].clone(), target);
  }
}

pub struct AbilityTrain {
  caster: UID,
  unit_type: UnitType,
}

impl AbilityTrain {
  pub fn new(caster: UID, unit_type: UnitType) -> Ability {
    Ability::NonTargeted(Rc::new(AbilityTrain { caster, unit_type }))
  }
}

impl AbilityCommon for AbilityTrain {
  fn caster(&self) -> UID {
    self.caster
  }

  fn keycode(&self) -> Keycode {
    Keycode::T
  }

  fn name(&self) -> &'static str {
    "Train unit"
  }
}

impl NonTargetedAbility for AbilityTrain {
  fn cast(&self, state: &mut State) {
    let unit_type = state.unit_types[0].clone();
    let building = state
      .get_building(self.caster)
      // TODO: Make cast() give a result
      .expect("caster not found when casting ability");
    let train_dur = GameDur::from_secs(3);
    if building.train_queue_max_len > building.train_queue.len() {
      building.train_queue.push_back(UnitTraining {
        unit_type,
        dur_total: train_dur,
        dur_left: train_dur,
      });
    } else {
      // TODO: Signal to the user that casting failed.
    }
  }
}
