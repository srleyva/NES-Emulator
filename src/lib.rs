mod apu;
mod bus;
mod cpu;
mod gamepad;
mod ppu;
mod rom;

#[macro_use]
extern crate bitflags;

use bus::MemoryBus;
use cpu::CPU;
use ppu::frame::Frame;
use rand::Rng;
use rom::Rom;
use sdl2::{event::Event, keyboard::Keycode, pixels::Color, pixels::PixelFormatEnum, EventPump};
use std::sync::mpsc;

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

pub fn start_game_from_rom_path(path: String) {
    let rom = Rom::from_path(path).unwrap();
    start_game_from_rom(rom);
}

pub fn start_game_from_rom(rom: Rom) {
    let (nmi_send, nmi_recv) = mpsc::channel();
    // init sdl2
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("NES", (256.0 * 3.0) as u32, (240.0 * 3.0) as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    canvas.set_scale(3.0, 3.0).unwrap();

    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_target(PixelFormatEnum::RGB24, 256, 240)
        .unwrap();

    let mut cpu = CPU::new(MemoryBus::new(rom, nmi_send), nmi_recv);
    cpu.reset_cpu();

    let right_bank = Frame::show_tile_bank(cpu.bus.get_chr_rom(), 1);

    texture.update(None, &right_bank.data, 256 * 3).unwrap();
    canvas.copy(&texture, None, None).unwrap();
    canvas.present();

    cpu.start_with_callback(move |cpu| {
        handle_user_input(cpu, &mut event_pump);
        ::std::thread::sleep(std::time::Duration::new(0, 70_000));
    });
}
