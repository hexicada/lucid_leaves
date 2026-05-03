use macroquad::prelude::*;

use crate::game_state::{GamePhase, GameState, LEVEL_TARGET_STEP, LEVELS_PER_SET};
use crate::ui_layout::{
    garden_hunt_button_rect,
    garden_return_button_rect,
    hunt_return_button_rect,
    point_in_rect,
};

impl GameState {
    pub(crate) fn update_level_transition(&mut self) {
        if is_key_pressed(KeyCode::Enter) {
            if self.level % LEVELS_PER_SET == 0 {
                self.reset_illegal_move_cost();
                self.phase = GamePhase::Shop;
            } else {
                self.level += 1;
                self.target += LEVEL_TARGET_STEP;
                self.phase = GamePhase::Playing;
            }
            self.is_farming = false;
        }

        if is_key_pressed(KeyCode::F) {
            self.is_farming = true;
            self.phase = GamePhase::Playing;
        }
    }

    pub(crate) fn update_shop(&mut self) {
        if is_key_pressed(KeyCode::Enter) {
            self.level += 1;
            self.target += LEVEL_TARGET_STEP;
            self.reset_illegal_move_cost();
            self.phase = GamePhase::Playing;
        }

        if is_key_pressed(KeyCode::Space) && self.get_leaves_wallet() >= 500 {
            self.spent_points += 500;
        }
    }

    pub(crate) fn update_garden(&mut self) {
        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            let (rx, ry, rw, rh) = garden_return_button_rect();
            let (hx, hy, hw, hh) = garden_hunt_button_rect();

            if point_in_rect(mx, my, rx, ry, rw, rh) {
                self.phase = GamePhase::Playing;
            } else if point_in_rect(mx, my, hx, hy, hw, hh) {
                self.phase = GamePhase::Hunt;
            }
        }

        if is_key_pressed(KeyCode::Escape) {
            self.phase = GamePhase::Playing;
        }
    }

    pub(crate) fn update_hunt(&mut self) {
        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            let (bx, by, bw, bh) = hunt_return_button_rect();
            if point_in_rect(mx, my, bx, by, bw, bh) {
                self.phase = GamePhase::Garden;
            }
        }

        if is_key_pressed(KeyCode::Escape) {
            self.phase = GamePhase::Garden;
        }
    }
}