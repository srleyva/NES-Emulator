mod apu;
pub mod bus;
pub mod cpu;
mod gamepad;
mod ppu;
pub mod rom;

#[macro_use]
extern crate bitflags;

use bus::MemoryBus;
use cpu::{processor_status::ProcessorStatus, CPU};

use rom::Rom;
use sdl2::{event::Event, keyboard::Keycode, pixels::Color, pixels::PixelFormatEnum, EventPump};

fn handle_user_input(cpu: &mut CPU, event_pump: &mut EventPump) {
    for event in event_pump.poll_iter() {
        if cfg!(debug_assertions) {
            println!("{:?}", event);
        }
        match event {
            Event::Quit { .. }
            | Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => std::process::exit(0),
            Event::KeyDown {
                keycode: Some(Keycode::W),
                ..
            } => {
                cpu.bus.write_byte(0xff, 0x77);
            }
            Event::KeyDown {
                keycode: Some(Keycode::S),
                ..
            } => {
                cpu.bus.write_byte(0xff, 0x73);
            }
            Event::KeyDown {
                keycode: Some(Keycode::A),
                ..
            } => {
                cpu.bus.write_byte(0xff, 0x61);
            }
            Event::KeyDown {
                keycode: Some(Keycode::D),
                ..
            } => {
                cpu.bus.write_byte(0xff, 0x64);
            }
            _ => { /* do nothing */ }
        }
    }
}

fn color(byte: u8) -> Color {
    match byte {
        0 => sdl2::pixels::Color::BLACK,
        1 => sdl2::pixels::Color::WHITE,
        2 | 9 => sdl2::pixels::Color::GREY,
        3 | 10 => sdl2::pixels::Color::RED,
        4 | 11 => sdl2::pixels::Color::GREEN,
        5 | 12 => sdl2::pixels::Color::BLUE,
        6 | 13 => sdl2::pixels::Color::MAGENTA,
        7 | 14 => sdl2::pixels::Color::YELLOW,
        _ => sdl2::pixels::Color::CYAN,
    }
}

fn read_screen_state(cpu: &mut CPU, frame: &mut [u8; 32 * 3 * 32]) -> bool {
    let mut frame_idx = 0;
    let mut update = false;
    for i in 0x0200..0x600 {
        let color_idx = cpu.bus.read_byte(i as u16);
        let (b1, b2, b3) = color(color_idx).rgb();
        if frame[frame_idx] != b1 || frame[frame_idx + 1] != b2 || frame[frame_idx + 2] != b3 {
            frame[frame_idx] = b1;
            frame[frame_idx + 1] = b2;
            frame[frame_idx + 2] = b3;
            update = true;
        }
        frame_idx += 3;
    }
    update
}

pub fn start_game_from_rom_path(path: String) {
    let rom = Rom::from_path(path).unwrap();
    start_game_from_rom(rom);
}

pub fn start_game_from_rom(rom: Rom) {
    // init sdl2
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Snake game", (32.0 * 10.0) as u32, (32.0 * 10.0) as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let _event_pump = sdl_context.event_pump().unwrap();
    canvas.set_scale(10.0, 10.0).unwrap();

    let creator = canvas.texture_creator();
    let _texture = creator
        .create_texture_target(PixelFormatEnum::RGB24, 32, 32)
        .unwrap();

    let mut cpu = CPU::new_with_state(
        MemoryBus::new(rom),
        0xC000,
        0xFD,
        1,
        2,
        3,
        ProcessorStatus::default(),
    );

    let _screen_state = [0_u8; 32 * 3 * 32];
    let _rng = rand::thread_rng();

    cpu.start_with_callback(move |_cpu, _instruction| {
        // handle_user_input(cpu, &mut event_pump);

        // cpu.bus.write_byte(0xfe, rng.gen_range(1..16));

        // if read_screen_state(cpu, &mut screen_state) {
        //     let tile_frame = cpu.bus.ppu.show_tile(1, 0);

        //     texture.update(None, &tile_frame.data, 256 * 3).unwrap();

        //     canvas.copy(&texture, None, None).unwrap();

        //     canvas.present();
        // }

        ::std::thread::sleep(std::time::Duration::from_secs(2));
    });
}
