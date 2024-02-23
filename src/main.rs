#[allow(dead_code)]
mod game;
#[allow(dead_code)]
mod sprite_sheet;
mod dimensions;

extern crate sdl2;

use sdl2::Sdl;
use sdl2::event::Event;
use sdl2::image;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use std::process::exit;
use std::thread::sleep;
use std::time::Duration;

use crate::game::{GridTile, TILE_WIDTH, tile_pos};
use crate::sprite_sheet::SpriteSheet;
use crate::dimensions::{WorldCoord, WorldPoint, ScreenCoord, ScreenPoint};

const EMPTY_TILE_COLOR: Color = Color::RGB(40, 42, 54);
const OBSTACLE_COLOR: Color = Color::RGB(255, 184, 108);
const UNIT_COLOR: Color = Color::RGB(255, 121, 198);
const UNIT_SELECTED_COLOR: Color = Color::RGB(80, 250, 123);
const UNIT_MOVING_COLOR: Color = Color::RGB(189, 147, 249);
const DRAG_PERIMETER_COLOR: Color = Color::RGB(0, 255, 0);

const SPRITE_SHEET_PATH: &str = "media/sprite-sheet.sps";
const SHOW_UNIT_DEBUG_BOXES: bool = false;

struct State<'a> {
    running: bool,
    game: game::State,
    drag_state: DragState,
    sprite_sheet: SpriteSheet<'a>,
    camera_pos: WorldPoint,
    input_state: InputState,
}

impl<'a> State<'a> {
    pub fn new<'s>(sprite_sheet: SpriteSheet<'s>) -> State<'s> {
        State {
            running: true,
            game: game::State::new(),
            drag_state: DragState::None,
            sprite_sheet,
            camera_pos: WorldPoint::new(WorldCoord(0.), WorldCoord(0.)),
            input_state: InputState::new(),
        }
    }
}

#[derive(Clone, Copy)]
enum DragState {
    None,
    BoxSelect(BoxSelect),
    CameraDrag,
}

#[derive(Clone, Copy)]
struct BoxSelect {
    from: WorldPoint,
    to: WorldPoint,
}


struct InputState {
    left_ctrl_down: bool,
    right_ctrl_down: bool,
    left_shift_down: bool,
    right_shift_down: bool,
    left_alt_down: bool,
    right_alt_down: bool,
}

#[allow(dead_code)]
impl InputState {
    pub fn new() -> InputState {
        InputState {
            left_ctrl_down: false,
            right_ctrl_down: false,
            left_shift_down: false,
            right_shift_down: false,
            left_alt_down: false,
            right_alt_down: false,
        }
    }

    pub fn ctrl(&self) -> bool { self.left_ctrl_down || self.right_ctrl_down }
    pub fn shift(&self) -> bool { self.left_shift_down || self.right_shift_down }
    pub fn alt(&self) -> bool { self.left_alt_down || self.right_alt_down }
}

impl BoxSelect {
    fn resolve(&self, final_pt: ScreenPoint, state: &mut State) {
        let rect = rect_from_points(
            self.from.to_screen(state.camera_pos), final_pt);
        for unit in state.game.units.iter_mut() {
            let rad = unit.rad;
            let top_left = unit.pos - WorldPoint::new(rad, rad);
            let top_left_screen = top_left.to_screen(state.camera_pos);
            unit.selected = rect.has_intersection(
                Rect::new(
                    top_left_screen.x.0,
                    top_left_screen.y.0,
                    (rad.0 * 2.) as u32,
                    (rad.0 * 2.) as u32
                )
            );
        }
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let _sdl_image_context = image::init(image::InitFlag::PNG).unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("rts!", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas()
        .present_vsync()
        .build().unwrap();
    let canvas_txc = canvas.texture_creator();

    let sprite_sheet =
        SpriteSheet::from_file(SPRITE_SHEET_PATH, &canvas_txc)
            .unwrap_or_else(|e| {
                println!(
                    "error loading sprite sheet \"{}\": {}",
                    SPRITE_SHEET_PATH,
                    e
                );
                exit(1);
            });

    let state = State::new(sprite_sheet);
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
            handle_event(&mut state, event);
        }
    }
}

fn handle_event(state: &mut State, event: Event) {
    match event {
        // Quit.
        Event::Quit {..} |
        Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
            state.running = false;
        },

        // Left mouse down / up: box select.
        Event::MouseButtonDown {x, y, mouse_btn: MouseButton::Left, ..} => {
            let scr_click = ScreenPoint{x: ScreenCoord(x), y: ScreenCoord(y)};
            let from = scr_click.to_world(state.camera_pos);
            state.drag_state = DragState::BoxSelect(BoxSelect {
                from,
                to: from,
            });
        },
        Event::MouseButtonUp {x, y, mouse_btn: MouseButton::Left, ..} => {
            // Select units that are in the box.
            if let DragState::BoxSelect(box_select) = state.drag_state {
                let (x, y) = (ScreenCoord(x), ScreenCoord(y));
                box_select.resolve(ScreenPoint{x, y}, state);
            }
            state.drag_state = DragState::None;
        },

        // Right mouse button -- issue or queue move command.
        Event::MouseButtonDown {x, y, mouse_btn: MouseButton::Right, ..} => {
            let (x, y) = (ScreenCoord(x), ScreenCoord(y));
            let click_pos = ScreenPoint{x, y}
                .to_world(state.camera_pos);
            for unit in state.game.units.iter_mut() {
                if unit.selected {
                    if state.input_state.shift() {
                        unit.queue_move(click_pos);
                    } else {
                        unit.move_to(click_pos);
                    }
                }
            }
        },

        // Middle mouse down/up: drag view.
        Event::MouseButtonDown {x, y, mouse_btn: MouseButton::Middle, .. } => {
            // End box-select if you middle mouse click-n-drag.
            if let DragState::BoxSelect(box_select) = state.drag_state {
                let (x, y) = (ScreenCoord(x), ScreenCoord(y));
                box_select.resolve(ScreenPoint{x, y}, state);
            }
            state.drag_state = DragState::CameraDrag;
        },
        Event::MouseButtonUp {mouse_btn: MouseButton::Middle, .. } => {
            if let DragState::CameraDrag = &state.drag_state {
                state.drag_state = DragState::None;
            }
        },

        Event::MouseMotion {x, y, xrel, yrel, ..} => {
            match &mut state.drag_state {
                DragState::BoxSelect(box_select) => {
                    box_select.to = ScreenPoint {
                        x: ScreenCoord(x),
                        y: ScreenCoord(y),
                    }.to_world(state.camera_pos);
                },
                DragState::CameraDrag => {
                    state.camera_pos -= WorldPoint {
                        x: WorldCoord(xrel as f32),
                        y: WorldCoord(yrel as f32),
                    };
                },
                DragState::None => {},
            }
        },

        Event::KeyDown {repeat: false, keycode, ..} => {
            let ist = &mut state.input_state;
            match keycode {
                Some(Keycode::LCtrl) => { ist.left_ctrl_down = true; }
                Some(Keycode::RCtrl) => { ist.right_ctrl_down = true; }
                Some(Keycode::LShift) => { ist.left_shift_down = true; }
                Some(Keycode::RShift) => { ist.right_shift_down = true; }
                Some(Keycode::LAlt) => { ist.left_alt_down = true; }
                Some(Keycode::RAlt) => { ist.right_alt_down = true; }
                _ => {},
            }
        },
        Event::KeyUp {keycode, ..} => {
            let ist = &mut state.input_state;
            match keycode {
                Some(Keycode::LCtrl) => { ist.left_ctrl_down = false; }
                Some(Keycode::RCtrl) => { ist.right_ctrl_down = false; }
                Some(Keycode::LShift) => { ist.left_shift_down = false; }
                Some(Keycode::RShift) => { ist.right_shift_down = false; }
                Some(Keycode::LAlt) => { ist.left_alt_down = false; }
                Some(Keycode::RAlt) => { ist.right_alt_down = false; }
                _ => {},
            }
        },

        _ => {},
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
        let window_pos = tile_pos(tile.x, tile.y).to_screen(state.camera_pos);
        let _ = canvas.fill_rect(
            Rect::new(
                window_pos.x.0, window_pos.y.0,
                TILE_WIDTH, TILE_WIDTH));
    }

    // Draw units.
    for unit in state.game.units.iter() {
        let rad = unit.rad;
        let top_left = unit.pos - WorldPoint::new(rad, rad);
        let top_left_scr = top_left.to_screen(state.camera_pos);
        let diam = unit.rad.0 * 2.;
        let dst = Rect::new(
            top_left_scr.x.0, top_left_scr.y.0,
            diam as u32, diam as u32
        );

        // Draw unit.
        let _ = state.sprite_sheet.blit_sprite_to_rect(
            unit.sprite_key.as_str(), canvas, dst);

        // Draw debug box around the unit.
        if SHOW_UNIT_DEBUG_BOXES || unit.selected {
            canvas.set_draw_color(
                if unit.selected { UNIT_SELECTED_COLOR }
                else if unit.move_queued() { UNIT_MOVING_COLOR }
                else { UNIT_COLOR }
            );
            let _ = canvas.draw_rect(dst);
        }
    }

    // Draw box-selection box.
    if let DragState::BoxSelect(box_select) = &state.drag_state {
        canvas.set_draw_color(DRAG_PERIMETER_COLOR);
        let _ = canvas.draw_rect(rect_from_points(
            box_select.from.to_screen(state.camera_pos),
            box_select.to.to_screen(state.camera_pos),
        ));
    }
}

fn rect_from_points(p1: ScreenPoint, p2: ScreenPoint) -> Rect {
    let xmin = i32::min(p1.x.0, p2.x.0);
    let xmax = i32::max(p1.x.0, p2.x.0);
    let ymin = i32::min(p1.y.0, p2.y.0);
    let ymax = i32::max(p1.y.0, p2.y.0);
    Rect::new(
        xmin, ymin,
        (xmax-xmin) as u32, (ymax-ymin) as u32)
}
