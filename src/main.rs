use macroquad::prelude::*;

mod tile;
mod game_state;
mod inventory;
mod garden;
mod shop;
use game_state::{GameState, BiomeTextures};

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

    // Helper: load a texture, falling back to a default if the file doesn't exist
    async fn load_or_fallback(path: &str, fallback: &str) -> Texture2D {
        match load_texture(path).await {
            Ok(t) => { t.set_filter(FilterMode::Nearest); t }
            Err(_) => {
                let t = load_texture(fallback).await.unwrap();
                t.set_filter(FilterMode::Nearest);
                t
            }
        }
    }

    async fn load_optional_texture(path: &str) -> Option<Texture2D> {
        match load_texture(path).await {
            Ok(t) => {
                t.set_filter(FilterMode::Nearest);
                Some(t)
            }
            Err(_) => None,
        }
    }

    // Biome 0 — Forest Floor (base assets)
    let b0_sun    = load_texture("assets/beryl_sheet.png").await.unwrap();  b0_sun.set_filter(FilterMode::Nearest);
    let b0_moon   = load_texture("assets/moon_sheet.png").await.unwrap();   b0_moon.set_filter(FilterMode::Nearest);
    let b0_leaf   = load_texture("assets/leaf_sheet.png").await.unwrap();   b0_leaf.set_filter(FilterMode::Nearest);
    let b0_exotic = load_texture("assets/exotic_sheet.png").await.unwrap(); b0_exotic.set_filter(FilterMode::Nearest);
    let b0_water  = load_texture("assets/water_sheet.png").await.unwrap();  b0_water.set_filter(FilterMode::Nearest);
    let b0_overlay = load_optional_texture("assets/forest_overlay.PNG").await;

    // Additional biomes — each slot falls back to biome 0 if assets aren't ready yet.
    // Add new biomes here as you produce them; just increment the list.
    // Biome 1 — Deep Cave
    let b1_sun    = load_or_fallback("assets/biome1/sun_sheet.png",    "assets/beryl_sheet.png").await;
    let b1_moon   = load_or_fallback("assets/biome1/moon_sheet.png",   "assets/moon_sheet.png").await;
    let b1_leaf   = load_or_fallback("assets/biome1/leaf_sheet.png",   "assets/leaf_sheet.png").await;
    let b1_exotic = load_or_fallback("assets/biome1/exotic_sheet.png", "assets/exotic_sheet.png").await;
    let b1_water  = load_or_fallback("assets/biome1/water_sheet.png",  "assets/water_sheet.png").await;

    // Biome 2 — Volcanic Rift
    let b2_sun    = load_or_fallback("assets/biome2/sun_sheet.png",    "assets/beryl_sheet.png").await;
    let b2_moon   = load_or_fallback("assets/biome2/moon_sheet.png",   "assets/moon_sheet.png").await;
    let b2_leaf   = load_or_fallback("assets/biome2/leaf_sheet.png",   "assets/leaf_sheet.png").await;
    let b2_exotic = load_or_fallback("assets/biome2/exotic_sheet.png", "assets/exotic_sheet.png").await;
    let b2_water  = load_or_fallback("assets/biome2/water_sheet.png",  "assets/water_sheet.png").await;

    // Biome 3 — Frozen Tundra
    let b3_sun    = load_or_fallback("assets/biome3/sun_sheet.png",    "assets/beryl_sheet.png").await;
    let b3_moon   = load_or_fallback("assets/biome3/moon_sheet.png",   "assets/moon_sheet.png").await;
    let b3_leaf   = load_or_fallback("assets/biome3/leaf_sheet.png",   "assets/leaf_sheet.png").await;
    let b3_exotic = load_or_fallback("assets/biome3/exotic_sheet.png", "assets/exotic_sheet.png").await;
    let b3_water  = load_or_fallback("assets/biome3/water_sheet.png",  "assets/water_sheet.png").await;

    // Biome 4 — Ocean Trench
    let b4_sun    = load_or_fallback("assets/biome4/sun_sheet.png",    "assets/beryl_sheet.png").await;
    let b4_moon   = load_or_fallback("assets/biome4/moon_sheet.png",   "assets/moon_sheet.png").await;
    let b4_leaf   = load_or_fallback("assets/biome4/leaf_sheet.png",   "assets/leaf_sheet.png").await;
    let b4_exotic = load_or_fallback("assets/biome4/exotic_sheet.png", "assets/exotic_sheet.png").await;
    let b4_water  = load_or_fallback("assets/biome4/water_sheet.png",  "assets/water_sheet.png").await;

    // Biome 5 — Sky Realm
    let b5_sun    = load_or_fallback("assets/biome5/sun_sheet.png",    "assets/beryl_sheet.png").await;
    let b5_moon   = load_or_fallback("assets/biome5/moon_sheet.png",   "assets/moon_sheet.png").await;
    let b5_leaf   = load_or_fallback("assets/biome5/leaf_sheet.png",   "assets/leaf_sheet.png").await;
    let b5_exotic = load_or_fallback("assets/biome5/exotic_sheet.png", "assets/exotic_sheet.png").await;
    let b5_water  = load_or_fallback("assets/biome5/water_sheet.png",  "assets/water_sheet.png").await;

    // Biome 6 — Fungal Wastes
    let b6_sun    = load_or_fallback("assets/biome6/sun_sheet.png",    "assets/beryl_sheet.png").await;
    let b6_moon   = load_or_fallback("assets/biome6/moon_sheet.png",   "assets/moon_sheet.png").await;
    let b6_leaf   = load_or_fallback("assets/biome6/leaf_sheet.png",   "assets/leaf_sheet.png").await;
    let b6_exotic = load_or_fallback("assets/biome6/exotic_sheet.png", "assets/exotic_sheet.png").await;
    let b6_water  = load_or_fallback("assets/biome6/water_sheet.png",  "assets/water_sheet.png").await;

    // Biome 7 — Celestial Void
    let b7_sun    = load_or_fallback("assets/biome7/sun_sheet.png",    "assets/beryl_sheet.png").await;
    let b7_moon   = load_or_fallback("assets/biome7/moon_sheet.png",   "assets/moon_sheet.png").await;
    let b7_leaf   = load_or_fallback("assets/biome7/leaf_sheet.png",   "assets/leaf_sheet.png").await;
    let b7_exotic = load_or_fallback("assets/biome7/exotic_sheet.png", "assets/exotic_sheet.png").await;
    let b7_water  = load_or_fallback("assets/biome7/water_sheet.png",  "assets/water_sheet.png").await;

    let biome_sets = vec![
        BiomeTextures { sun: b0_sun, moon: b0_moon, leaf: b0_leaf, exotic: b0_exotic, water: b0_water, overlay: b0_overlay },
        BiomeTextures { sun: b1_sun, moon: b1_moon, leaf: b1_leaf, exotic: b1_exotic, water: b1_water, overlay: None },
        BiomeTextures { sun: b2_sun, moon: b2_moon, leaf: b2_leaf, exotic: b2_exotic, water: b2_water, overlay: None },
        BiomeTextures { sun: b3_sun, moon: b3_moon, leaf: b3_leaf, exotic: b3_exotic, water: b3_water, overlay: None },
        BiomeTextures { sun: b4_sun, moon: b4_moon, leaf: b4_leaf, exotic: b4_exotic, water: b4_water, overlay: None },
        BiomeTextures { sun: b5_sun, moon: b5_moon, leaf: b5_leaf, exotic: b5_exotic, water: b5_water, overlay: None },
        BiomeTextures { sun: b6_sun, moon: b6_moon, leaf: b6_leaf, exotic: b6_exotic, water: b6_water, overlay: None },
        BiomeTextures { sun: b7_sun, moon: b7_moon, leaf: b7_leaf, exotic: b7_exotic, water: b7_water, overlay: None },
    ];

    let garden_bg_texture = load_texture("assets/gardenbgplots.png").await.unwrap();
    garden_bg_texture.set_filter(FilterMode::Nearest);

    // 2. PASS IT TO THE GAME
    let mut game = GameState::new(
        biome_sets,
        garden_bg_texture,
    );

    loop {
        game.update();
        game.draw();
        next_frame().await
    }
}