use macroquad::prelude::*;

use crate::economy;
use crate::game_state::{
    GameState,
    GardenTool,
    GRID_HEIGHT,
    GRID_WIDTH,
    LEAF_AUX_SWAY_AMP_DEG,
    LEAF_AUX_SWAY_PHASE,
    LEAF_AUX_SWAY_PIVOT_NX,
    LEAF_AUX_SWAY_PIVOT_NY,
    LEAF_SWAY_AMP_DEG,
    LEAF_SWAY_PIVOT_NX,
    LEAF_SWAY_PIVOT_NY,
    LEAF_SWAY_SPEED,
    LEAF_SWAY_SPEED_2,
    LEVELS_PER_SET,
    MATCH_CLEAR_DELAY,
};
use crate::inventory::{Inventory, ItemType};
use crate::match_logic;
use crate::tile::TileType;
use crate::ui_layout::{
    garden_hunt_button_rect,
    garden_return_button_rect,
    hunt_return_button_rect,
    playing_descend_button_rect,
    playing_visit_garden_button_rect,
    Layout,
};

pub fn draw_shop_screen(leaves_wallet: i32) {
    let sw = screen_width();
    let sh = screen_height();
    draw_text("THE SHRINE", sw * 0.25, sh * 0.17, (sh * 0.103).max(40.0), PURPLE);
    draw_text(
        &format!("Leaves: {}", leaves_wallet),
        sw * 0.31,
        sh * 0.34,
        (sh * 0.069).max(28.0),
        GOLD,
    );
    draw_text(
        "[SPACE] Spend 500 | [ENTER] Next Biome",
        sw * 0.19,
        sh * 0.69,
        (sh * 0.052).max(22.0),
        WHITE,
    );
}

pub fn draw_garden_screen(
    garden_bg_texture: &Texture2D,
    iso_left_origin_nx: f32,
    iso_left_origin_ny: f32,
    iso_right_origin_nx: f32,
    iso_right_origin_ny: f32,
    iso_tile_hw: f32,
    iso_tile_hh: f32,
    iso_dot_radius: f32,
    inventory: &Inventory,
    selected_tool: Option<GardenTool>,
    drawer_open: bool,
) {
    let sw = screen_width();
    let sh = screen_height();
    draw_texture_ex(
        garden_bg_texture,
        0.0,
        0.0,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(sw, sh)),
            ..Default::default()
        },
    );

    let origins = [
        (sw * iso_left_origin_nx, sh * iso_left_origin_ny),
        (sw * iso_right_origin_nx, sh * iso_right_origin_ny),
    ];
    for (origin_x, origin_y) in origins {
        for gy in 0..5usize {
            for gx in 0..5usize {
                let sx = origin_x + (gx as f32 - gy as f32) * iso_tile_hw;
                let sy = origin_y + (gx as f32 + gy as f32) * iso_tile_hh;
                draw_circle(sx, sy, iso_dot_radius, color_u8!(116, 78, 40, 240));
            }
        }
    }

    draw_rectangle(0.0, 0.0, sw, sh * 0.12, color_u8!(14, 28, 14, 175));
    draw_text(
        "THE GARDEN",
        sw * 0.03,
        sh * 0.08,
        (sh * 0.07).max(30.0),
        color_u8!(220, 245, 185, 255),
    );
    draw_text(
        "Rest phase: tend plots and manage resources",
        sw * 0.25,
        sh * 0.08,
        (sh * 0.035).max(18.0),
        WHITE,
    );

    let (rx, ry, rw, rh) = garden_return_button_rect();
    draw_rectangle(rx, ry, rw, rh, color_u8!(52, 80, 58, 255));
    draw_rectangle_lines(rx, ry, rw, rh, 3.0, color_u8!(170, 225, 170, 255));
    draw_text(
        "RETURN TO PUZZLE",
        rx + rw * 0.08,
        ry + rh * 0.62,
        (rh * 0.42).max(20.0),
        WHITE,
    );

    let (hx, hy, hw, hh) = garden_hunt_button_rect();
    draw_rectangle(hx, hy, hw, hh, color_u8!(78, 65, 36, 255));
    draw_rectangle_lines(hx, hy, hw, hh, 3.0, color_u8!(240, 200, 120, 255));
    draw_text(
        "GO HUNT",
        hx + hw * 0.29,
        hy + hh * 0.62,
        (hh * 0.48).max(20.0),
        WHITE,
    );

    // Draw inventory drawer at bottom
    draw_garden_drawer(sw, sh, inventory, selected_tool, drawer_open);
}

fn draw_garden_drawer(
    sw: f32,
    sh: f32,
    inventory: &Inventory,
    selected_tool: Option<GardenTool>,
    drawer_open: bool,
) {
    let drawer_y = sh * 0.84;
    let drawer_h = sh * 0.16;
    let drawer_bg = color_u8!(40, 50, 40, 220);
    let drawer_border = color_u8!(120, 180, 120, 255);

    let tab_w = sw * 0.18;
    let tab_h = sh * 0.05;
    let tab_x = (sw - tab_w) * 0.5;
    let tab_y = if drawer_open { drawer_y - tab_h * 0.8 } else { sh * 0.95 - tab_h };

    if drawer_open {
        // Draw drawer background
        draw_rectangle(0.0, drawer_y, sw, drawer_h, drawer_bg);
        draw_rectangle_lines(0.0, drawer_y, sw, drawer_h, 3.0, drawer_border);
    }

    draw_rectangle(tab_x, tab_y, tab_w, tab_h, color_u8!(54, 74, 54, 240));
    draw_rectangle_lines(tab_x, tab_y, tab_w, tab_h, 2.0, drawer_border);
    draw_text(
        if drawer_open { "INVENTORY v" } else { "INVENTORY ^" },
        tab_x + tab_w * 0.13,
        tab_y + tab_h * 0.68,
        (tab_h * 0.55).max(14.0),
        color_u8!(220, 245, 185, 255),
    );

    if !drawer_open {
        return;
    }

    let font_sm = (drawer_h * 0.18).max(11.0);

    // Tool buttons
    let btn_size = (drawer_h * 0.52).min(sw * 0.09);
    let btn_start_y = drawer_y + drawer_h * 0.24;
    let spacing = sw * 0.012;

    let buttons = [
        ("CAN", GardenTool::Water, economy::inventory_count(inventory, ItemType::WateringCan), color_u8!(55, 125, 210, 255)),
        ("SUN", GardenTool::PlantSun, economy::inventory_count(inventory, ItemType::SeedDay), color_u8!(210, 165, 35, 255)),
        ("MON", GardenTool::PlantMoon, economy::inventory_count(inventory, ItemType::SeedNight), color_u8!(100, 125, 220, 255)),
        ("ESS", GardenTool::PlantEssence, economy::inventory_count(inventory, ItemType::MoonbloomEssence), color_u8!(160, 90, 220, 255)),
        ("FERT", GardenTool::Fertilize, economy::inventory_count(inventory, ItemType::Fertilizer), color_u8!(185, 55, 55, 255)),
    ];

    let total_w = buttons.len() as f32 * btn_size + (buttons.len() as f32 - 1.0) * spacing;
    let start_x = (sw - total_w) * 0.5;

    for (idx, (label, tool, count, color)) in buttons.iter().enumerate() {
        let btn_x = start_x + idx as f32 * (btn_size + spacing);
        let is_active = selected_tool == Some(*tool);
        let btn_color = if is_active {
            Color::new(color.r as f32 / 255.0, color.g as f32 / 255.0, color.b as f32 / 255.0, 1.0)
        } else if *count == 0 {
            color_u8!(80, 80, 80, 200)
        } else {
            *color
        };

        // Base slot tile (placeholder for future item art)
        draw_rectangle(btn_x, btn_start_y, btn_size, btn_size, btn_color);
        let border_color = if is_active { YELLOW } else { WHITE };
        let border_width = if is_active { 4.0 } else { 2.0 };
        draw_rectangle_lines(btn_x, btn_start_y, btn_size, btn_size, border_width, border_color);

        // Tiny neutral inset to suggest art frame area
        draw_rectangle(
            btn_x + btn_size * 0.18,
            btn_start_y + btn_size * 0.18,
            btn_size * 0.64,
            btn_size * 0.50,
            color_u8!(32, 38, 32, 160),
        );

        // Keep tiny code marker for now; easy to remove once icons are in.
        draw_text(
            label,
            btn_x + btn_size * 0.08,
            btn_start_y + btn_size * 0.92,
            (font_sm * 0.85).max(10.0),
            if *count == 0 { color_u8!(150, 150, 150, 255) } else { WHITE },
        );

        // Count badge in top-right corner
        let badge_w = btn_size * 0.28;
        let badge_h = btn_size * 0.28;
        let badge_x = btn_x + btn_size - badge_w - btn_size * 0.05;
        let badge_y = btn_start_y + btn_size * 0.05;
        draw_rectangle(badge_x, badge_y, badge_w, badge_h, color_u8!(18, 24, 18, 235));
        draw_rectangle_lines(badge_x, badge_y, badge_w, badge_h, 1.5, color_u8!(220, 235, 210, 255));
        draw_text(
            &format!("{}", count),
            badge_x + badge_w * 0.24,
            badge_y + badge_h * 0.74,
            (font_sm * 0.95).max(10.0),
            color_u8!(230, 245, 220, 255),
        );
    }

    draw_text("[I] Toggle", sw * 0.02, drawer_y + drawer_h * 0.66, (font_sm * 0.75).max(10.0), color_u8!(180, 200, 180, 190));
}

pub fn draw_hunt_screen() {
    let sw = screen_width();
    let sh = screen_height();
    clear_background(color_u8!(48, 24, 26, 255));
    draw_text(
        "HUNT (PLACEHOLDER)",
        sw * 0.19,
        sh * 0.20,
        (sh * 0.095).max(34.0),
        color_u8!(255, 210, 180, 255),
    );
    draw_text(
        "Wizard mice mini-game implementation is next sprint",
        sw * 0.13,
        sh * 0.34,
        (sh * 0.045).max(20.0),
        WHITE,
    );

    let (bx, by, bw, bh) = hunt_return_button_rect();
    draw_rectangle(bx, by, bw, bh, color_u8!(85, 45, 44, 255));
    draw_rectangle_lines(bx, by, bw, bh, 3.0, color_u8!(255, 190, 170, 255));
    draw_text(
        "BACK TO GARDEN",
        bx + bw * 0.11,
        by + bh * 0.62,
        (bh * 0.42).max(20.0),
        WHITE,
    );
}

pub fn draw_board_and_effects(state: &GameState, layout: &Layout) {
    // --- ANIMATION MATH (Global Time) ---
    let time = get_time() as f32;
    let speed = 13.0_f32; // 13 frames per second

    // Generic ping-pong helper for spritesheets with N horizontal frames.
    let ping_pong_frame = |frames: u32| -> f32 {
        if frames <= 1 {
            return 0.0;
        }
        let max = (frames - 1) as f32;
        let cycle = max * 2.0;
        let mut idx = (time * speed) % cycle;
        if idx > max {
            idx = cycle - idx;
        }
        idx.floor()
    };

    // For 13-frame sprites (Sun, Moon, Leaf): ping-pong cycle
    let current_frame_13 = ping_pong_frame(13);

    // For 32-frame sprite (Leaf): ping-pong cycle
    let total_frames_32 = 62.0; // 32 forward + 30 back = 62 total loop
    let mut frame_index_32 = (time * speed) % total_frames_32;
    if frame_index_32 > 31.0 {
        frame_index_32 = 62.0 - frame_index_32;
    }
    let current_frame_32 = frame_index_32.floor() as f32;

    // For 41-frame sprite (Exotic): simple loop
    let total_frames_41 = 41.0;
    let frame_index_41 = (time * speed) % total_frames_41;
    let current_frame_41 = frame_index_41.floor() as f32;

    // For 13-frame sprite (Water): ping-pong cycle
    let current_frame_water_13 = ping_pong_frame(13);

    // 1. DRAW GRID
    let set_idx = ((state.level - 1) / LEVELS_PER_SET) as usize;
    let is_cave_biome = set_idx == 1;
    let gems = &state.biome_sets[set_idx.min(state.biome_sets.len() - 1)];
    let clear_progress = if !state.pending_matches.is_empty() {
        (1.0 - state.clear_timer / MATCH_CLEAR_DELAY).clamp(0.0, 1.0)
    } else {
        0.0
    };

    for x in 0..GRID_WIDTH {
        for y in 0..GRID_HEIGHT {
            let tile = &state.grid[x][y];
            let draw_x = layout.grid_offset_x + x as f32 * layout.tile_size;
            let draw_y = layout.grid_offset_y + y as f32 * layout.tile_size + tile.offset_y;
            let matched = match_logic::pending_match_kind_at(&state.pending_matches, x, y).is_some();
            let scale = if matched { 1.0 - 0.18 * clear_progress } else { 1.0 };
            let alpha = if matched { 1.0 - clear_progress } else { 1.0 };
            let flash = if matched && clear_progress < 0.18 {
                1.0 - clear_progress / 0.18
            } else {
                0.0
            };

            if matched {
                let glow_color = match_logic::tile_particle_color(tile.kind);
                draw_circle(
                    draw_x + layout.tile_size * 0.5,
                    draw_y + layout.tile_size * 0.5,
                    layout.tile_size * (0.28 + 0.18 * flash),
                    Color::new(glow_color.r, glow_color.g, glow_color.b, 0.22 * flash),
                );
            }

            match tile.kind {
                TileType::Sun => {
                    let sprite_size = 64.0;
                    let scaled = layout.tile_size * scale;
                    let scaled_x = draw_x + (layout.tile_size - scaled) * 0.5;
                    let scaled_y = draw_y + (layout.tile_size - scaled) * 0.5;
                    draw_texture_ex(
                        &gems.sun,
                        scaled_x,
                        scaled_y,
                        Color::new(1.0, 1.0, 1.0, alpha),
                        DrawTextureParams {
                            dest_size: Some(vec2(scaled, scaled)),
                            source: Some(Rect::new(
                                current_frame_13 * sprite_size as f32,
                                0.0,
                                sprite_size as f32,
                                sprite_size as f32,
                            )),
                            ..Default::default()
                        },
                    );
                }
                TileType::Moon => {
                    let sprite_size = 64.0;
                    let moon_frame = if is_cave_biome {
                        ping_pong_frame(16)
                    } else {
                        current_frame_13
                    };
                    let scaled = layout.tile_size * scale;
                    let scaled_x = draw_x + (layout.tile_size - scaled) * 0.5;
                    let scaled_y = draw_y + (layout.tile_size - scaled) * 0.5;
                    draw_texture_ex(
                        &gems.moon,
                        scaled_x,
                        scaled_y,
                        Color::new(1.0, 1.0, 1.0, alpha),
                        DrawTextureParams {
                            dest_size: Some(vec2(scaled, scaled)),
                            source: Some(Rect::new(
                                moon_frame * sprite_size as f32,
                                0.0,
                                sprite_size as f32,
                                sprite_size as f32,
                            )),
                            ..Default::default()
                        },
                    );
                }
                TileType::Leaf => {
                    let sprite_size = 64.0;
                    let leaf_w = layout.tile_size * 1.25;
                    let scaled_w = leaf_w * scale;
                    let scaled_h = layout.tile_size * scale;
                    let leaf_x = draw_x - (leaf_w - layout.tile_size) * 0.5 + (leaf_w - scaled_w) * 0.5;
                    let leaf_y = draw_y + (layout.tile_size - scaled_h) * 0.5;
                    draw_texture_ex(
                        &gems.leaf,
                        leaf_x,
                        leaf_y,
                        Color::new(1.0, 1.0, 1.0, alpha),
                        DrawTextureParams {
                            dest_size: Some(vec2(scaled_w, scaled_h)),
                            source: Some(Rect::new(
                                current_frame_32 * sprite_size as f32,
                                0.0,
                                sprite_size as f32,
                                sprite_size as f32,
                            )),
                            ..Default::default()
                        },
                    );
                }
                TileType::Exotic => {
                    let sprite_size = 64.0;
                    let exotic_frame = if is_cave_biome {
                        ping_pong_frame(29)
                    } else {
                        current_frame_41
                    };
                    let scaled = layout.tile_size * scale;
                    let scaled_x = draw_x + (layout.tile_size - scaled) * 0.5;
                    let scaled_y = draw_y + (layout.tile_size - scaled) * 0.5;
                    draw_texture_ex(
                        &gems.exotic,
                        scaled_x,
                        scaled_y,
                        Color::new(1.0, 1.0, 1.0, alpha),
                        DrawTextureParams {
                            dest_size: Some(vec2(scaled, scaled)),
                            source: Some(Rect::new(
                                exotic_frame * sprite_size as f32,
                                0.0,
                                sprite_size as f32,
                                sprite_size as f32,
                            )),
                            ..Default::default()
                        },
                    );
                }
                TileType::Water => {
                    let sprite_size = 64.0;
                    let scaled = layout.tile_size * scale;
                    let scaled_x = draw_x + (layout.tile_size - scaled) * 0.5;
                    let scaled_y = draw_y + (layout.tile_size - scaled) * 0.5;
                    draw_texture_ex(
                        &gems.water,
                        scaled_x,
                        scaled_y,
                        Color::new(1.0, 1.0, 1.0, alpha),
                        DrawTextureParams {
                            dest_size: Some(vec2(scaled, scaled)),
                            source: Some(Rect::new(
                                current_frame_water_13 * sprite_size as f32,
                                0.0,
                                sprite_size as f32,
                                sprite_size as f32,
                            )),
                            ..Default::default()
                        },
                    );
                }
                _ => {
                    let base = tile.get_color(state.level);
                    draw_rectangle(
                        draw_x,
                        draw_y,
                        layout.tile_size - 2.0,
                        layout.tile_size - 2.0,
                        Color::new(base.r, base.g, base.b, alpha),
                    );
                }
            }
        }
    }

    // Draw overlay after tiles so frame art stays visible even with opaque gem sprites.
    if let Some(overlay) = &gems.overlay {
        draw_texture_ex(
            overlay,
            0.0,
            0.0,
            Color::new(1.0, 1.0, 1.0, 0.85),
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                ..Default::default()
            },
        );
    }

    // Keep selection feedback above the overlay.
    if let Some((sx, sy)) = state.selected {
        if match_logic::pending_match_kind_at(&state.pending_matches, sx, sy).is_none() {
            let draw_x = layout.grid_offset_x + sx as f32 * layout.tile_size;
            let draw_y = layout.grid_offset_y + sy as f32 * layout.tile_size + state.grid[sx][sy].offset_y;
            draw_rectangle_lines(draw_x, draw_y, layout.tile_size - 2.0, layout.tile_size - 2.0, 4.0, WHITE);
        }
    }

    if state.cascade_pulse > 0.0 {
        let board_w = layout.tile_size * GRID_WIDTH as f32;
        let board_h = layout.tile_size * GRID_HEIGHT as f32;
        draw_rectangle(
            layout.grid_offset_x,
            layout.grid_offset_y,
            board_w,
            board_h,
            Color::new(
                state.pulse_color.r,
                state.pulse_color.g,
                state.pulse_color.b,
                0.08 * state.cascade_pulse,
            ),
        );
    }

    for particle in &state.particles {
        let life_ratio = (particle.life / particle.max_life).clamp(0.0, 1.0);
        let size = particle.size * (0.55 + 0.45 * life_ratio);
        draw_circle(
            particle.x,
            particle.y,
            size,
            Color::new(particle.color.r, particle.color.g, particle.color.b, 0.70 * life_ratio),
        );
    }

    // --- Animated leaf overlay (Forest Floor only) ---
    if set_idx == 0 {
        if let Some(ref leaves_tex) = state.leaves_main_texture {
            let sw = screen_width();
            let sh = screen_height();
            let t = get_time() as f32;
            let tau = std::f32::consts::TAU;

            let sway_deg = (t * LEAF_SWAY_SPEED * tau).sin() * LEAF_SWAY_AMP_DEG
                + (t * LEAF_SWAY_SPEED_2 * tau).sin() * LEAF_SWAY_AMP_DEG * 0.38;
            let sway_rad = sway_deg * std::f32::consts::PI / 180.0;

            let pivot_x = sw * LEAF_SWAY_PIVOT_NX;
            let pivot_y = sh * LEAF_SWAY_PIVOT_NY;

            draw_texture_ex(
                leaves_tex,
                0.0,
                0.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(sw, sh)),
                    rotation: sway_rad,
                    pivot: Some(vec2(pivot_x, pivot_y)),
                    ..Default::default()
                },
            );
        }

        if let Some(ref leaves_aux_tex) = state.leaves_aux_texture {
            let sw = screen_width();
            let sh = screen_height();
            let t = get_time() as f32;
            let tau = std::f32::consts::TAU;

            let sway_deg = ((t * LEAF_SWAY_SPEED * tau) + LEAF_AUX_SWAY_PHASE).sin() * LEAF_AUX_SWAY_AMP_DEG
                + ((t * LEAF_SWAY_SPEED_2 * tau) + LEAF_AUX_SWAY_PHASE * 0.61).sin()
                    * LEAF_AUX_SWAY_AMP_DEG
                    * 0.42;
            let sway_rad = sway_deg * std::f32::consts::PI / 180.0;

            let pivot_x = sw * LEAF_AUX_SWAY_PIVOT_NX;
            let pivot_y = sh * LEAF_AUX_SWAY_PIVOT_NY;

            draw_texture_ex(
                leaves_aux_tex,
                0.0,
                0.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(sw, sh)),
                    rotation: sway_rad,
                    pivot: Some(vec2(pivot_x, pivot_y)),
                    ..Default::default()
                },
            );
        }
    }
}

pub fn draw_playing_ui(
    layout: &Layout,
    level: i32,
    target: i32,
    level_target_step: i32,
    total_points: i32,
    leaves_wallet: i32,
    illegal_move_cost: i32,
    inventory: &Inventory,
    is_farming: bool,
) {
    let bar_x = layout.ui_panel_x;
    let bar_y = layout.grid_offset_y;
    let bar_width = layout.ui_panel_width * 0.92;
    let row_h = layout.tile_size * 0.5;
    let prev_threshold = target - level_target_step;
    let level_progress = (total_points - prev_threshold) as f32 / level_target_step as f32;

    draw_rectangle(bar_x, bar_y, bar_width, row_h * 0.27, GRAY);
    draw_rectangle(
        bar_x,
        bar_y,
        bar_width * level_progress.clamp(0.0, 1.0),
        row_h * 0.27,
        GOLD,
    );

    let font_lg = (row_h * 1.05).max(18.0);
    let font_sm = (row_h * 0.88).max(15.0);
    draw_text(&format!("Level {}", level), bar_x, bar_y + row_h * 1.0, font_lg, WHITE);
    draw_text(
        &format!("Leaves: {}", leaves_wallet),
        bar_x,
        bar_y + row_h * 2.1,
        font_lg,
        GOLD,
    );
    draw_text(
        &format!("Illicit Move Cost: {}", illegal_move_cost),
        bar_x,
        bar_y + row_h * 3.1,
        font_sm,
        ORANGE,
    );

    let inv_title_y = bar_y + row_h * 4.25;
    draw_text(
        &format!("Inventory {}/8", economy::inventory_used_slots(inventory)),
        bar_x,
        inv_title_y,
        font_sm,
        WHITE,
    );

    let slot_size = (layout.ui_panel_width * 0.16).min(row_h * 0.98).max(20.0);
    let slot_gap = layout.ui_panel_width * 0.018;
    let slot_y = inv_title_y + row_h * 0.28;
    let chip_font = (font_sm * 0.70).max(11.0);
    let inventory_chips = [
        (
            "CAN",
            economy::inventory_count(inventory, ItemType::WateringCan),
            color_u8!(55, 125, 210, 255),
        ),
        (
            "SUN",
            economy::inventory_count(inventory, ItemType::SeedDay),
            color_u8!(210, 165, 35, 255),
        ),
        (
            "MON",
            economy::inventory_count(inventory, ItemType::SeedNight),
            color_u8!(100, 125, 220, 255),
        ),
        (
            "ESS",
            economy::inventory_count(inventory, ItemType::MoonbloomEssence),
            color_u8!(160, 90, 220, 255),
        ),
        (
            "FERT",
            economy::inventory_count(inventory, ItemType::Fertilizer),
            color_u8!(185, 55, 55, 255),
        ),
    ];

    for (index, (label, count, color)) in inventory_chips.iter().enumerate() {
        let x = bar_x + index as f32 * (slot_size + slot_gap);
        draw_rectangle(x, slot_y, slot_size, slot_size, *color);
        draw_rectangle_lines(x, slot_y, slot_size, slot_size, 2.0, WHITE);

        draw_rectangle(
            x + slot_size * 0.18,
            slot_y + slot_size * 0.18,
            slot_size * 0.64,
            slot_size * 0.50,
            color_u8!(32, 38, 32, 160),
        );

        draw_text(
            label,
            x + slot_size * 0.08,
            slot_y + slot_size * 0.92,
            chip_font,
            WHITE,
        );

        let badge_w = slot_size * 0.28;
        let badge_h = slot_size * 0.28;
        let badge_x = x + slot_size - badge_w - slot_size * 0.05;
        let badge_y = slot_y + slot_size * 0.05;
        draw_rectangle(badge_x, badge_y, badge_w, badge_h, color_u8!(18, 24, 18, 235));
        draw_rectangle_lines(badge_x, badge_y, badge_w, badge_h, 1.5, color_u8!(220, 235, 210, 255));
        draw_text(
            &format!("{}", count),
            badge_x + badge_w * 0.24,
            badge_y + badge_h * 0.74,
            (chip_font * 0.95).max(10.0),
            color_u8!(230, 245, 220, 255),
        );
    }

    let (visit_x, visit_y, visit_w, visit_h) = playing_visit_garden_button_rect(layout);
    draw_rectangle(visit_x, visit_y, visit_w, visit_h, color_u8!(36, 80, 42, 255));
    draw_rectangle_lines(
        visit_x,
        visit_y,
        visit_w,
        visit_h,
        3.0,
        color_u8!(152, 230, 160, 255),
    );
    draw_text(
        "VISIT GARDEN",
        visit_x + visit_w * 0.13,
        visit_y + visit_h * 0.68,
        font_lg,
        WHITE,
    );

    if is_farming {
        let (btn_x, btn_y, btn_w, btn_h) = playing_descend_button_rect(layout);
        draw_rectangle(btn_x, btn_y, btn_w, btn_h, DARKGREEN);
        draw_rectangle_lines(btn_x, btn_y, btn_w, btn_h, 3.0, GREEN);
        draw_text(
            "DESCEND >",
            btn_x + btn_w * 0.1,
            btn_y + btn_h * 0.68,
            font_lg,
            WHITE,
        );
    }
}

pub fn draw_level_transition_ui() {
    let sw = screen_width();
    let sh = screen_height();
    draw_text(
        "FOG CLEARED",
        sw * 0.19,
        sh * 0.43,
        (sh * 0.103).max(36.0),
        GREEN,
    );
    draw_text(
        "[ENTER] Descend (Next Level)",
        sw * 0.15,
        sh * 0.55,
        (sh * 0.052).max(22.0),
        WHITE,
    );
    draw_text(
        "[F] Farm (Stay & Collect)",
        sw * 0.15,
        sh * 0.62,
        (sh * 0.052).max(22.0),
        GOLD,
    );
}