use std::{fs, path::PathBuf};

use words_core::Game;
use words_render::{day, seed, View};

fn main() -> Result<(), String> {
    let dir = PathBuf::from("/mnt/us/words");
    let data = dir.join("rounds.json");
    let game = Game::restore(&fs::read(&data).unwrap_or_default(), day(), seed());
    primitive_app::run(View::new(game, Some(data)), &dir)
}
