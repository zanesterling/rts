extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use std::thread::sleep;
use std::time::Duration;

const OBJ_RAD: u32 = 20;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("rts!", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut obj_x = 0;
    let mut obj_y = 0;
    'running: loop {
        canvas.set_draw_color(Color::RGB(40, 42, 54));
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::MouseButtonDown {x, y, ..} => {
                    obj_x = x;
                    obj_y = y;
                },
                Event::MouseMotion { mousestate, x, y, ..} => {
                    if mousestate.left() {
                        obj_x = x;
                        obj_y = y;
                    }
                },
                _ => {},
            }
        }

        canvas.set_draw_color(Color::RGB(255, 121, 198));
        let _ = canvas.fill_rect(Rect::new(
            obj_x - OBJ_RAD as i32, obj_y - OBJ_RAD as i32,
            2*OBJ_RAD, 2*OBJ_RAD));
        canvas.present();
        sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
