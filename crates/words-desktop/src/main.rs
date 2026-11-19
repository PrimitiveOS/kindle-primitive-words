use std::{
    fs,
    path::{Path, PathBuf},
};

use primitive_app::{font_paths, frame, write_png, HEIGHT, WIDTH};
use primitive_ui::Canvas;
use words_core::Game;
use words_render::View;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = PathBuf::from(
        std::env::args_os()
            .nth(1)
            .unwrap_or_else(|| "build/previews".into()),
    );
    fs::create_dir_all(&output)?;
    let fonts =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../doannt-kindle-launcher/assets/fonts");
    let mut canvas = Canvas::new(WIDTH, HEIGHT, &font_paths(&fonts))?;
    let mut game = Game::restore(b"", 20_000, 6);
    game.daily.draft = "stare".into();
    game.submit().unwrap();
    game.daily.draft = "plane".into();
    game.submit().unwrap();
    let app = View::new(game, None);
    write_png(
        &output.join("words-daily.png"),
        WIDTH,
        HEIGHT,
        &frame(&app, &mut canvas, false),
    )?;
    write_png(
        &output.join("words-dark.png"),
        WIDTH,
        HEIGHT,
        &frame(&app, &mut canvas, true),
    )?;
    Ok(())
}
