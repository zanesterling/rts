#[allow(dead_code)]
mod game;
#[allow(dead_code)]
mod sprite_sheet;
mod units;

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

use crate::game::{GridTile, TILE_WIDTH};
use crate::sprite_sheet::SpriteSheet;
use crate::units::WorldPoint;

const OBJ_RAD: f32 = 16.;

const EMPTY_TILE_COLOR: Color = Color::RGB(40, 42, 54);
const OBSTACLE_COLOR: Color = Color::RGB(255, 184, 108);
const UNIT_COLOR: Color = Color::RGB(255, 121, 198);
const UNIT_SELECTED_COLOR: Color = Color::RGB(80, 250, 123);
const UNIT_MOVING_COLOR: Color = Color::RGB(189, 147, 249);
const DRAG_PERIMETER_COLOR: Color = Color::RGB(0, 255, 0);

const SPRITE_SHEET_PATH: &str = "media/sprite-sheet.sps";

struct State<'a> {
    running: bool,
    game: game::State,
    drag_state: DragState,
    sprite_sheet: SpriteSheet<'a>,
    camera_pos: WorldPoint,
}

enum DragState {
    None,
    BoxSelect(BoxSelect),
    CameraDrag,
}

struct BoxSelect {
    from_x: i32,
    from_y: i32,
    to_x: i32,
    to_y: i32,
}

impl BoxSelect {
    fn resolve(&self, final_x: i32, final_y: i32, game: &mut game::State) {
        let rect = rect_from_points(
            self.from_x, self.from_y, final_x, final_y);
        for unit in game.units.iter_mut() {
            unit.selected = rect.has_intersection(
                Rect::new(
                    (unit.pos.x - OBJ_RAD) as i32,
                    (unit.pos.y - OBJ_RAD) as i32,
                    (OBJ_RAD * 2.) as u32,
                    (OBJ_RAD * 2.) as u32
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

    let mut canvas = window.into_canvas().build().unwrap();
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

    let state = State {
        running: true,
        game: game::State::new(),
        drag_state: DragState::None,
        sprite_sheet: sprite_sheet,
        camera_pos: WorldPoint::new(0., 0.),
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
            state.drag_state = DragState::BoxSelect(BoxSelect {
                from_x: x,
                from_y: y,
                to_x: x,
                to_y: y,
            });
        },
        Event::MouseButtonUp {x, y, mouse_btn: MouseButton::Left, ..} => {
            // Select units that are in the box.
            if let DragState::BoxSelect(box_select) = &state.drag_state {
                box_select.resolve(x, y, &mut state.game);
            }
            state.drag_state = DragState::None;
        },

        // Right mouse button -- issue move command.
        Event::MouseButtonDown {x, y, mouse_btn: MouseButton::Right, ..} => {
            for unit in state.game.units.iter_mut() {
                if unit.selected {
                    unit.move_target = Some(
                        WorldPoint::new(x as f32, y as f32));
                }
            }
        },

        // Middle mouse down/up: drag view.
        Event::MouseButtonDown {x, y, mouse_btn: MouseButton::Middle, .. } => {
            // End box-select if you middle mouse click-n-drag.
            if let DragState::BoxSelect(box_select) = &state.drag_state {
                box_select.resolve(x, y, &mut state.game);
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
                    box_select.to_x = x;
                    box_select.to_y = y;
                },
                DragState::CameraDrag => {
                    state.camera_pos.x -= xrel as f32;
                    state.camera_pos.y -= yrel as f32;
                },
                DragState::None => {},
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
        let wind_x = (tile.x * TILE_WIDTH) as i32 - state.camera_pos.x as i32;
        let wind_y = (tile.y * TILE_WIDTH) as i32 - state.camera_pos.y as i32;
        let _ = canvas.fill_rect(
            Rect::new(wind_x, wind_y, TILE_WIDTH, TILE_WIDTH));
    }

    // Draw units.
    for unit in state.game.units.iter() {
        let rad = OBJ_RAD;
        let wind_x = (unit.pos.x - rad - state.camera_pos.x) as i32;
        let wind_y = (unit.pos.y - rad - state.camera_pos.y) as i32;
        let dst = Rect::new(
            wind_x, wind_y,
            (2.*rad) as u32, (2.*rad) as u32
        );

        // Draw unit.
        let _ = state.sprite_sheet.blit_sprite_to_rect(
            unit.sprite_key.as_str(), canvas, dst);

        // Draw debug box around the unit.
        canvas.set_draw_color(
            if unit.selected { UNIT_SELECTED_COLOR }
            else if unit.move_target.is_some() { UNIT_MOVING_COLOR }
            else { UNIT_COLOR }
        );
        let _ = canvas.draw_rect(dst);
    }

    // Draw box-selection box.
    if let DragState::BoxSelect(box_select) = &state.drag_state {
        canvas.set_draw_color(DRAG_PERIMETER_COLOR);
        let _ = canvas.draw_rect(rect_from_points(
            box_select.from_x, box_select.from_y,
            box_select.to_x, box_select.to_y,
        ));
    }
}

fn rect_from_points(x1: i32, y1: i32, x2: i32, y2: i32) -> Rect {
    let xmin = i32::min(x1, x2);
    let xmax = i32::max(x1, x2);
    let ymin = i32::min(y1, y2);
    let ymax = i32::max(y1, y2);
    Rect::new(
        xmin, ymin,
        (xmax-xmin) as u32, (ymax-ymin) as u32)
}
