use macroquad::prelude::*;

use crate::economy;
use crate::inventory::{Inventory, ItemType};
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
                draw_circle(sx, sy, iso_dot_radius, color_u8!(255, 20, 147, 240));
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
        sw * 0.34,
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

    let chip_h = (row_h * 0.9).max(20.0);
    let chip_w = layout.ui_panel_width * 0.17;
    let chip_gap = layout.ui_panel_width * 0.018;
    let chip_y = inv_title_y + row_h * 0.28;
    let chip_font = (font_sm * 0.78).max(12.0);
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
        let x = bar_x + index as f32 * (chip_w + chip_gap);
        draw_rectangle(x, chip_y, chip_w, chip_h, *color);
        draw_rectangle_lines(x, chip_y, chip_w, chip_h, 2.0, WHITE);
        draw_text(
            &format!("{}:{}", label, count),
            x + chip_w * 0.08,
            chip_y + chip_h * 0.72,
            chip_font,
            WHITE,
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