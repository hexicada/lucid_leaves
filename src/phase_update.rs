use macroquad::prelude::*;

use crate::game_state::{GamePhase, GameState, GardenTool, LEVEL_TARGET_STEP, LEVELS_PER_SET, ISO_TILE_HW, ISO_TILE_HH, ISO_LEFT_ORIGIN_NX, ISO_LEFT_ORIGIN_NY, ISO_DOT_RADIUS};
use crate::ui_layout::{
    garden_hunt_button_rect,
    garden_return_button_rect,
    hunt_return_button_rect,
    point_in_rect,
};
use crate::inventory::ItemType;
use crate::garden::PlantType;

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
        if is_key_pressed(KeyCode::Escape) {
            if self.garden_selected_tool.is_some() {
                self.garden_selected_tool = None;
            } else if self.garden_drawer_open {
                self.garden_drawer_open = false;
            } else {
                self.phase = GamePhase::Playing;
            }
            return;
        }

        if is_key_pressed(KeyCode::I) {
            self.garden_drawer_open = !self.garden_drawer_open;
            if !self.garden_drawer_open {
                self.garden_selected_tool = None;
            }
        }

        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            let (rx, ry, rw, rh) = garden_return_button_rect();
            let (hx, hy, hw, hh) = garden_hunt_button_rect();

            if point_in_rect(mx, my, rx, ry, rw, rh) {
                self.phase = GamePhase::Playing;
                return;
            } else if point_in_rect(mx, my, hx, hy, hw, hh) {
                self.phase = GamePhase::Hunt;
                return;
            }

            if self.check_drawer_toggle_click(mx, my) {
                self.garden_drawer_open = !self.garden_drawer_open;
                if !self.garden_drawer_open {
                    self.garden_selected_tool = None;
                }
                return;
            }

            // Check drawer tool buttons
            if let Some(tool) = self.check_drawer_button_click(mx, my) {
                self.garden_selected_tool = Some(tool);
                return;
            }

            // If tool is active, try clicking a plot
            if let Some(tool) = self.garden_selected_tool {
                if let Some(plot_idx) = self.point_to_plot_index(mx, my) {
                    let now_unix = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs() as i64;
                    
                    self.apply_garden_tool(tool, plot_idx, now_unix);
                    self.garden_selected_tool = None;
                }
            }
        }
    }

    fn check_drawer_toggle_click(&self, mx: f32, my: f32) -> bool {
        let sw = screen_width();
        let sh = screen_height();
        let drawer_y = sh * 0.84;
        let tab_w = sw * 0.18;
        let tab_h = sh * 0.05;
        let tab_x = (sw - tab_w) * 0.5;
        let tab_y = if self.garden_drawer_open { drawer_y - tab_h * 0.8 } else { sh * 0.95 - tab_h };
        point_in_rect(mx, my, tab_x, tab_y, tab_w, tab_h)
    }

    fn check_drawer_button_click(&self, mx: f32, my: f32) -> Option<GardenTool> {
        if !self.garden_drawer_open {
            return None;
        }

        let sw = screen_width();
        let sh = screen_height();
        let drawer_y = sh * 0.84;
        let drawer_h = sh * 0.16;
        
        // Only check if in drawer area
        if my < drawer_y {
            return None;
        }

        let btn_size = (drawer_h * 0.52).min(sw * 0.09);
        let btn_start_y = drawer_y + drawer_h * 0.24;
        let spacing = sw * 0.012;

        let buttons = [
            ("CAN", GardenTool::Water),
            ("SUN", GardenTool::PlantSun),
            ("MON", GardenTool::PlantMoon),
            ("ESS", GardenTool::PlantEssence),
            ("FERT", GardenTool::Fertilize),
        ];

        let total_w = buttons.len() as f32 * btn_size + (buttons.len() as f32 - 1.0) * spacing;
        let start_x = (sw - total_w) * 0.5;

        for (idx, (_, tool)) in buttons.iter().enumerate() {
            let btn_x = start_x + idx as f32 * (btn_size + spacing);
            if point_in_rect(mx, my, btn_x, btn_start_y, btn_size, btn_size) {
                return Some(*tool);
            }
        }
        None
    }

    fn point_to_plot_index(&self, mx: f32, my: f32) -> Option<usize> {
        let sw = screen_width();
        let sh = screen_height();
        
        let left_origin_x = sw * ISO_LEFT_ORIGIN_NX;
        let left_origin_y = sh * ISO_LEFT_ORIGIN_NY;
        
        // Check left lobe (plots 0-8, but only using 3x3 = 9 plots)
        for (plot_local_idx, (gx, gy)) in [(0, 0), (1, 0), (2, 0),
                                             (0, 1), (1, 1), (2, 1),
                                             (0, 2), (1, 2), (2, 2)].iter().enumerate() {
            let sx = left_origin_x + (*gx as f32 - *gy as f32) * ISO_TILE_HW;
            let sy = left_origin_y + (*gx as f32 + *gy as f32) * ISO_TILE_HH;
            
            let dx = mx - sx;
            let dy = my - sy;
            if dx * dx + dy * dy <= ISO_DOT_RADIUS * ISO_DOT_RADIUS * 25.0 {
                return Some(plot_local_idx);
            }
        }
        None
    }

    fn apply_garden_tool(&mut self, tool: GardenTool, plot_idx: usize, now_unix: i64) {
        use crate::economy::inventory_count;
        
        // Find the slot first (before mutably borrowing plot)
        let slot_idx = self.find_inventory_slot_for_tool(tool);
        
        let plot = &mut self.garden.plots[plot_idx];
        
        match tool {
            GardenTool::Water => {
                if inventory_count(&self.inventory, ItemType::WateringCan) > 0 {
                    if plot.water() {
                        if let Some(idx) = slot_idx {
                            self.inventory.discard_one(idx);
                        }
                    }
                }
            }
            GardenTool::PlantSun => {
                if inventory_count(&self.inventory, ItemType::SeedDay) > 0 {
                    if plot.plant(PlantType::DayBloom, now_unix) {
                        if let Some(idx) = slot_idx {
                            self.inventory.discard_one(idx);
                        }
                    }
                }
            }
            GardenTool::PlantMoon => {
                if inventory_count(&self.inventory, ItemType::SeedNight) > 0 {
                    if plot.plant(PlantType::NightBloom, now_unix) {
                        if let Some(idx) = slot_idx {
                            self.inventory.discard_one(idx);
                        }
                    }
                }
            }
            GardenTool::PlantEssence => {
                if inventory_count(&self.inventory, ItemType::MoonbloomEssence) > 0 {
                    if plot.plant(PlantType::DayBloom, now_unix) {
                        if let Some(idx) = slot_idx {
                            self.inventory.discard_one(idx);
                        }
                        plot.moonbloom_infused = true;
                    }
                }
            }
            GardenTool::Fertilize => {
                if inventory_count(&self.inventory, ItemType::Fertilizer) > 0 {
                    if plot.fertilize() {
                        if let Some(idx) = slot_idx {
                            self.inventory.discard_one(idx);
                        }
                    }
                }
            }
        }
    }

    fn find_inventory_slot_for_tool(&self, tool: GardenTool) -> Option<usize> {
        let item = match tool {
            GardenTool::Water => ItemType::WateringCan,
            GardenTool::PlantSun => ItemType::SeedDay,
            GardenTool::PlantMoon => ItemType::SeedNight,
            GardenTool::PlantEssence => ItemType::MoonbloomEssence,
            GardenTool::Fertilize => ItemType::Fertilizer,
        };
        self.inventory.slots.iter().position(|slot| slot.item == Some(item))
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