use crate::dimensions::WorldPoint as Point;
use crate::game::{BuildingType, GameDur, State, UnitTraining, UnitType, UID};
use crate::map::{TilePoint, ToTilePoint, TILE_WIDTH};

use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect as SdlRect;
use sdl2::render::{BlendMode, Canvas};
use sdl2::video::Window;

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

  // Draw anything you want to while the ability is selected.
  fn draw(&self, canvas: &mut Canvas<Window>, mouse: Point, camera: Point);
}

// An ability for worker units: build a building at the target location.
pub struct AbilityBuild {
  caster: UID,
  building_type: BuildingType,
}

impl AbilityBuild {
  pub fn new(caster: UID, building_type: BuildingType) -> Ability {
    Ability::PointTargeted(Rc::new(AbilityBuild {
      caster,
      building_type,
    }))
  }

  fn where_to_build(&self, mouse: Point) -> TilePoint {
    // Do the division in integer space to right behavior for even- and
    // odd-sided buildings.
    let building_half_dim =
      TilePoint::new(self.building_type.width / 2, self.building_type.height / 2);
    (mouse - building_half_dim.to_world_point()).to_tile_point()
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

const BUILD_GHOST_COLOR: Color = Color::RGBA(139, 233, 253, 128);
impl PointTargetedAbility for AbilityBuild {
  fn cast(&self, state: &mut State, target: Point) {
    state.make_building(self.building_type.clone(), self.where_to_build(target));
  }

  fn draw(&self, canvas: &mut Canvas<Window>, mouse: Point, camera: Point) {
    let build_pos = self.where_to_build(mouse);
    canvas.set_draw_color(BUILD_GHOST_COLOR);
    canvas.set_blend_mode(BlendMode::Blend);
    let top_left = build_pos.to_world_point().to_window(camera);
    let width = self.building_type.width * TILE_WIDTH;
    let height = self.building_type.height * TILE_WIDTH;
    let _ = canvas.fill_rect(SdlRect::new(top_left.x, top_left.y, width, height));
  }
}

// An ability for production structures: add a unit to the train queue.
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
