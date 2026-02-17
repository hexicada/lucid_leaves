use macroquad::prelude::*;

mod tile;
mod game_state;
use game_state::GameState;

#[macroquad::main("Lucid Leaves")]
async fn main() {
    rand::srand(macroquad::miniquad::date::now() as u64);

    // 1. LOAD THE ASSET
    // We use "nearest" filter so pixels stay sharp when scaled up
    let beryl_texture = load_texture("assets/beryl_sheet.png").await.unwrap();
    beryl_texture.set_filter(FilterMode::Nearest);

    // 2. PASS IT TO THE GAME
    let mut game = GameState::new(beryl_texture);

    loop {
        game.update();
        game.draw();
        next_frame().await
    }
}