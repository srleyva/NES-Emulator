extern crate nes;

use nes::start_game_from_rom_path;
use std::env;

fn main() {
    let rom_path = env::args().nth(1).unwrap();
    start_game_from_rom_path(rom_path)
}
