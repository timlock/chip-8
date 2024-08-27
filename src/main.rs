use std::fs;

use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

use chip8_emulator::{Chip8, DISPLAY_HEIGHT, DISPLAY_WIDTH};

const SCALE: u32 = 10;
const WIDTH: u32 = DISPLAY_WIDTH as u32 * SCALE;
const HEIGHT: u32 = DISPLAY_HEIGHT as u32 * SCALE;
const TICKS: usize = 10;

const DEBUG: bool = false;

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

    canvas.clear();
    canvas.present();

    let mut emulator = Chip8::new(TICKS, DEBUG)?;
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
                    scancode,
                    ..
                } => {
                    println!("Key down {:?}", event);
                    if let Scancode::Escape = scancode.unwrap() {
                        break 'game;
                    }
                    if let Ok(key) = scancode_to_char(scancode.unwrap()) {
                        emulator.on_input(key, true);
                    }
                }
                Event::KeyUp {
                    scancode,
                    ..
                } => {
                    println!("Key up {:?}", event);
                    if let Scancode::Escape = scancode.unwrap() {
                        break 'game;
                    }
                    if let Ok(key) = scancode_to_char(scancode.unwrap()) {
                        emulator.on_input(key, false);
                    }
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

                let y = (i / DISPLAY_WIDTH) as i32;
                let x = (i % DISPLAY_WIDTH) as i32;
                let rect = Rect::new(x * SCALE as i32, y * SCALE as i32, SCALE, SCALE);
                if pixel && DEBUG{
                    println!("Box x:{x} y:{y}");
                }
                canvas.fill_rect(rect)?;
            }

            canvas.present();
        }
    }
    Ok(())
}

fn scancode_to_char(scancode: Scancode) -> Result<char, String> {
    let key = match scancode {
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
        _ => return Err(format!("invalid key input: {}", scancode.name()))
    };
    Ok(key)
}