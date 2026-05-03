use macroquad::prelude::*;

use crate::game_state::{GamePhase, GameState, GRID_HEIGHT, GRID_WIDTH};
use crate::ui_layout::{
    point_in_rect,
    playing_descend_button_rect,
    playing_visit_garden_button_rect,
    Layout,
};

impl GameState {
    pub(crate) fn update_playing(&mut self) {
        let delta = get_frame_time();
        self.animate_tiles(delta);
        self.update_match_effects(delta);
        let layout = Layout::compute(GRID_WIDTH, GRID_HEIGHT);

        if self.is_clearing() {
            self.clear_timer -= delta;
            if self.clear_timer <= 0.0 {
                self.finalize_match_clear();
            }
        } else if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            self.handle_playing_click(mx, my, &layout);
        }

        if self.phase == GamePhase::Playing
            && !self.is_clearing()
            && self.total_points >= self.target
            && !self.is_farming
        {
            self.phase = GamePhase::LevelTransition;
        }
    }

    pub(crate) fn handle_playing_click(&mut self, mx: f32, my: f32, layout: &Layout) {
        let (garden_x, garden_y, garden_w, garden_h) = playing_visit_garden_button_rect(layout);
        if point_in_rect(mx, my, garden_x, garden_y, garden_w, garden_h) {
            self.phase = GamePhase::Garden;
            return;
        }

        if self.is_farming {
            let (btn_x, btn_y, btn_w, btn_h) = playing_descend_button_rect(layout);
            if point_in_rect(mx, my, btn_x, btn_y, btn_w, btn_h) {
                self.phase = GamePhase::LevelTransition;
                return;
            }
        }

        let gx = ((mx - layout.grid_offset_x) / layout.tile_size).floor() as isize;
        let gy = ((my - layout.grid_offset_y) / layout.tile_size).floor() as isize;

        if gx < 0 || gx >= GRID_WIDTH as isize || gy < 0 || gy >= GRID_HEIGHT as isize {
            self.selected = None;
            return;
        }

        self.handle_board_selection(gx as usize, gy as usize);
    }

    pub(crate) fn handle_board_selection(&mut self, gx: usize, gy: usize) {
        match self.selected {
            None => self.selected = Some((gx, gy)),
            Some((sx, sy)) => {
                let dx = (gx as isize - sx as isize).abs();
                let dy = (gy as isize - sy as isize).abs();
                if (dx == 1 && dy == 0) || (dx == 0 && dy == 1) {
                    self.resolve_swap(sx, sy, gx, gy);
                } else {
                    self.selected = Some((gx, gy));
                }
            }
        }
    }

    pub(crate) fn resolve_swap(&mut self, sx: usize, sy: usize, gx: usize, gy: usize) {
        let temp = self.grid[sx][sy];
        self.grid[sx][sy] = self.grid[gx][gy];
        self.grid[gx][gy] = temp;

        let matches = self.find_matches();
        if matches.is_empty() {
            // Intentionally keep the swap: illicit moves are a mechanic,
            // so do not revert or deny non-matching swaps here.
            self.charge_illegal_move();
        } else {
            self.begin_match_clear(matches, false);
        }

        self.selected = None;
    }
}