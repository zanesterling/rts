#[allow(dead_code)]
mod dimensions;
#[allow(dead_code)]
mod game;
#[allow(dead_code)]
mod map;
#[allow(dead_code)]
mod sprite_sheet;

extern crate rand;
extern crate sdl2;

use rand::Rng;
use sdl2::event::{Event, WindowEvent};
use sdl2::image;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::ttf::Font;
use sdl2::video::Window;
use sdl2::Sdl;

use std::process::exit;
use std::rc::Rc;
use std::thread::sleep;
use std::time::Duration;

use crate::dimensions::{DisplayPoint, ToWorld, WindowPoint, WorldCoord, WorldPoint};
use crate::map::{GridTile, TILE_WIDTH};
use crate::sprite_sheet::SpriteSheet;

const EMPTY_TILE_COLOR: Color = Color::RGB(40, 42, 54);
const OBSTACLE_COLOR: Color = Color::RGB(255, 184, 108);
const UNIT_COLOR: Color = Color::RGB(255, 121, 198);
const UNIT_SELECTED_COLOR: Color = Color::RGB(80, 250, 123);
const UNIT_MOVING_COLOR: Color = Color::RGB(189, 147, 249);
const DRAG_PERIMETER_COLOR: Color = Color::RGB(0, 255, 0);
const WAYPOINT_COLOR: Color = UNIT_MOVING_COLOR;

const COLOR_WHITE: Color = Color::RGB(248, 248, 242);

const WAYPOINT_RAD: u32 = 2;

const SPRITE_SHEET_PATH: &str = "media/sprite-sheet.sps";
const SHOW_UNIT_DEBUG_BOXES: bool = false;

// TODO: Do some stuff to pick the right window / let user pick.
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;
const DISPLAY_TL_X: i32 = 1524;
const DISPLAY_TL_Y: i32 = 446;
const DISPLAY_BR_X: i32 = 3202 + WINDOW_WIDTH as i32;
const DISPLAY_BR_Y: i32 = 1256 + WINDOW_HEIGHT as i32;

struct State<'a, 'b> {
    // "Immutable" stuff.
    sprite_sheet: SpriteSheet<'a>,
    font: Font<'b, 'static>,
    display_bounds: DisplayBounds,

    // State of the game.
    running: bool,
    game: game::State,

    // Interaction state.
    cursor_state: CursorState,
    key_state: KeyState,
    camera_pos: WorldPoint,
    window_pos: DisplayPoint,
    mouse_pos: WindowPoint,
}

impl<'a, 'b> State<'a, 'b> {
    pub fn new<'s, 'f>(
        sprite_sheet: SpriteSheet<'s>,
        display_bounds: DisplayBounds,
        font: Font<'f, 'static>,
    ) -> State<'s, 'f> {
        State {
            sprite_sheet,
            font,
            display_bounds,

            running: true,
            game: game::State::new(),

            cursor_state: CursorState::None,
            key_state: KeyState::new(),
            camera_pos: WorldPoint::new(WorldCoord(0.), WorldCoord(0.)),
            // This is wrong, but will be set on the first WindowMove event.
            window_pos: DisplayPoint::new(0, 0),
            // This is wrong, but will be set on the next MouseMotion event.
            mouse_pos: WindowPoint::new(0, 0),
        }
    }

    // Returns a world point corresponding to the top-left corner of the
    // renderable window.
    pub fn camera_pos(&self) -> WorldPoint {
        self.camera_pos
    }
}

#[derive(Clone)]
enum CursorState {
    None,
    BoxSelect(BoxSelect),
    CameraDrag,
    AbilitySelected(Rc<dyn game::Ability>),
}

#[derive(Clone, Copy)]
struct BoxSelect {
    from: WorldPoint,
    to: WorldPoint,
}

#[allow(dead_code)]
struct DisplayBounds {
    top_left_x: i32,
    top_left_y: i32,
    width: u32,
    height: u32,
}

struct KeyState {
    left_ctrl_down: bool,
    right_ctrl_down: bool,
    left_shift_down: bool,
    right_shift_down: bool,
    left_alt_down: bool,
    right_alt_down: bool,
}

#[allow(dead_code)]
impl KeyState {
    pub fn new() -> KeyState {
        KeyState {
            left_ctrl_down: false,
            right_ctrl_down: false,
            left_shift_down: false,
            right_shift_down: false,
            left_alt_down: false,
            right_alt_down: false,
        }
    }

    pub fn ctrl(&self) -> bool {
        self.left_ctrl_down || self.right_ctrl_down
    }
    pub fn shift(&self) -> bool {
        self.left_shift_down || self.right_shift_down
    }
    pub fn alt(&self) -> bool {
        self.left_alt_down || self.right_alt_down
    }
}

impl BoxSelect {
    fn resolve(&self, final_pt: WindowPoint, state: &mut State) {
        let camera_pos = state.camera_pos();
        let selection_rect = rect_from_points(self.from.to_window(camera_pos), final_pt);
        for unit in state.game.units.iter_mut() {
            let unit_bounds =
                rect_from_center_rad(unit.pos.to_window(camera_pos), unit.window_rad());
            unit.selected = selection_rect.has_intersection(unit_bounds);
        }
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let _sdl_image_context = image::init(image::InitFlag::PNG).unwrap();

    let sdl_ttf_context = sdl2::ttf::init().unwrap();
    let font = sdl_ttf_context
        .load_font("media/Serif.ttf", 24)
        .expect("couldn't load font");

    let video = sdl_context.video().unwrap();

    let window = video
        .window("rts!", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let canvas_txc = canvas.texture_creator();

    let sprite_sheet = SpriteSheet::from_file(SPRITE_SHEET_PATH, &canvas_txc).unwrap_or_else(|e| {
        println!(
            "error loading sprite sheet \"{}\": {}",
            SPRITE_SHEET_PATH, e
        );
        exit(1);
    });

    let state = {
        let display_bounds = DisplayBounds {
            top_left_x: DISPLAY_TL_X,
            top_left_y: DISPLAY_TL_Y,
            width: (DISPLAY_BR_X - DISPLAY_TL_X) as u32,
            height: (DISPLAY_BR_Y - DISPLAY_TL_Y) as u32,
        };
        State::new(sprite_sheet, display_bounds, font)
    };
    render(&mut canvas, &state);
    canvas.present();
    main_loop(state, canvas, sdl_context);
}

fn main_loop(mut state: State, mut canvas: Canvas<Window>, sdl_context: Sdl) {
    let mut event_pump = sdl_context.event_pump().unwrap();
    while state.running {
        // Update world.
        state.game.tick();

        // Render.
        render(&mut canvas, &state);
        canvas.present();
        sleep(Duration::new(0, 1_000_000_000u32 / 120));

        // Handle input.
        for event in event_pump.poll_iter() {
            handle_event(&mut state, &mut canvas, event);
        }
    }
}

fn handle_event(state: &mut State, canvas: &mut Canvas<Window>, event: Event) {
    match event {
        // Quit.
        Event::Quit { .. }
        | Event::KeyDown {
            keycode: Some(Keycode::Escape),
            ..
        } => {
            state.running = false;
        }

        // Left mouse down / up: box select.
        Event::MouseButtonDown {
            x,
            y,
            mouse_btn: MouseButton::Left,
            ..
        } => {
            let scr_click = WindowPoint::new(x, y);
            let from = scr_click.to_world(state.camera_pos());
            match &state.cursor_state {
                CursorState::AbilitySelected(ability) => {
                    ability.cast(&mut state.game, state.mouse_pos.to_world(state.camera_pos));
                    state.cursor_state = CursorState::None;
                }
                _ => {
                    state.cursor_state = CursorState::BoxSelect(BoxSelect { from, to: from });
                }
            }
        }
        Event::MouseButtonUp {
            x,
            y,
            mouse_btn: MouseButton::Left,
            ..
        } => {
            // Select units that are in the box.
            if let CursorState::BoxSelect(box_select) = state.cursor_state {
                box_select.resolve(WindowPoint::new(x, y), state);
            }
            state.cursor_state = CursorState::None;
        }

        // Right mouse button -- issue or queue move command.
        Event::MouseButtonDown {
            x,
            y,
            mouse_btn: MouseButton::Right,
            ..
        } => {
            let click_pos = WindowPoint::new(x, y).to_world(state.camera_pos());
            for unit in state.game.units.iter_mut() {
                if unit.selected {
                    if !state.key_state.shift() {
                        unit.waypoints.clear();
                    }
                    let found_path = unit.pathfind(&state.game.map, click_pos);
                    if !found_path {
                        unit.waypoints.push_back(click_pos);
                    }
                }
            }
        }

        // Middle mouse down/up: drag view.
        Event::MouseButtonDown {
            x,
            y,
            mouse_btn: MouseButton::Middle,
            ..
        } => {
            // End box-select if you middle mouse click-n-drag.
            if let CursorState::BoxSelect(box_select) = state.cursor_state {
                box_select.resolve(WindowPoint::new(x, y), state);
            }
            state.cursor_state = CursorState::CameraDrag;
        }
        Event::MouseButtonUp {
            mouse_btn: MouseButton::Middle,
            ..
        } => {
            if let CursorState::CameraDrag = &state.cursor_state {
                state.cursor_state = CursorState::None;
            }
        }

        Event::MouseMotion {
            x, y, xrel, yrel, ..
        } => {
            let camera_pos = state.camera_pos();
            state.mouse_pos = WindowPoint::new(x, y);
            match &mut state.cursor_state {
                CursorState::BoxSelect(box_select) => {
                    box_select.to = WindowPoint::new(x, y).to_world(camera_pos);
                }
                CursorState::CameraDrag => {
                    state.camera_pos -= WorldPoint {
                        x: WorldCoord(xrel as f32),
                        y: WorldCoord(yrel as f32),
                    };
                }
                CursorState::AbilitySelected(_) => {}
                CursorState::None => {}
            }
        }

        Event::KeyDown {
            repeat: false,
            keycode,
            ..
        } => {
            let ist = &mut state.key_state;
            match keycode {
                Some(Keycode::LCtrl) => {
                    ist.left_ctrl_down = true;
                }
                Some(Keycode::RCtrl) => {
                    ist.right_ctrl_down = true;
                }
                Some(Keycode::LShift) => {
                    ist.left_shift_down = true;
                }
                Some(Keycode::RShift) => {
                    ist.right_shift_down = true;
                }
                Some(Keycode::LAlt) => {
                    ist.left_alt_down = true;
                }
                Some(Keycode::RAlt) => {
                    ist.right_alt_down = true;
                }

                Some(Keycode::R) => {
                    let mut thread_rng = rand::thread_rng();
                    let x = thread_rng.gen_range(DISPLAY_TL_X..DISPLAY_BR_X - WINDOW_WIDTH as i32);
                    let y = thread_rng.gen_range(DISPLAY_TL_Y..DISPLAY_BR_Y - WINDOW_HEIGHT as i32);
                    canvas.window_mut().set_position(
                        sdl2::video::WindowPos::Positioned(x),
                        sdl2::video::WindowPos::Positioned(y),
                    );
                }
                Some(Keycode::P) => {
                    let (x, y) = canvas.window().position();
                    println!("({}, {})", x, y);
                }

                Some(keycode) => {
                    // If the key corresponds to a usable ability on a selected
                    // unit, go into the AbilitySelected state, so that it will
                    // cast on the next click.
                    let mut ability = None;
                    for unit in state.game.units.iter() {
                        if !unit.selected {
                            continue;
                        }
                        for x in unit.abilities.iter() {
                            if x.keycode() == keycode {
                                ability = Some((*x).clone());
                            }
                        }
                    }
                    if let Some(ability) = ability {
                        state.cursor_state = CursorState::AbilitySelected(ability);
                    }
                }

                _ => {}
            }
        }
        Event::KeyUp { keycode, .. } => {
            let ist = &mut state.key_state;
            match keycode {
                Some(Keycode::LCtrl) => {
                    ist.left_ctrl_down = false;
                }
                Some(Keycode::RCtrl) => {
                    ist.right_ctrl_down = false;
                }
                Some(Keycode::LShift) => {
                    ist.left_shift_down = false;
                }
                Some(Keycode::RShift) => {
                    ist.right_shift_down = false;
                }
                Some(Keycode::LAlt) => {
                    ist.left_alt_down = false;
                }
                Some(Keycode::RAlt) => {
                    ist.right_alt_down = false;
                }
                _ => {}
            }
        }

        Event::Window {
            win_event: WindowEvent::Moved(x, y),
            ..
        } => {
            state.window_pos = DisplayPoint::new(
                x - state.display_bounds.top_left_x,
                y - state.display_bounds.top_left_y,
            );
        }

        _ => {}
    }
}

fn render(canvas: &mut Canvas<Window>, state: &State) {
    canvas.set_draw_color(Color::BLACK);
    canvas.clear();

    // Draw terrain.
    for tile in state.game.map.tiles() {
        canvas.set_draw_color(match tile.tile {
            GridTile::Empty => EMPTY_TILE_COLOR,
            GridTile::Obstacle => OBSTACLE_COLOR,
        });
        let window_pos = tile.pos.to_world_point().to_window(state.camera_pos());
        let _ = canvas.fill_rect(Rect::new(
            window_pos.x(),
            window_pos.y(),
            TILE_WIDTH,
            TILE_WIDTH,
        ));
    }

    // Draw units.
    for unit in state.game.units.iter() {
        if unit.selected {
            for p in unit.waypoints.iter() {
                draw_waypoint(canvas, p.to_window(state.camera_pos()));
            }
        }

        // Draw unit.
        let bounds =
            rect_from_center_rad(unit.pos.to_window(state.camera_pos()), unit.window_rad());
        let _ = state
            .sprite_sheet
            .blit_sprite_to_rect(unit.sprite_key.as_str(), canvas, bounds);

        // Draw debug box around the unit.
        if SHOW_UNIT_DEBUG_BOXES || unit.selected {
            canvas.set_draw_color(if unit.selected {
                UNIT_SELECTED_COLOR
            } else if unit.move_queued() {
                UNIT_MOVING_COLOR
            } else {
                UNIT_COLOR
            });
            let _ = canvas.draw_rect(bounds);
        }
    }

    // Draw box-selection box.
    if let CursorState::BoxSelect(box_select) = &state.cursor_state {
        canvas.set_draw_color(DRAG_PERIMETER_COLOR);
        let _ = canvas.draw_rect(rect_from_points(
            box_select.from.to_window(state.camera_pos()),
            box_select.to.to_window(state.camera_pos()),
        ));
    }

    // Draw abilities, if there are any selected units.
    let unit = state.game.units.iter().find(|unit| unit.selected);
    if let Some(unit) = unit {
        for ability in unit.abilities.iter() {
            let text = format!("[{}] {}", ability.keycode(), ability.name());
            let top_left = WindowPoint::new(0, 0);
            draw_text(canvas, top_left, &state.font, &text)
                .expect("couldn't draw units' abilities");
        }
    }

    if let CursorState::AbilitySelected(ability) = &state.cursor_state {
        let top_left= WindowPoint::new(0, WINDOW_HEIGHT as i32 - state.font.height());
        draw_text(canvas, top_left, &state.font, ability.name())
            .expect("couldn't draw active ability");
    }
}

fn draw_waypoint(canvas: &mut Canvas<Window>, p: WindowPoint) {
    canvas.set_draw_color(WAYPOINT_COLOR);
    let _ = canvas.draw_rect(rect_from_center_rad(p, WAYPOINT_RAD));
}

// TODO: Clean up this code.
// This is intensely janky and low-res, and it's creating a new surface
// every single frame for each ability that a selected unit has.
fn draw_text(canvas: &mut Canvas<Window>, p: WindowPoint, font: &Font, text: &str) -> Result<(), String> {
    let surface = font.render(text).solid(COLOR_WHITE)
        .map_err(|e| format!("couldn't render text: {}", e))?;
    let texture_creator = canvas.texture_creator();
    let texture = texture_creator.create_texture_from_surface(&surface)
        .map_err(|e| format!("couldn't create texture: {}", e))?;

    let bounds = texture.query();
    let target_rect = Rect::new(p.x, p.y, bounds.width, bounds.height);
    canvas.copy(&texture, None, target_rect)
        .map_err(|e| format!("couldn't copy texture to canvas: {}", e))
}

fn rect_from_points(p1: WindowPoint, p2: WindowPoint) -> Rect {
    let xmin = i32::min(p1.x(), p2.x());
    let xmax = i32::max(p1.x(), p2.x());
    let ymin = i32::min(p1.y(), p2.y());
    let ymax = i32::max(p1.y(), p2.y());
    Rect::new(xmin, ymin, (xmax - xmin) as u32, (ymax - ymin) as u32)
}

fn rect_from_center_rad(p: WindowPoint, rad: u32) -> Rect {
    Rect::from_center(p, rad * 2, rad * 2)
}
