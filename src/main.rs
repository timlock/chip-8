use std::fs;
use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

use chip8_emulator::{Chip8, DISPLAY_HEIGHT, DISPLAY_WIDTH};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;


fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("chip8-emulator", WIDTH, HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .map_err(|err| err.to_string())?;

    let mut canvas = window
        .into_canvas()
        .build()
        .map_err(|err| err.to_string())?;

    let mut emulator = Chip8::new(30)?;
    let rom = fs::read("roms/IBM Logo.ch8").map_err(|err| err.to_string())?;

    emulator.load_program(&rom)?;

    let mut event_pump = sdl_context.event_pump()?;
    'game: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    break 'game;
                }
                Event::KeyDown {
                    timestamp,
                    window_id,
                    keycode,
                    scancode,
                    keymod,
                    repeat
                } => {
                    println!("Key down {:?}", event);

                    let key = match scancode.unwrap() {
                        Scancode::A => 'a',
                        Scancode::C => 'c',
                        Scancode::D => 'd',
                        Scancode::E => 'e',
                        Scancode::F => 'f',
                        Scancode::Q => 'q',
                        Scancode::R => 'r',
                        Scancode::S => 's',
                        Scancode::W => 'w',
                        Scancode::X => 'x',
                        Scancode::Y => 'y',
                        Scancode::Z => 'z',
                        Scancode::Num1 => '1',
                        Scancode::Num2 => '2',
                        Scancode::Num3 => '3',
                        Scancode::Num4 => '4',
                        Scancode::Escape => {
                            break 'game;
                        }
                        _ => { ' ' }
                    };

                    emulator.on_input(key)
                }
                Event::KeyUp {
                    timestamp,
                    window_id,
                    keycode,
                    scancode,
                    keymod,
                    repeat
                } => {
                    println!("Key up {:?}", event);
                }
                _ => {}
            }
            emulator.update()?;
            let pixels = emulator.screen();
            canvas.set_draw_color(Color::BLACK);
            canvas.clear();

            for i in 0..pixels.len() {
                let pixel = pixels[i];
                match pixel {
                    true => canvas.set_draw_color(Color::WHITE),
                    false => canvas.set_draw_color(Color::BLACK)
                }

                let y = (i / DISPLAY_HEIGHT) as i32;
                let x = (i % DISPLAY_WIDTH) as i32;
                let rect = Rect::new(x, y, 10, 10);
                canvas.fill_rect(rect)?;
            }

            canvas.present();
        }
    }
    Ok(())
}


// use sdl2::event::Event;
// use sdl2::keyboard::Keycode;
// use sdl2::pixels::Color;
// use sdl2::rect::Rect;
// use sdl2::render::Canvas;
// use sdl2::video::Window;
//
// const SCALE: u32 = 15;
// const WINDOW_WIDTH: u32 = (600 as u32) * SCALE;
// const WINDOW_HEIGHT: u32 = (800 as u32) * SCALE;
// const TICKS_PER_FRAME: usize = 10;
//
// fn main() {
//     // Setup SDL
//     let sdl_context = sdl2::init().unwrap();
//     let video_subsystem = sdl_context.video().unwrap();
//     let window = video_subsystem
//         .window("Chip-8 Emulator", WINDOW_WIDTH, WINDOW_HEIGHT)
//         .position_centered()
//         .opengl()
//         .build()
//         .unwrap();
//
//     let mut canvas = window.into_canvas().present_vsync().build().unwrap();
//     canvas.clear();
//     canvas.present();
//
//     let mut event_pump = sdl_context.event_pump().unwrap();
//

//     'gameloop: loop {
//         for evt in event_pump.poll_iter() {
//             match evt {
//                 Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
//                     break 'gameloop;
//                 }
//                 Event::KeyDown { keycode: Some(key), .. } => {
//                     // if let Some(k) = key2btn(key) {
//                     //     chip8.keypress(k, true);
//                     // }
//                 }
//                 Event::KeyUp { keycode: Some(key), .. } => {
//                     // if let Some(k) = key2btn(key) {
//                     //     chip8.keypress(k, false);
//                     // }
//                 }
//                 _ => ()
//             }
//         }
//
//         // for _ in 0..TICKS_PER_FRAME {
//         //     chip8.tick();
//         // }
//         // chip8.tick_timers();
//         draw_screen(&mut canvas);
//     }
// }
//
// fn draw_screen(canvas: &mut Canvas<Window>) {
//     // Clear canvas as black
//     canvas.set_draw_color(Color::RGB(0, 0, 0));
//     canvas.clear();
//
//     let screen_buf: [bool; 600 * 800] = [true; 600 * 800];
//     // Now set draw color to white, iterate through each point and see if it should be drawn
//     canvas.set_draw_color(Color::RGB(255, 255, 255));
//     for (i, pixel) in screen_buf.iter().enumerate() {
//         if *pixel {
//             // Convert our 1D array's index into a 2D (x,y) position
//             let x = (i % 600) as u32;
//             let y = (i / 800) as u32;
//
//             // Draw a rectangle at (x,y), scaled up by our SCALE value
//             let rect = Rect::new((x * SCALE) as i32, (y * SCALE) as i32, SCALE, SCALE);
//             canvas.fill_rect(rect).unwrap();
//         }
//     }
//     canvas.present();
// }