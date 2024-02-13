mod game;

extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use std::thread::sleep;
use std::time::Duration;

const OBJ_RAD: u32 = 20;
const BG_COLOR: Color = Color::RGB(40, 42, 54);
const UNIT_COLOR: Color = Color::RGB(255, 121, 198);
const UNIT_SELECTED_COLOR: Color = Color::RGB(80, 250, 123);

struct State {
    running: bool,
    game: game::State,
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("rts!", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(BG_COLOR);
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut state = State {
        running: true,
        game: game::State::new(),
    };
    while state.running {
        // Render.
        render(&mut canvas, &state.game);
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
        Event::Quit {..} |
        Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
            state.running = false;
        },
        Event::KeyDown { keycode: Some(Keycode::R), .. } => {
            canvas.set_draw_color(BG_COLOR);
            canvas.clear();
        },
        Event::MouseButtonDown {..} => {},
        Event::MouseButtonUp {..} => {},
        Event::MouseMotion {..} => {},
        _ => {},
    }
}

fn render(canvas: &mut Canvas<Window>, game_state: &game::State) {
    canvas.set_draw_color(BG_COLOR);
    canvas.clear();

    for unit in game_state.units.iter() {
        canvas.set_draw_color(
            if unit.selected { UNIT_SELECTED_COLOR } else { UNIT_COLOR }
        );
        let _ = canvas.fill_rect(Rect::new(
            unit.world_x - OBJ_RAD as i32, unit.world_y - OBJ_RAD as i32,
            2*OBJ_RAD, 2*OBJ_RAD
        ));
    }
}