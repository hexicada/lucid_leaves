use macroquad::prelude::*;
use crate::economy;
use crate::match_logic::{self, GemParticle, MatchCell};
use crate::render;
use crate::tile::*; // Import our neighbor, the Tile
use crate::inventory::Inventory;
use crate::shop::Shop;
use crate::garden::Garden;
use crate::ui_layout::{
    Layout,
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

#[derive(Clone, Copy, PartialEq)]
pub enum GardenTool {
    PlantSun,
    PlantMoon,
    PlantEssence,
    Water,
    Fertilize,
}

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

pub struct GameState {
    pub grid: [[Tile; GRID_HEIGHT]; GRID_WIDTH],
    pub selected: Option<(usize, usize)>,

    // Asset Storage
    pub biome_sets: Vec<BiomeTextures>,
    pub garden_bg_texture: Texture2D,
    pub leaves_main_texture: Option<Texture2D>,
    pub leaves_aux_texture: Option<Texture2D>,

    // Match juice state
    pub pending_matches: Vec<MatchCell>,
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
    pub garden: Garden,
    
    // Garden UI tool mode
    pub garden_selected_tool: Option<GardenTool>,
    pub garden_drawer_open: bool,
    
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
            garden: Garden::new(),
            garden_selected_tool: None,
            garden_drawer_open: false,
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

    pub(crate) fn is_clearing(&self) -> bool {
        !self.pending_matches.is_empty()
    }

    pub(crate) fn find_matches(&self) -> Vec<MatchCell> {
        match_logic::find_matches(&self.grid)
    }

    fn spawn_match_particles(&mut self, matches: &[MatchCell]) {
        let layout = Layout::compute(GRID_WIDTH, GRID_HEIGHT);
        match_logic::spawn_match_particles(&self.grid, &mut self.particles, matches, &layout);
    }

    pub(crate) fn begin_match_clear(&mut self, matches: Vec<MatchCell>, is_cascade: bool) {
        if matches.is_empty() {
            return;
        }

        self.clear_timer = MATCH_CLEAR_DELAY;
        self.clear_was_cascade = is_cascade;
        self.pulse_color = match_logic::tile_particle_color(matches[0].2);
        if matches.len() >= 4 || is_cascade {
            self.cascade_pulse = 1.0;
        }
        self.spawn_match_particles(&matches);
        self.pending_matches = matches;
    }

    fn clear_matches_immediately(&mut self, matches: Vec<MatchCell>) {
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

    pub(crate) fn finalize_match_clear(&mut self) {
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

    pub(crate) fn update_match_effects(&mut self, delta: f32) {
        match_logic::update_match_effects(&mut self.particles, &mut self.cascade_pulse, delta);
    }

    #[cfg(feature = "dev")]
    pub(crate) fn save_progress(&self) {
        let data = format!("{}\n{}\n{}\n{}\n",
            self.level,
            self.total_points,
            self.spent_points,
            self.illegal_move_cost,
        );
        let _ = std::fs::write("dev_save.txt", data);
    }

    #[cfg(feature = "dev")]
    pub(crate) fn load_progress(&mut self) {
        if let Ok(data) = std::fs::read_to_string("dev_save.txt") {
            let mut lines = data.lines();
            if let (Some(l), Some(tp), Some(sp), Some(ic)) = (
                lines.next().and_then(|s| s.parse::<i32>().ok()),
                lines.next().and_then(|s| s.parse::<i32>().ok()),
                lines.next().and_then(|s| s.parse::<i32>().ok()),
                lines.next().and_then(|s| s.parse::<i32>().ok()),
            ) {
                self.level         = l;
                self.target        = l * LEVEL_TARGET_STEP;
                self.total_points  = tp;
                self.spent_points  = sp;
                self.illegal_move_cost = ic;
                self.is_farming    = false;
                self.phase         = GamePhase::Playing;
                self.reset_board();
            }
        }
    }

    #[cfg_attr(not(feature = "dev"), allow(dead_code))]
    pub(crate) fn reset_board(&mut self) {
        for x in 0..GRID_WIDTH {
            for y in 0..GRID_HEIGHT {
                self.grid[x][y] = crate::tile::Tile::new_random();
            }
        }
        loop {
            let matches = self.find_matches();
            if matches.is_empty() { break; }
            self.clear_matches_immediately(matches);
            self.apply_gravity();
        }
        self.pending_matches = vec![];
        self.particles = vec![];
        self.clear_timer = 0.0;
        self.selected = None;
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
            render::draw_shop_screen(self.get_leaves_wallet());
            return;
        }

        if self.phase == GamePhase::Garden {
            render::draw_garden_screen(
                &self.garden_bg_texture,
                ISO_LEFT_ORIGIN_NX,
                ISO_LEFT_ORIGIN_NY,
                ISO_RIGHT_ORIGIN_NX,
                ISO_RIGHT_ORIGIN_NY,
                ISO_TILE_HW,
                ISO_TILE_HH,
                ISO_DOT_RADIUS,
                &self.garden,
                &self.inventory,
                self.garden_selected_tool,
                self.garden_drawer_open,
            );
            return;
        }

        if self.phase == GamePhase::Hunt {
            render::draw_hunt_screen();
            return;
        }

        render::draw_board_and_effects(self, &layout);

        // 2. DRAW UI
        match self.phase {
            GamePhase::Playing => {
                render::draw_playing_ui(
                    &layout,
                    self.level,
                    self.target,
                    LEVEL_TARGET_STEP,
                    self.total_points,
                    self.get_leaves_wallet(),
                    self.illegal_move_cost,
                    &self.inventory,
                    self.is_farming,
                );
            },
            GamePhase::LevelTransition => {
                render::draw_level_transition_ui();
            },
            _ => {}
        }
    }
}