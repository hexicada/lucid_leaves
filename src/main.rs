use macroquad::prelude::*;

mod tile;
mod game_state;
mod inventory;
mod garden;
mod shop;
use game_state::GameState;

fn window_conf() -> Conf {
    Conf {
        window_title: "Lucid Leaves".to_owned(),
        window_width: 800,
        window_height: 580,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    rand::srand(macroquad::miniquad::date::now() as u64);

    // 1. LOAD THE ASSETS
    // We use "nearest" filter so pixels stay sharp when scaled up
    let beryl_texture = load_texture("assets/beryl_sheet.png").await.unwrap();
    beryl_texture.set_filter(FilterMode::Nearest);
    let moon_texture = load_texture("assets/moon_sheet.png").await.unwrap();
    moon_texture.set_filter(FilterMode::Nearest);
    let leaf_texture = load_texture("assets/leaf_sheet.png").await.unwrap();
    leaf_texture.set_filter(FilterMode::Nearest);
    let exotic_texture = load_texture("assets/exotic_sheet.png").await.unwrap();
    exotic_texture.set_filter(FilterMode::Nearest);
    let water_texture = load_texture("assets/water_sheet.png").await.unwrap();
    water_texture.set_filter(FilterMode::Nearest);
    let garden_bg_texture = load_texture("assets/gardenbgplots.png").await.unwrap();
    garden_bg_texture.set_filter(FilterMode::Nearest);

    // 2. PASS IT TO THE GAME
    let mut game = GameState::new(
        beryl_texture,
        moon_texture,
        leaf_texture,
        exotic_texture,
        water_texture,
        garden_bg_texture,
    );

    loop {
        game.update();
        game.draw();
        next_frame().await
    }
}