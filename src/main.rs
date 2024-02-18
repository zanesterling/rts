mod game;
mod sprite_sheet;

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

use crate::sprite_sheet::SpriteSheet;

const OBJ_RAD: f32 = 16.;
const BG_COLOR: Color = Color::RGB(40, 42, 54);
const UNIT_COLOR: Color = Color::RGB(255, 121, 198);
const UNIT_SELECTED_COLOR: Color = Color::RGB(80, 250, 123);
const UNIT_MOVING_COLOR: Color = Color::RGB(189, 147, 249);
const DRAG_PERIMETER_COLOR: Color = Color::RGB(0, 255, 0);
const SPRITE_SHEET_PATH: &str = "media/sprite-sheet.sps";
struct State<'a> {
    running: bool,
    game: game::State,
    drag_state: Option<DragState>,
    sprite_sheet: SpriteSheet<'a>,
}

struct DragState {
    from_x: i32,
    from_y: i32,
    to_x: i32,
    to_y: i32,
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

    canvas.set_draw_color(BG_COLOR);
    canvas.clear();
    canvas.present();
    let state = State {
        running: true,
        game: game::State::new(),
        drag_state: None,
        sprite_sheet: sprite_sheet,
    };
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
        Event::Quit {..} |
        Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
            state.running = false;
        },

        // R key: reset
        Event::KeyDown { keycode: Some(Keycode::R), .. } => {
            canvas.set_draw_color(BG_COLOR);
            canvas.clear();
        },

        // Left mouse down / up: box select.
        Event::MouseButtonDown {x, y, mouse_btn: MouseButton::Left, ..} => {
            state.drag_state = Some(DragState {
                from_x: x,
                from_y: y,
                to_x: x,
                to_y: y,
            });
        },
        Event::MouseButtonUp {x, y, mouse_btn: MouseButton::Left, ..} => {
            // Select units that are in the box.
            if let Some(drag_state) = &state.drag_state {
                let rect = rect_from_points(
                    drag_state.from_x, drag_state.from_y, x, y);
                for unit in state.game.units.iter_mut() {
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
            state.drag_state = None;
        },

        // Right mouse button -- issue move command.
        Event::MouseButtonDown {x, y, mouse_btn: MouseButton::Right, ..} => {
            for unit in state.game.units.iter_mut() {
                if unit.selected {
                    unit.move_target = Some(
                        game::Point::new(x as f32, y as f32));
                }
            }
        },

        Event::MouseMotion {x, y, mousestate, ..} => {
            if mousestate.left() {
                match &mut state.drag_state {
                    Some(drag_state) => {
                        drag_state.to_x = x;
                        drag_state.to_y = y;
                    },
                    None => {},
                }
            }
        },
        _ => {},
    }
}

fn render(canvas: &mut Canvas<Window>, state: &State) {
    canvas.set_draw_color(BG_COLOR);
    canvas.clear();

    // Draw units.
    for unit in state.game.units.iter() {
        let rad = OBJ_RAD;
        let dst = Rect::new(
            (unit.pos.x - rad) as i32, (unit.pos.y - rad) as i32,
            (2.*OBJ_RAD) as u32, (2.*OBJ_RAD) as u32
        );
        let _ = state.sprite_sheet.blit_sprite_to_rect(
            unit.sprite_key.as_str(), canvas, dst);

        canvas.set_draw_color(
            if unit.selected { UNIT_SELECTED_COLOR }
            else if unit.move_target.is_some() { UNIT_MOVING_COLOR }
            else { UNIT_COLOR }
        );
        let _ = canvas.draw_rect(Rect::new(
            (unit.pos.x - OBJ_RAD) as i32, (unit.pos.y - OBJ_RAD) as i32,
            (2.*OBJ_RAD) as u32, (2.*OBJ_RAD) as u32
        ));
    }

    // Draw box-selection box.
    match &state.drag_state {
        Some(drag_state) => {
            canvas.set_draw_color(DRAG_PERIMETER_COLOR);
            let _ = canvas.draw_rect(rect_from_points(
                drag_state.from_x, drag_state.from_y,
                drag_state.to_x, drag_state.to_y,
            ));
        },
        None => {},
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