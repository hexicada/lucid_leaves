use macroquad::prelude::*;

// Tell Rust to look for these two files
mod tile;
mod game_state;

// Bring the GameState struct into scope so we can use it
use game_state::GameState;

#[macroquad::main("Lucid Leaves")]
async fn main() {
    rand::srand(macroquad::miniquad::date::now() as u64);
    
    let mut game = GameState::new();

    loop {
        // 1. Update the Brain
        game.update();
        
        // 2. Draw the Screen
        game.draw();

        next_frame().await
    }
}