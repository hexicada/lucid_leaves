use macroquad::prelude::*;
use macroquad::rand::gen_range;
use crate::tile::*; // Import our neighbor, the Tile
use crate::inventory::{Inventory, ItemType};
use crate::shop::Shop;

pub const GRID_WIDTH: usize = 8;
pub const GRID_HEIGHT: usize = 8;
pub const TILE_SIZE: f32 = 64.0;
pub const LEVELS_PER_SET: i32 = 3; // Shop appears every 3 levels
pub const LEVEL_TARGET_STEP: i32 = 1200;
pub const DROP_RATE_LEAF_LEAVES: f32 = 0.30;
pub const DROP_RATE_WATERING_CAN: f32 = 0.30;
pub const DROP_RATE_SEED_DAY: f32 = 0.20;
pub const DROP_RATE_MOON_ITEM: f32 = 0.10;
pub const DROP_RATE_FERTILIZER: f32 = 0.15;
pub const LEAF_DROP_BONUS: i32 = 30;

/// Computed each frame from screen dimensions so the layout adapts to any window size.
pub struct Layout {
    pub tile_size: f32,
    pub grid_offset_x: f32,
    pub grid_offset_y: f32,
    pub ui_panel_x: f32,
    pub ui_panel_width: f32,
}

impl Layout {
    pub fn compute() -> Self {
        let sw = screen_width();
        let sh = screen_height();
        let h_margin = (sw * 0.0125).floor();
        let gap      = (sw * 0.0225).floor();
        let ui_w     = (sw * 0.325).floor();
        let board_max_w = sw - h_margin * 2.0 - gap - ui_w;
        let board_max_h = sh * 0.96;
        let tile_size = (board_max_w / GRID_WIDTH as f32)
            .min(board_max_h / GRID_HEIGHT as f32)
            .floor()
            .max(16.0);
        let board_w = tile_size * GRID_WIDTH as f32;
        let board_h = tile_size * GRID_HEIGHT as f32;
        let grid_offset_x = h_margin;
        let grid_offset_y = ((sh - board_h) * 0.5).max(4.0).floor();
        let ui_panel_x = grid_offset_x + board_w + gap;
        Layout { tile_size, grid_offset_x, grid_offset_y, ui_panel_x, ui_panel_width: ui_w }
    }
}
pub const ILLEGAL_MOVE_COST_START: i32 = 100;
pub const ILLEGAL_MOVE_COST_STEP: i32 = 100;
pub const PRICE_WATERING_CAN: i32 = 220;
pub const PRICE_SEED_DAY: i32 = 260;
pub const PRICE_SEED_NIGHT: i32 = 320;
pub const PRICE_MOONBLOOM_ESSENCE: i32 = 900;
pub const PRICE_FERTILIZER: i32 = 380;

#[allow(dead_code)]
#[derive(PartialEq)]
pub enum GamePhase {
    Playing,
    LevelTransition, // The "Fog Cleared" choice screen
    Shop,            // The Tarquin Screen
    Garden,          // Tend plants, visit Bagira consignment
    Hunt,            // Scrub mouse mini-game
    BossHunt,        // Biome-boundary boss encounter
}

pub struct GameState {
    pub grid: [[Tile; GRID_HEIGHT]; GRID_WIDTH],
    pub selected: Option<(usize, usize)>,

    // Asset Storage
    pub beryl_texture: Texture2D,
    pub moon_texture: Texture2D,
    pub leaf_texture: Texture2D,
    
    // --- THE BACKEND VARIABLES ---
    pub total_points: i32, // Lifetime score (The "Level" Bar)
    pub spent_points: i32, // Spent at shop
    
    pub target: i32,       // Points needed to clear fog
    pub level: i32,
    pub phase: GamePhase,
    pub illegal_move_cost: i32,
    pub inventory: Inventory,
    pub shop: Shop,
    
    // NEW FLAG: Are we in "Overtime"?
    pub is_farming: bool,
}

impl GameState {
    pub fn new(beryl_texture: Texture2D, moon_texture: Texture2D, leaf_texture: Texture2D) -> Self {
        let mut grid = [[Tile { kind: TileType::Empty, offset_y: 0.0 }; GRID_HEIGHT]; GRID_WIDTH];
        for x in 0..GRID_WIDTH { for y in 0..GRID_HEIGHT { grid[x][y] = Tile::new_random(); }}

        let mut game = GameState {
            grid,
            selected: None,
            total_points: 0,
            spent_points: 0,
            target: LEVEL_TARGET_STEP,
            level: 1,
            phase: GamePhase::Playing,
            illegal_move_cost: ILLEGAL_MOVE_COST_START,
            inventory: Inventory::new(),
            shop: Shop::new(),
            is_farming: false, // Start normally
            beryl_texture,
            moon_texture,
            leaf_texture,
        };

        // Clear any initial matches
        loop {
            let had_matches = game.resolve_matches();
            if had_matches {
                game.apply_gravity();
            } else {
                break;
            }
        }

        game
    }

    // Helper to calculate "Wallet" (Leaves)
    pub fn get_leaves_wallet(&self) -> i32 {
        self.total_points - self.spent_points
    }

    pub fn charge_illegal_move(&mut self) {
        self.spent_points += self.illegal_move_cost;
        self.illegal_move_cost += ILLEGAL_MOVE_COST_STEP;
    }

    pub fn reset_illegal_move_cost(&mut self) {
        self.illegal_move_cost = ILLEGAL_MOVE_COST_START;
    }

    fn base_price_for_item(item: ItemType) -> i32 {
        match item {
            ItemType::WateringCan => PRICE_WATERING_CAN,
            ItemType::SeedDay => PRICE_SEED_DAY,
            ItemType::SeedNight => PRICE_SEED_NIGHT,
            ItemType::MoonbloomEssence => PRICE_MOONBLOOM_ESSENCE,
            ItemType::Fertilizer => PRICE_FERTILIZER,
            // Placeholder pricing for non-resource items until full shop generation is wired.
            ItemType::BoardModifier(_) => 500,
            ItemType::FoodBuff(_) => 200,
        }
    }

    fn add_resource_or_consign(&mut self, item: ItemType) {
        if !self.inventory.push(item) {
            let paid = self.shop.bagira.consign(item, Self::base_price_for_item(item));
            self.total_points += paid;
        }
    }

    fn roll_resource_drop(&mut self, tile_kind: TileType) {
        let roll = gen_range(0.0, 1.0);
        match tile_kind {
            TileType::Leaf => {
                if roll < DROP_RATE_LEAF_LEAVES {
                    self.total_points += LEAF_DROP_BONUS;
                }
            }
            TileType::Water => {
                if roll < DROP_RATE_WATERING_CAN {
                    self.add_resource_or_consign(ItemType::WateringCan);
                }
            }
            TileType::Sun => {
                if roll < DROP_RATE_SEED_DAY {
                    self.add_resource_or_consign(ItemType::SeedDay);
                }
            }
            TileType::Moon => {
                if roll < DROP_RATE_MOON_ITEM {
                    let moon_item = if gen_range(0.0, 1.0) < 0.30 {
                        ItemType::MoonbloomEssence
                    } else {
                        ItemType::SeedNight
                    };
                    self.add_resource_or_consign(moon_item);
                }
            }
            TileType::Exotic => {
                if roll < DROP_RATE_FERTILIZER {
                    self.add_resource_or_consign(ItemType::Fertilizer);
                }
            }
            TileType::Empty => {}
        }
    }

    fn inventory_count(&self, target: ItemType) -> u32 {
        self.inventory
            .slots
            .iter()
            .filter_map(|slot| {
                if slot.item == Some(target) {
                    Some(slot.count)
                } else {
                    None
                }
            })
            .sum()
    }

    fn inventory_used_slots(&self) -> usize {
        self.inventory
            .slots
            .iter()
            .filter(|slot| slot.item.is_some())
            .count()
    }

    fn point_in_rect(mx: f32, my: f32, x: f32, y: f32, w: f32, h: f32) -> bool {
        mx >= x && mx <= x + w && my >= y && my <= y + h
    }

    fn playing_visit_garden_button_rect(layout: &Layout) -> (f32, f32, f32, f32) {
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

    fn playing_descend_button_rect(layout: &Layout) -> (f32, f32, f32, f32) {
        let row_h = layout.tile_size * 0.5;
        let (btn_x, visit_y, btn_w, btn_h) = Self::playing_visit_garden_button_rect(layout);
        let btn_y = visit_y + btn_h + row_h * 0.35;
        (btn_x, btn_y, btn_w, btn_h)
    }

    fn garden_return_button_rect() -> (f32, f32, f32, f32) {
        let sw = screen_width();
        let sh = screen_height();
        (sw * 0.14, sh * 0.72, sw * 0.30, sh * 0.10)
    }

    fn garden_hunt_button_rect() -> (f32, f32, f32, f32) {
        let sw = screen_width();
        let sh = screen_height();
        (sw * 0.56, sh * 0.72, sw * 0.30, sh * 0.10)
    }

    fn hunt_return_button_rect() -> (f32, f32, f32, f32) {
        let sw = screen_width();
        let sh = screen_height();
        (sw * 0.35, sh * 0.72, sw * 0.30, sh * 0.10)
    }

    pub fn draw_water_gem(&self, draw_x: f32, draw_y: f32, time: f64, tile_size: f32) {
        let size = tile_size - 2.0;
        let cx = draw_x + size * 0.5;
        let bob = (time as f32 * 2.4).sin() * 1.3;
        let pulse = ((time as f32 * 2.0).sin() + 1.0) * 0.5;
        let body_cy = draw_y + size * 0.57 + bob;

        draw_circle(
            cx,
            draw_y + size * 0.9,
            size * (0.2 + pulse * 0.02),
            color_u8!(10, 25, 40, 85),
        );

        draw_circle(cx, body_cy, size * 0.27, color_u8!(35, 150, 235, 255));
        draw_circle(
            cx - size * 0.14,
            body_cy - size * 0.02,
            size * 0.17,
            color_u8!(75, 185, 248, 255),
        );
        draw_circle(
            cx + size * 0.14,
            body_cy - size * 0.02,
            size * 0.17,
            color_u8!(55, 170, 242, 255),
        );

        let top = vec2(cx, draw_y + size * 0.12 + bob);
        let left_shoulder = vec2(draw_x + size * 0.34, draw_y + size * 0.44 + bob);
        let right_shoulder = vec2(draw_x + size * 0.66, draw_y + size * 0.44 + bob);
        draw_triangle(top, left_shoulder, right_shoulder, color_u8!(155, 230, 255, 245));

        draw_line(
            draw_x + size * 0.32,
            draw_y + size * 0.47 + bob,
            draw_x + size * 0.27,
            draw_y + size * 0.73 + bob,
            2.0,
            color_u8!(155, 230, 255, 130),
        );
        draw_line(
            draw_x + size * 0.68,
            draw_y + size * 0.47 + bob,
            draw_x + size * 0.73,
            draw_y + size * 0.73 + bob,
            2.0,
            color_u8!(155, 230, 255, 130),
        );

        draw_circle(
            draw_x + size * 0.43,
            draw_y + size * 0.33 + bob,
            size * 0.08,
            color_u8!(235, 250, 255, 170),
        );
        draw_circle(
            draw_x + size * 0.56,
            draw_y + size * 0.5 + bob,
            size * 0.045,
            color_u8!(200, 240, 255, 110),
        );
    }

    pub fn draw_exotic_gem(&self, draw_x: f32, draw_y: f32, time: f64, tile_size: f32) {
        let size = tile_size - 2.0;
        let cx = draw_x + size * 0.5;
        let bob = (time as f32 * 2.8 + 0.6).sin() * 1.1;
        let pulse = ((time as f32 * 3.0).sin() + 1.0) * 0.5;

        let top = vec2(cx, draw_y + size * 0.05 + bob);
        let upper_left = vec2(draw_x + size * 0.24, draw_y + size * 0.29 + bob);
        let upper_right = vec2(draw_x + size * 0.76, draw_y + size * 0.29 + bob);
        let mid_left = vec2(draw_x + size * 0.18, draw_y + size * 0.52 + bob);
        let mid_right = vec2(draw_x + size * 0.82, draw_y + size * 0.52 + bob);
        let lower_left = vec2(draw_x + size * 0.3, draw_y + size * 0.76 + bob);
        let lower_right = vec2(draw_x + size * 0.7, draw_y + size * 0.76 + bob);
        let bottom = vec2(cx, draw_y + size * 0.95 + bob);
        let center = vec2(cx, draw_y + size * 0.5 + bob);

        draw_circle(
            cx,
            draw_y + size * 0.91,
            size * (0.21 + pulse * 0.02),
            color_u8!(45, 8, 10, 95),
        );

        draw_triangle(top, upper_left, center, color_u8!(255, 185, 185, 255));
        draw_triangle(top, center, upper_right, color_u8!(255, 130, 130, 255));
        draw_triangle(upper_left, mid_left, center, color_u8!(228, 65, 72, 255));
        draw_triangle(center, mid_right, upper_right, color_u8!(192, 33, 40, 255));
        draw_triangle(mid_left, lower_left, center, color_u8!(168, 20, 28, 255));
        draw_triangle(center, lower_right, mid_right, color_u8!(148, 14, 22, 255));
        draw_triangle(lower_left, bottom, center, color_u8!(118, 8, 18, 255));
        draw_triangle(center, bottom, lower_right, color_u8!(180, 28, 35, 255));

        draw_line(top.x, top.y, upper_left.x, upper_left.y, 2.0, color_u8!(255, 226, 226, 190));
        draw_line(top.x, top.y, upper_right.x, upper_right.y, 2.0, color_u8!(255, 226, 226, 190));
        draw_line(upper_left.x, upper_left.y, mid_left.x, mid_left.y, 2.0, color_u8!(255, 166, 166, 150));
        draw_line(upper_right.x, upper_right.y, mid_right.x, mid_right.y, 2.0, color_u8!(255, 166, 166, 150));
        draw_line(mid_left.x, mid_left.y, lower_left.x, lower_left.y, 2.0, color_u8!(245, 120, 120, 135));
        draw_line(mid_right.x, mid_right.y, lower_right.x, lower_right.y, 2.0, color_u8!(245, 120, 120, 135));
        draw_line(upper_left.x, upper_left.y, center.x, center.y, 1.5, color_u8!(255, 240, 240, 170));
        draw_line(upper_right.x, upper_right.y, center.x, center.y, 1.5, color_u8!(255, 240, 240, 170));
        draw_line(lower_left.x, lower_left.y, bottom.x, bottom.y, 2.0, color_u8!(255, 110, 110, 145));
        draw_line(lower_right.x, lower_right.y, bottom.x, bottom.y, 2.0, color_u8!(255, 110, 110, 145));

        // Side spikes make exotic read sharper than the rounded water droplet.
        draw_triangle(
            mid_left,
            vec2(draw_x + size * 0.06, draw_y + size * 0.5 + bob),
            vec2(draw_x + size * 0.2, draw_y + size * 0.62 + bob),
            color_u8!(170, 18, 25, 225),
        );
        draw_triangle(
            mid_right,
            vec2(draw_x + size * 0.94, draw_y + size * 0.5 + bob),
            vec2(draw_x + size * 0.8, draw_y + size * 0.62 + bob),
            color_u8!(170, 18, 25, 225),
        );

        draw_circle(
            draw_x + size * 0.42,
            draw_y + size * 0.28 + bob,
            size * 0.075,
            color_u8!(255, 236, 236, 170),
        );
        draw_circle(
            draw_x + size * 0.58,
            draw_y + size * 0.45 + bob,
            size * 0.04,
            color_u8!(255, 210, 210, 100),
        );
    }

    pub fn resolve_matches(&mut self) -> bool {
        let mut to_remove = vec![];
        // 1. Horizontal
        for y in 0..GRID_HEIGHT { for x in 0..GRID_WIDTH - 2 {
            let t1 = self.grid[x][y].kind; let t2 = self.grid[x+1][y].kind; let t3 = self.grid[x+2][y].kind;
            if t1 != TileType::Empty && t1 == t2 && t2 == t3 { to_remove.push((x,y)); to_remove.push((x+1,y)); to_remove.push((x+2,y)); }
        }}
        // 2. Vertical
        for x in 0..GRID_WIDTH { for y in 0..GRID_HEIGHT - 2 {
            let t1 = self.grid[x][y].kind; let t2 = self.grid[x][y+1].kind; let t3 = self.grid[x][y+2].kind;
            if t1 != TileType::Empty && t1 == t2 && t2 == t3 { to_remove.push((x,y)); to_remove.push((x,y+1)); to_remove.push((x,y+2)); }
        }}
        
        to_remove.sort(); to_remove.dedup();
        if to_remove.is_empty() { return false; }

        let points = to_remove.len() as i32 * 10;
        self.total_points += points;

        // Snapshot matched kinds before clearing so drops reflect what was matched.
        let matched_kinds: Vec<TileType> = to_remove
            .iter()
            .map(|(x, y)| self.grid[*x][*y].kind)
            .collect();
        
        for (rx, ry) in to_remove { self.grid[rx][ry].kind = TileType::Empty; }

        for kind in matched_kinds {
            self.roll_resource_drop(kind);
        }

        true
    }
    pub fn apply_gravity(&mut self) {
        for x in 0..GRID_WIDTH {
            // Step 1: Scan from BOTTOM to TOP (y=7 down to y=0)
            for y in (0..GRID_HEIGHT).rev() {
                if self.grid[x][y].kind == TileType::Empty {
                    // Look for a non-empty tile ABOVE (indices 0 to y-1)
                    // We use .rev() to find the CLOSEST tile first
                    if let Some(source_y) = (0..y).rev().find(|&sy| self.grid[x][sy].kind != TileType::Empty) {
                        let distance_moved = y - source_y;
                        self.grid[x][y] = self.grid[x][source_y];
                        self.grid[x][y].offset_y = -(distance_moved as f32 * TILE_SIZE);
                        self.grid[x][source_y].kind = TileType::Empty;
                    }
                }
            }
            
            // Step 2: Fill the remaining empty slots at the top with new tiles
            for y in 0..GRID_HEIGHT {
                if self.grid[x][y].kind == TileType::Empty {
                    self.grid[x][y] = Tile::new_random();
                    self.grid[x][y].offset_y = -(TILE_SIZE * 4.0); // Spawn from above
                }
            }
        }
    }

    // Animate tile offsets back to 0
    pub fn animate_tiles(&mut self, delta: f32) {
        let animation_speed = 800.0; // Pixels per second
        for x in 0..GRID_WIDTH {
            for y in 0..GRID_HEIGHT {
                if self.grid[x][y].offset_y != 0.0 {
                    let move_amount = animation_speed * delta;
                    if self.grid[x][y].offset_y < 0.0 {
                        self.grid[x][y].offset_y += move_amount;
                        if self.grid[x][y].offset_y > 0.0 {
                            self.grid[x][y].offset_y = 0.0;
                        }
                    } else {
                        self.grid[x][y].offset_y -= move_amount;
                        if self.grid[x][y].offset_y < 0.0 {
                            self.grid[x][y].offset_y = 0.0;
                        }
                    }
                }
            }
        }
    }

    pub fn update(&mut self) {
        match self.phase {
            GamePhase::Playing => {
                // Animate tile offsets
                self.animate_tiles(get_frame_time());
                let layout = Layout::compute();

                // 1. MOUSE LOGIC FOR GRID
                if is_mouse_button_pressed(MouseButton::Left) {
                    let (mx, my) = mouse_position();
                    
                    // CHECK IF CLICKING DESCEND BUTTON (Only if farming)
                    let mut clicked_button = false;

                    let (garden_x, garden_y, garden_w, garden_h) = Self::playing_visit_garden_button_rect(&layout);
                    if Self::point_in_rect(mx, my, garden_x, garden_y, garden_w, garden_h) {
                        self.phase = GamePhase::Garden;
                        clicked_button = true;
                    }

                    if self.is_farming {
                        let (btn_x, btn_y, btn_w, btn_h) = Self::playing_descend_button_rect(&layout);
                        if !clicked_button && Self::point_in_rect(mx, my, btn_x, btn_y, btn_w, btn_h) {
                                self.phase = GamePhase::LevelTransition;
                                clicked_button = true;
                        }
                    }

                    if !clicked_button {
                        // GRID LOGIC
                        let gx = ((mx - layout.grid_offset_x) / layout.tile_size).floor() as isize;
                        let gy = ((my - layout.grid_offset_y) / layout.tile_size).floor() as isize;

                        if gx >= 0 && gx < GRID_WIDTH as isize && gy >= 0 && gy < GRID_HEIGHT as isize {
                            match self.selected {
                                None => self.selected = Some((gx as usize, gy as usize)),
                                Some((sx, sy)) => {
                                    let gx = gx as usize; let gy = gy as usize;
                                    let dx = (gx as isize - sx as isize).abs(); let dy = (gy as isize - sy as isize).abs();
                                    if (dx == 1 && dy == 0) || (dx == 0 && dy == 1) {
                                        let temp = self.grid[sx][sy]; self.grid[sx][sy] = self.grid[gx][gy]; self.grid[gx][gy] = temp;
                                        let had_matches = self.resolve_matches();
                                        if had_matches {
                                            // Cascade: resolve matches and apply gravity until stable
                                            self.apply_gravity();
                                            loop {
                                                let had_matches = self.resolve_matches();
                                                if had_matches {
                                                    self.apply_gravity();
                                                } else {
                                                    break;
                                                }
                                            }
                                        } else {
                                            self.charge_illegal_move();
                                        }
                                        self.selected = None;
                                    } else { self.selected = Some((gx, gy)); }
                                }
                            }
                        } else { self.selected = None; }
                    }
                }

                // 2. CHECK LEVEL THRESHOLD
                // If target met AND we aren't already farming, trigger transition
                if self.phase == GamePhase::Playing && self.total_points >= self.target && !self.is_farming {
                    self.phase = GamePhase::LevelTransition;
                }
            }
            
            GamePhase::LevelTransition => {
                // OPTION A: Next Level
                if is_key_pressed(KeyCode::Enter) {
                    if self.level % LEVELS_PER_SET == 0 {
                        self.reset_illegal_move_cost();
                        self.phase = GamePhase::Shop; 
                    } else { 
                        self.level += 1;
                        self.target += LEVEL_TARGET_STEP;
                        self.phase = GamePhase::Playing; 
                    }
                    // Reset farming flag for the new level
                    self.is_farming = false; 
                }
                
                // OPTION B: Stay and Farm
                if is_key_pressed(KeyCode::F) {
                    self.is_farming = true;
                    self.phase = GamePhase::Playing;
                }
            }

            GamePhase::Shop => {
                if is_key_pressed(KeyCode::Enter) {
                    self.level += 1;
                    self.target += LEVEL_TARGET_STEP;
                    self.reset_illegal_move_cost();
                    self.phase = GamePhase::Playing;
                }
                // Debug spending
                if is_key_pressed(KeyCode::Space) && self.get_leaves_wallet() >= 500 {
                    self.spent_points += 500;
                }
            }
            GamePhase::Garden => {
                if is_mouse_button_pressed(MouseButton::Left) {
                    let (mx, my) = mouse_position();
                    let (rx, ry, rw, rh) = Self::garden_return_button_rect();
                    let (hx, hy, hw, hh) = Self::garden_hunt_button_rect();

                    if Self::point_in_rect(mx, my, rx, ry, rw, rh) {
                        self.phase = GamePhase::Playing;
                    } else if Self::point_in_rect(mx, my, hx, hy, hw, hh) {
                        self.phase = GamePhase::Hunt;
                    }
                }

                if is_key_pressed(KeyCode::Escape) {
                    self.phase = GamePhase::Playing;
                }
            }
            GamePhase::Hunt => {
                if is_mouse_button_pressed(MouseButton::Left) {
                    let (mx, my) = mouse_position();
                    let (bx, by, bw, bh) = Self::hunt_return_button_rect();
                    if Self::point_in_rect(mx, my, bx, by, bw, bh) {
                        self.phase = GamePhase::Garden;
                    }
                }

                if is_key_pressed(KeyCode::Escape) {
                    self.phase = GamePhase::Garden;
                }
            }
            GamePhase::BossHunt => {}
        }
    }

    pub fn draw(&self) {
        // Biome Background Logic
        let set_index = (self.level - 1) / LEVELS_PER_SET;
        let bg_color = match set_index { 0 => BLACK, 1 => color_u8!(20, 10, 40, 255), _ => color_u8!(30, 0, 0, 255) };
        clear_background(bg_color);
    let layout = Layout::compute();

        if self.phase == GamePhase::Shop {
            let sw = screen_width();
            let sh = screen_height();
            draw_text("THE SHRINE",                              sw * 0.25, sh * 0.17, (sh * 0.103).max(40.0), PURPLE);
            draw_text(&format!("Leaves: {}", self.get_leaves_wallet()), sw * 0.31, sh * 0.34, (sh * 0.069).max(28.0), GOLD);
            draw_text("[SPACE] Spend 500 | [ENTER] Next Biome", sw * 0.19, sh * 0.69, (sh * 0.052).max(22.0), WHITE);
            return;
        }

        if self.phase == GamePhase::Garden {
            let sw = screen_width();
            let sh = screen_height();
            clear_background(color_u8!(28, 52, 30, 255));
            draw_text("THE GARDEN", sw * 0.30, sh * 0.18, (sh * 0.10).max(36.0), color_u8!(220, 245, 185, 255));
            draw_text("Rest phase: tend plots and manage resources", sw * 0.17, sh * 0.30, (sh * 0.045).max(20.0), WHITE);

            let (rx, ry, rw, rh) = Self::garden_return_button_rect();
            draw_rectangle(rx, ry, rw, rh, color_u8!(52, 80, 58, 255));
            draw_rectangle_lines(rx, ry, rw, rh, 3.0, color_u8!(170, 225, 170, 255));
            draw_text("RETURN TO PUZZLE", rx + rw * 0.08, ry + rh * 0.62, (rh * 0.42).max(20.0), WHITE);

            let (hx, hy, hw, hh) = Self::garden_hunt_button_rect();
            draw_rectangle(hx, hy, hw, hh, color_u8!(78, 65, 36, 255));
            draw_rectangle_lines(hx, hy, hw, hh, 3.0, color_u8!(240, 200, 120, 255));
            draw_text("GO HUNT", hx + hw * 0.29, hy + hh * 0.62, (hh * 0.48).max(20.0), WHITE);
            return;
        }

        if self.phase == GamePhase::Hunt {
            let sw = screen_width();
            let sh = screen_height();
            clear_background(color_u8!(48, 24, 26, 255));
            draw_text("HUNT (PLACEHOLDER)", sw * 0.19, sh * 0.20, (sh * 0.095).max(34.0), color_u8!(255, 210, 180, 255));
            draw_text("Wizard mice mini-game implementation is next sprint", sw * 0.13, sh * 0.34, (sh * 0.045).max(20.0), WHITE);

            let (bx, by, bw, bh) = Self::hunt_return_button_rect();
            draw_rectangle(bx, by, bw, bh, color_u8!(85, 45, 44, 255));
            draw_rectangle_lines(bx, by, bw, bh, 3.0, color_u8!(255, 190, 170, 255));
            draw_text("BACK TO GARDEN", bx + bw * 0.11, by + bh * 0.62, (bh * 0.42).max(20.0), WHITE);
            return;
        }

        // --- NEW: ANIMATION MATH (Global Time) ---
        // Cycles frames 0->12->0 over roughly 2 seconds
        let time = get_time();
        let speed = 13.0; // 13 frames per second
        let total_frames = 24.0; // 13 forward + 11 back = 24 total loop
        
        // The "Ping Pong" Index Calculation
        let mut frame_index = (time * speed) % total_frames;
        if frame_index > 12.0 {
            frame_index = 24.0 - frame_index; // Reverse it for the second half
        }
        let current_frame = frame_index.floor() as f32; // Convert to flat number (0..12)

       // 1. DRAW GRID
        for x in 0..GRID_WIDTH {
            for y in 0..GRID_HEIGHT {
                let tile = &self.grid[x][y];
                let draw_x = layout.grid_offset_x + x as f32 * layout.tile_size;
                let draw_y = layout.grid_offset_y + y as f32 * layout.tile_size + tile.offset_y;
                
                // PART A: DRAW THE TILE (Beryl or Normal)
                match tile.kind {
                    TileType::Sun => {
                        let sprite_size = 64.0;
                        draw_texture_ex(
                            &self.beryl_texture,
                            draw_x, 
                            draw_y,
                            WHITE,
                            DrawTextureParams {
                                dest_size: Some(vec2(layout.tile_size, layout.tile_size)),
                                source: Some(Rect::new(
                                    current_frame * sprite_size as f32, 
                                    0.0,                        
                                    sprite_size as f32,                
                                    sprite_size as f32                 
                                )),
                                ..Default::default()
                            },
                        );
                    }, // <--- Comma here helps the compiler!
                    TileType::Moon => {
                        let sprite_size = 64.0;
                        draw_texture_ex(
                            &self.moon_texture,
                            draw_x,
                            draw_y,
                            WHITE,
                            DrawTextureParams {
                                dest_size: Some(vec2(layout.tile_size, layout.tile_size)),
                                source: Some(Rect::new(
                                    current_frame * sprite_size as f32,
                                    0.0,
                                    sprite_size as f32,
                                    sprite_size as f32
                                )),
                                ..Default::default()
                            },
                        );
                    },
                    TileType::Leaf => {
                        let sprite_size = 64.0;
                        draw_texture_ex(
                            &self.leaf_texture,
                            draw_x,
                            draw_y,
                            WHITE,
                            DrawTextureParams {
                                dest_size: Some(vec2(layout.tile_size, layout.tile_size)),
                                source: Some(Rect::new(
                                    current_frame * sprite_size as f32,
                                    0.0,
                                    sprite_size as f32,
                                    sprite_size as f32
                                )),
                                ..Default::default()
                            },
                        );
                    },
                    TileType::Exotic => {
                        self.draw_exotic_gem(draw_x, draw_y, time, layout.tile_size);
                    },
                    TileType::Water => {
                        self.draw_water_gem(draw_x, draw_y, time, layout.tile_size);
                    },
                    
                    _ => {
                        // Draw everything else normally
                        draw_rectangle(draw_x, draw_y, layout.tile_size - 2.0, layout.tile_size - 2.0, tile.get_color(self.level));
                    }
                } // <--- END OF MATCH

                // PART B: DRAW THE SELECTION (Outside the match so it works on ALL tiles)
                if let Some((sx, sy)) = self.selected {
                    if sx == x && sy == y { 
                        draw_rectangle_lines(draw_x, draw_y, layout.tile_size - 2.0, layout.tile_size - 2.0, 4.0, WHITE); 
                    }
                }
            }
        }
        // 2. DRAW UI
        match self.phase {
            GamePhase::Playing => {
                // PROGRESS BAR (aligned with board top)
                let bar_x    = layout.ui_panel_x;
                let bar_y    = layout.grid_offset_y;
                let bar_width = layout.ui_panel_width * 0.92;
                let row_h    = layout.tile_size * 0.5;
                let prev_threshold = self.target - LEVEL_TARGET_STEP;
                let level_progress = (self.total_points - prev_threshold) as f32 / LEVEL_TARGET_STEP as f32;

                draw_rectangle(bar_x, bar_y, bar_width, row_h * 0.27, GRAY);
                draw_rectangle(bar_x, bar_y, bar_width * level_progress.clamp(0.0, 1.0), row_h * 0.27, GOLD);

                let font_lg = (row_h * 1.05).max(18.0);
                let font_sm = (row_h * 0.88).max(15.0);
                draw_text(&format!("Level {}", self.level),                          bar_x, bar_y + row_h * 1.0, font_lg, WHITE);
                draw_text(&format!("Leaves: {}", self.get_leaves_wallet()),          bar_x, bar_y + row_h * 2.1, font_lg, GOLD);
                draw_text(&format!("Illicit Move Cost: {}", self.illegal_move_cost), bar_x, bar_y + row_h * 3.1, font_sm, ORANGE);

                let inv_title_y = bar_y + row_h * 4.25;
                draw_text(
                    &format!("Inventory {}/8", self.inventory_used_slots()),
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
                    ("CAN", self.inventory_count(ItemType::WateringCan), color_u8!(55, 125, 210, 255)),
                    ("SUN", self.inventory_count(ItemType::SeedDay), color_u8!(210, 165, 35, 255)),
                    ("MON", self.inventory_count(ItemType::SeedNight), color_u8!(100, 125, 220, 255)),
                    ("ESS", self.inventory_count(ItemType::MoonbloomEssence), color_u8!(160, 90, 220, 255)),
                    ("FERT", self.inventory_count(ItemType::Fertilizer), color_u8!(185, 55, 55, 255)),
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

                // FARMING BUTTON
                let (visit_x, visit_y, visit_w, visit_h) = Self::playing_visit_garden_button_rect(&layout);
                draw_rectangle(visit_x, visit_y, visit_w, visit_h, color_u8!(36, 80, 42, 255));
                draw_rectangle_lines(visit_x, visit_y, visit_w, visit_h, 3.0, color_u8!(152, 230, 160, 255));
                draw_text("VISIT GARDEN", visit_x + visit_w * 0.13, visit_y + visit_h * 0.68, font_lg, WHITE);

                if self.is_farming {
                    let (btn_x, btn_y, btn_w, btn_h) = Self::playing_descend_button_rect(&layout);
                    draw_rectangle(btn_x, btn_y, btn_w, btn_h, DARKGREEN);
                    draw_rectangle_lines(btn_x, btn_y, btn_w, btn_h, 3.0, GREEN);
                    draw_text("DESCEND >", btn_x + btn_w * 0.1, btn_y + btn_h * 0.68, font_lg, WHITE);
                }
            },
            GamePhase::LevelTransition => {
                let sw = screen_width();
                let sh = screen_height();
                draw_text("FOG CLEARED",                  sw * 0.19, sh * 0.43, (sh * 0.103).max(36.0), GREEN);
                draw_text("[ENTER] Descend (Next Level)", sw * 0.15, sh * 0.55, (sh * 0.052).max(22.0), WHITE);
                draw_text("[F] Farm (Stay & Collect)",    sw * 0.15, sh * 0.62, (sh * 0.052).max(22.0), GOLD);
            },
            _ => {}
        }
    }
}