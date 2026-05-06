use macroquad::prelude::*;

pub struct Layout {
    pub tile_size: f32,
    pub grid_offset_x: f32,
    pub grid_offset_y: f32,
    pub ui_panel_x: f32,
    pub ui_panel_width: f32,
}

impl Layout {
    pub fn compute(grid_width: usize, grid_height: usize) -> Self {
        let sw = screen_width();
        let sh = screen_height();
        let h_margin = (sw * 0.0125).floor();
        let gap = (sw * 0.0225).floor();
        let ui_w = (sw * 0.325).floor();
        let board_max_w = sw - h_margin * 2.0 - gap - ui_w;
        let board_max_h = sh * 0.96;
        let base_tile_size = (board_max_w / grid_width as f32)
            .min(board_max_h / grid_height as f32)
            .floor();

        // Narrow windows make the frame art crowd the board; gently reduce board scale
        // at small widths, then blend back to full size on wider windows.
        let width_blend = ((sw - 800.0) / 400.0).clamp(0.0, 1.0);
        let board_scale_comp = 0.97 + 0.03 * width_blend;
        let tile_size = (base_tile_size * board_scale_comp).floor().max(16.0);
        let board_w = tile_size * grid_width as f32;
        let board_h = tile_size * grid_height as f32;
        let grid_offset_x = h_margin;
        let grid_offset_y = ((sh - board_h) * 0.5).max(4.0).floor();
        let ui_panel_x = grid_offset_x + board_w + gap;

        Layout {
            tile_size,
            grid_offset_x,
            grid_offset_y,
            ui_panel_x,
            ui_panel_width: ui_w,
        }
    }
}

pub fn point_in_rect(mx: f32, my: f32, x: f32, y: f32, w: f32, h: f32) -> bool {
    mx >= x && mx <= x + w && my >= y && my <= y + h
}

pub fn playing_visit_garden_button_rect(layout: &Layout) -> (f32, f32, f32, f32) {
    let row_h = layout.tile_size * 0.5;
    let inv_title_y = layout.grid_offset_y + row_h * 4.25;
    let chip_h = (row_h * 0.9).max(20.0);
    let chip_y = inv_title_y + row_h * 0.28;
    let btn_x = layout.ui_panel_x;
    let btn_y = chip_y + chip_h + row_h * 0.45;
    let btn_w = layout.ui_panel_width * 0.77;
    let btn_h = btn_w * 0.25;
    (btn_x, btn_y, btn_w, btn_h)
}

pub fn playing_descend_button_rect(layout: &Layout) -> (f32, f32, f32, f32) {
    let row_h = layout.tile_size * 0.5;
    let (btn_x, visit_y, btn_w, btn_h) = playing_visit_garden_button_rect(layout);
    let btn_y = visit_y + btn_h + row_h * 0.35;
    (btn_x, btn_y, btn_w, btn_h)
}

pub fn garden_return_button_rect() -> (f32, f32, f32, f32) {
    let sw = screen_width();
    let sh = screen_height();
    (sw * 0.65, sh * 0.02, sw * 0.15, sh * 0.06)
}

pub fn garden_hunt_button_rect() -> (f32, f32, f32, f32) {
    let sw = screen_width();
    let sh = screen_height();
    (sw * 0.82, sh * 0.02, sw * 0.15, sh * 0.06)
}

pub fn hunt_return_button_rect() -> (f32, f32, f32, f32) {
    let sw = screen_width();
    let sh = screen_height();
    (sw * 0.35, sh * 0.72, sw * 0.30, sh * 0.10)
}
