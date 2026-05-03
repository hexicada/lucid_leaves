use macroquad::prelude::*;
use macroquad::rand::gen_range;
use crate::economy;
use crate::tile::*; // Import our neighbor, the Tile
use crate::inventory::{Inventory, ItemType};
use crate::shop::Shop;
use crate::ui_layout::{
    Layout,
    point_in_rect,
    playing_visit_garden_button_rect,
    playing_descend_button_rect,
    garden_return_button_rect,
    garden_hunt_button_rect,
    hunt_return_button_rect,
};

pub const GRID_WIDTH: usize = 8;
pub const GRID_HEIGHT: usize = 8;
pub const TILE_SIZE: f32 = 64.0;
pub const LEVELS_PER_SET: i32 = 3; // Shop appears every 3 levels
pub const LEVEL_TARGET_STEP: i32 = 1200;
pub const MATCH_CLEAR_DELAY: f32 = 0.12;
// --- Leaf sway constants (Forest Floor biome) ---
pub const LEAF_SWAY_SPEED: f32    = 0.55;  // Hz — primary oscillator
pub const LEAF_SWAY_SPEED_2: f32  = 0.85;  // Hz — secondary oscillator
pub const LEAF_SWAY_AMP_DEG: f32  = 1.2;   // max rotation in degrees
pub const LEAF_SWAY_PIVOT_NX: f32 = 0.54;  // normalised x of branch pivot
pub const LEAF_SWAY_PIVOT_NY: f32 = 0.37;  // normalised y of branch pivot
pub const LEAF_AUX_SWAY_AMP_DEG: f32 = 0.8;
pub const LEAF_AUX_SWAY_PHASE: f32 = 1.35;
pub const LEAF_AUX_SWAY_PIVOT_NX: f32 = 0.18;
pub const LEAF_AUX_SWAY_PIVOT_NY: f32 = 0.18;

// --- Garden isometric grid positioning ---
// Left lobe grid
pub const ISO_TILE_HW: f32         = 48.0; // tile half-width in pixels
pub const ISO_TILE_HH: f32         = 32.0; // tile half-height in pixels

pub const ISO_LEFT_ORIGIN_NX: f32  = 0.28; // normalized screen x - left lobe top corner
pub const ISO_LEFT_ORIGIN_NY: f32  = 0.38; // normalized screen y - left lobe top corner

// Right lobe grid
pub const ISO_RIGHT_ORIGIN_NX: f32 = 0.68; // normalized screen x - right lobe top corner
pub const ISO_RIGHT_ORIGIN_NY: f32 = 0.38; // normalized screen y - right lobe top corner

pub const ISO_DOT_RADIUS: f32      = 4.0;  // radius of positioning dots
pub const ILLEGAL_MOVE_COST_START: i32 = 100;
pub const ILLEGAL_MOVE_COST_STEP: i32 = 100;

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

/// Gem textures for a single biome. Biomes (in order):
/// 0 - Forest Floor | 1 - Deep Cave    | 2 - Volcanic Rift | 3 - Frozen Tundra
/// 4 - Ocean Trench | 5 - Sky Realm    | 6 - Fungal Wastes  | 7 - Celestial Void
pub struct BiomeTextures {
    pub sun:    Texture2D, // Gem 0 — e.g. Beryl / Crystal / Magma ...
    pub moon:   Texture2D, // Gem 1
    pub leaf:   Texture2D, // Gem 2
    pub exotic: Texture2D, // Gem 3
    pub water:  Texture2D, // Gem 4
    pub overlay: Option<Texture2D>, // Optional board overlay (scaled to board bounds)
}

#[derive(Clone, Copy)]
pub struct GemParticle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub life: f32,
    pub max_life: f32,
    pub size: f32,
    pub color: Color,
}

pub struct GameState {
    pub grid: [[Tile; GRID_HEIGHT]; GRID_WIDTH],
    pub selected: Option<(usize, usize)>,

    // Asset Storage
    pub biome_sets: Vec<BiomeTextures>,
    pub garden_bg_texture: Texture2D,
    pub leaves_main_texture: Option<Texture2D>,
    pub leaves_aux_texture: Option<Texture2D>,

    // Match juice state
    pub pending_matches: Vec<(usize, usize, TileType)>,
    pub particles: Vec<GemParticle>,
    pub clear_timer: f32,
    pub cascade_pulse: f32,
    pub pulse_color: Color,
    pub clear_was_cascade: bool,
    
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
    pub fn new(
        biome_sets: Vec<BiomeTextures>,
        garden_bg_texture: Texture2D,
    ) -> Self {
        debug_assert!(!biome_sets.is_empty(), "biome_sets must not be empty — GameState::new requires at least one BiomeTextures entry");
        let mut grid = [[Tile { kind: TileType::Empty, offset_y: 0.0 }; GRID_HEIGHT]; GRID_WIDTH];
        for x in 0..GRID_WIDTH { for y in 0..GRID_HEIGHT { grid[x][y] = Tile::new_random(); }}

        let mut game = GameState {
            grid,
            selected: None,
            biome_sets,
            garden_bg_texture,
            leaves_main_texture: None,
            leaves_aux_texture: None,
            pending_matches: vec![],
            particles: vec![],
            clear_timer: 0.0,
            cascade_pulse: 0.0,
            pulse_color: WHITE,
            clear_was_cascade: false,
            total_points: 0,
            spent_points: 0,
            target: LEVEL_TARGET_STEP,
            level: 1,
            phase: GamePhase::Playing,
            illegal_move_cost: ILLEGAL_MOVE_COST_START,
            inventory: Inventory::new(),
            shop: Shop::new(),
            is_farming: false,
        };

        // Clear any initial matches immediately so the opening board starts stable.
        loop {
            let matches = game.find_matches();
            if matches.is_empty() {
                break;
            }
            game.clear_matches_immediately(matches);
            game.apply_gravity();
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

    fn tile_particle_color(kind: TileType) -> Color {
        match kind {
            TileType::Sun => color_u8!(255, 215, 110, 255),
            TileType::Moon => color_u8!(180, 205, 255, 255),
            TileType::Water => color_u8!(90, 170, 255, 255),
            TileType::Leaf => color_u8!(150, 235, 150, 255),
            TileType::Exotic => color_u8!(230, 140, 255, 255),
            TileType::Empty => WHITE,
        }
    }

    fn is_clearing(&self) -> bool {
        !self.pending_matches.is_empty()
    }

    fn pending_match_kind_at(&self, x: usize, y: usize) -> Option<TileType> {
        self.pending_matches
            .iter()
            .find(|(mx, my, _)| *mx == x && *my == y)
            .map(|(_, _, kind)| *kind)
    }

    fn find_matches(&self) -> Vec<(usize, usize, TileType)> {
        let mut to_remove = vec![];

        for y in 0..GRID_HEIGHT { for x in 0..GRID_WIDTH - 2 {
            let t1 = self.grid[x][y].kind; let t2 = self.grid[x+1][y].kind; let t3 = self.grid[x+2][y].kind;
            if t1 != TileType::Empty && t1 == t2 && t2 == t3 {
                to_remove.push((x, y, t1));
                to_remove.push((x + 1, y, t2));
                to_remove.push((x + 2, y, t3));
            }
        }}

        for x in 0..GRID_WIDTH { for y in 0..GRID_HEIGHT - 2 {
            let t1 = self.grid[x][y].kind; let t2 = self.grid[x][y+1].kind; let t3 = self.grid[x][y+2].kind;
            if t1 != TileType::Empty && t1 == t2 && t2 == t3 {
                to_remove.push((x, y, t1));
                to_remove.push((x, y + 1, t2));
                to_remove.push((x, y + 2, t3));
            }
        }}

        to_remove.sort_by_key(|(x, y, _)| (*x, *y));
        to_remove.dedup_by_key(|(x, y, _)| (*x, *y));
        to_remove
    }

    fn spawn_match_particles(&mut self, matches: &[(usize, usize, TileType)]) {
        let layout = Layout::compute(GRID_WIDTH, GRID_HEIGHT);
        for (x, y, kind) in matches {
            let center_x = layout.grid_offset_x + *x as f32 * layout.tile_size + layout.tile_size * 0.5;
            let center_y = layout.grid_offset_y + *y as f32 * layout.tile_size + self.grid[*x][*y].offset_y + layout.tile_size * 0.5;
            let color = Self::tile_particle_color(*kind);
            for _ in 0..8 {
                let life = gen_range(0.18, 0.35);
                self.particles.push(GemParticle {
                    x: center_x + gen_range(-8.0, 8.0),
                    y: center_y + gen_range(-8.0, 8.0),
                    vx: gen_range(-18.0, 18.0),
                    vy: gen_range(-34.0, -10.0),
                    life,
                    max_life: life,
                    size: gen_range(3.0, 7.0),
                    color,
                });
            }
        }
    }

    fn begin_match_clear(&mut self, matches: Vec<(usize, usize, TileType)>, is_cascade: bool) {
        if matches.is_empty() {
            return;
        }

        self.clear_timer = MATCH_CLEAR_DELAY;
        self.clear_was_cascade = is_cascade;
        self.pulse_color = Self::tile_particle_color(matches[0].2);
        if matches.len() >= 4 || is_cascade {
            self.cascade_pulse = 1.0;
        }
        self.spawn_match_particles(&matches);
        self.pending_matches = matches;
    }

    fn clear_matches_immediately(&mut self, matches: Vec<(usize, usize, TileType)>) {
        if matches.is_empty() {
            return;
        }

        self.total_points += matches.len() as i32 * 10;
        for (x, y, kind) in matches {
            self.grid[x][y].kind = TileType::Empty;
            economy::roll_resource_drop(
                kind,
                &mut self.inventory,
                &mut self.shop,
                &mut self.total_points,
            );
        }
    }

    fn finalize_match_clear(&mut self) {
        if self.pending_matches.is_empty() {
            return;
        }

        let matches = std::mem::take(&mut self.pending_matches);
        self.clear_timer = 0.0;
        self.total_points += matches.len() as i32 * 10;

        for (x, y, kind) in matches {
            self.grid[x][y].kind = TileType::Empty;
            economy::roll_resource_drop(
                kind,
                &mut self.inventory,
                &mut self.shop,
                &mut self.total_points,
            );
        }

        self.apply_gravity();

        let next_matches = self.find_matches();
        if !next_matches.is_empty() {
            self.begin_match_clear(next_matches, true);
        } else {
            self.clear_was_cascade = false;
        }
    }

    fn update_match_effects(&mut self, delta: f32) {
        for particle in &mut self.particles {
            particle.life -= delta;
            particle.x += particle.vx * delta;
            particle.y += particle.vy * delta;
            particle.vx *= 0.98;
            particle.vy -= 4.0 * delta;
        }
        self.particles.retain(|particle| particle.life > 0.0);
        self.cascade_pulse = (self.cascade_pulse - delta * 4.0).max(0.0);
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

    fn update_playing(&mut self) {
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

    fn handle_playing_click(&mut self, mx: f32, my: f32, layout: &Layout) {
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

    fn handle_board_selection(&mut self, gx: usize, gy: usize) {
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

    fn resolve_swap(&mut self, sx: usize, sy: usize, gx: usize, gy: usize) {
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

    fn update_level_transition(&mut self) {
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

    fn update_shop(&mut self) {
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

    fn update_garden(&mut self) {
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

    fn update_hunt(&mut self) {
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

    pub fn update(&mut self) {
        match self.phase {
            GamePhase::Playing => self.update_playing(),
            GamePhase::LevelTransition => self.update_level_transition(),
            GamePhase::Shop => self.update_shop(),
            GamePhase::Garden => self.update_garden(),
            GamePhase::Hunt => self.update_hunt(),
            GamePhase::BossHunt => {}
        }
    }

    pub fn draw(&self) {
        // Biome Background Logic
        let set_index = (self.level - 1) / LEVELS_PER_SET;
        let bg_color = match set_index { 0 => BLACK, 1 => BLACK, _ => color_u8!(30, 0, 0, 255) };
        clear_background(bg_color);
    let layout = Layout::compute(GRID_WIDTH, GRID_HEIGHT);

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
            draw_texture_ex(
                &self.garden_bg_texture,
                0.0,
                0.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(sw, sh)),
                    ..Default::default()
                },
            );

            // Isometric plot positioning dots
            let origins = [
                (sw * ISO_LEFT_ORIGIN_NX, sh * ISO_LEFT_ORIGIN_NY),
                (sw * ISO_RIGHT_ORIGIN_NX, sh * ISO_RIGHT_ORIGIN_NY),
            ];
            for (origin_x, origin_y) in origins {
                for gy in 0..5usize {
                    for gx in 0..5usize {
                        let sx = origin_x + (gx as f32 - gy as f32) * ISO_TILE_HW;
                        let sy = origin_y + (gx as f32 + gy as f32) * ISO_TILE_HH;
                        draw_circle(sx, sy, ISO_DOT_RADIUS, color_u8!(255, 20, 147, 240));
                    }
                }
            }

            draw_rectangle(0.0, 0.0, sw, sh * 0.12, color_u8!(14, 28, 14, 175));
            draw_text("THE GARDEN", sw * 0.03, sh * 0.08, (sh * 0.07).max(30.0), color_u8!(220, 245, 185, 255));
            draw_text("Rest phase: tend plots and manage resources", sw * 0.34, sh * 0.08, (sh * 0.035).max(18.0), WHITE);

            let (rx, ry, rw, rh) = garden_return_button_rect();
            draw_rectangle(rx, ry, rw, rh, color_u8!(52, 80, 58, 255));
            draw_rectangle_lines(rx, ry, rw, rh, 3.0, color_u8!(170, 225, 170, 255));
            draw_text("RETURN TO PUZZLE", rx + rw * 0.08, ry + rh * 0.62, (rh * 0.42).max(20.0), WHITE);

            let (hx, hy, hw, hh) = garden_hunt_button_rect();
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

            let (bx, by, bw, bh) = hunt_return_button_rect();
            draw_rectangle(bx, by, bw, bh, color_u8!(85, 45, 44, 255));
            draw_rectangle_lines(bx, by, bw, bh, 3.0, color_u8!(255, 190, 170, 255));
            draw_text("BACK TO GARDEN", bx + bw * 0.11, by + bh * 0.62, (bh * 0.42).max(20.0), WHITE);
            return;
        }

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
        let set_idx = ((self.level - 1) / LEVELS_PER_SET) as usize;
        let is_cave_biome = set_idx == 1;
        let gems = &self.biome_sets[set_idx.min(self.biome_sets.len() - 1)];
        let clear_progress = if self.is_clearing() {
            (1.0 - self.clear_timer / MATCH_CLEAR_DELAY).clamp(0.0, 1.0)
        } else {
            0.0
        };

        for x in 0..GRID_WIDTH {
            for y in 0..GRID_HEIGHT {
                let tile = &self.grid[x][y];
                let draw_x = layout.grid_offset_x + x as f32 * layout.tile_size;
                let draw_y = layout.grid_offset_y + y as f32 * layout.tile_size + tile.offset_y;
                let matched = self.pending_match_kind_at(x, y).is_some();
                let scale = if matched { 1.0 - 0.18 * clear_progress } else { 1.0 };
                let alpha = if matched { 1.0 - clear_progress } else { 1.0 };
                let flash = if matched && clear_progress < 0.18 {
                    1.0 - clear_progress / 0.18
                } else {
                    0.0
                };

                if matched {
                    let glow_color = Self::tile_particle_color(tile.kind);
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
                                    sprite_size as f32
                                )),
                                ..Default::default()
                            },
                        );
                    },
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
                                    sprite_size as f32
                                )),
                                ..Default::default()
                            },
                        );
                    },
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
                                    sprite_size as f32
                                )),
                                ..Default::default()
                            },
                        );
                    },
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
                                    sprite_size as f32
                                )),
                                ..Default::default()
                            },
                        );
                    },
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
                                    sprite_size as f32
                                )),
                                ..Default::default()
                            },
                        );
                    },
                    _ => {
                        let base = tile.get_color(self.level);
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
        if let Some((sx, sy)) = self.selected {
            if self.pending_match_kind_at(sx, sy).is_none() {
                let draw_x = layout.grid_offset_x + sx as f32 * layout.tile_size;
                let draw_y = layout.grid_offset_y + sy as f32 * layout.tile_size + self.grid[sx][sy].offset_y;
                draw_rectangle_lines(draw_x, draw_y, layout.tile_size - 2.0, layout.tile_size - 2.0, 4.0, WHITE);
            }
        }

        if self.cascade_pulse > 0.0 {
            let board_w = layout.tile_size * GRID_WIDTH as f32;
            let board_h = layout.tile_size * GRID_HEIGHT as f32;
            draw_rectangle(
                layout.grid_offset_x,
                layout.grid_offset_y,
                board_w,
                board_h,
                Color::new(self.pulse_color.r, self.pulse_color.g, self.pulse_color.b, 0.08 * self.cascade_pulse),
            );
        }

        for particle in &self.particles {
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
            if let Some(ref leaves_tex) = self.leaves_main_texture {
                let sw = screen_width();
                let sh = screen_height();
                let t = get_time() as f32;
                let tau = std::f32::consts::TAU;

                let sway_deg =
                    (t * LEAF_SWAY_SPEED * tau).sin() * LEAF_SWAY_AMP_DEG
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

            if let Some(ref leaves_aux_tex) = self.leaves_aux_texture {
                let sw = screen_width();
                let sh = screen_height();
                let t = get_time() as f32;
                let tau = std::f32::consts::TAU;

                let sway_deg =
                    ((t * LEAF_SWAY_SPEED * tau) + LEAF_AUX_SWAY_PHASE).sin() * LEAF_AUX_SWAY_AMP_DEG
                    + ((t * LEAF_SWAY_SPEED_2 * tau) + LEAF_AUX_SWAY_PHASE * 0.61).sin() * LEAF_AUX_SWAY_AMP_DEG * 0.42;
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
                    &format!("Inventory {}/8", economy::inventory_used_slots(&self.inventory)),
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
                    ("CAN", economy::inventory_count(&self.inventory, ItemType::WateringCan), color_u8!(55, 125, 210, 255)),
                    ("SUN", economy::inventory_count(&self.inventory, ItemType::SeedDay), color_u8!(210, 165, 35, 255)),
                    ("MON", economy::inventory_count(&self.inventory, ItemType::SeedNight), color_u8!(100, 125, 220, 255)),
                    ("ESS", economy::inventory_count(&self.inventory, ItemType::MoonbloomEssence), color_u8!(160, 90, 220, 255)),
                    ("FERT", economy::inventory_count(&self.inventory, ItemType::Fertilizer), color_u8!(185, 55, 55, 255)),
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
                let (visit_x, visit_y, visit_w, visit_h) = playing_visit_garden_button_rect(&layout);
                draw_rectangle(visit_x, visit_y, visit_w, visit_h, color_u8!(36, 80, 42, 255));
                draw_rectangle_lines(visit_x, visit_y, visit_w, visit_h, 3.0, color_u8!(152, 230, 160, 255));
                draw_text("VISIT GARDEN", visit_x + visit_w * 0.13, visit_y + visit_h * 0.68, font_lg, WHITE);

                if self.is_farming {
                    let (btn_x, btn_y, btn_w, btn_h) = playing_descend_button_rect(&layout);
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