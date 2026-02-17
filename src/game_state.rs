use macroquad::prelude::*;
use crate::tile::*; // Import our neighbor, the Tile

pub const GRID_WIDTH: usize = 8;
pub const GRID_HEIGHT: usize = 8;
pub const TILE_SIZE: f32 = 64.0;
pub const GRID_OFFSET_X: f32 = 100.0;
pub const GRID_OFFSET_Y: f32 = 100.0;
pub const LEVELS_PER_SET: i32 = 3; // Shop appears every 3 levels

#[derive(PartialEq)]
pub enum GamePhase {
    Playing,
    LevelTransition, // The "Fog Cleared" choice screen
    Shop,            // The Tarquin Screen
}

pub struct GameState {
    pub grid: [[Tile; GRID_HEIGHT]; GRID_WIDTH],
    pub selected: Option<(usize, usize)>,

    // Asset Storage
    pub beryl_texture: Texture2D,
    
    // --- THE BACKEND VARIABLES ---
    pub total_points: i32, // Lifetime score (The "Level" Bar)
    pub spent_points: i32, // Spent at shop
    
    pub target: i32,       // Points needed to clear fog
    pub level: i32,
    pub phase: GamePhase,
    
    // NEW FLAG: Are we in "Overtime"?
    pub is_farming: bool,
}

impl GameState {
    pub fn new(beryl_texture: Texture2D) -> Self {
        let mut grid = [[Tile { kind: TileType::Empty, offset_y: 0.0 }; GRID_HEIGHT]; GRID_WIDTH];
        for x in 0..GRID_WIDTH { for y in 0..GRID_HEIGHT { grid[x][y] = Tile::new_random(); }}

        let mut game = GameState {
            grid,
            selected: None,
            total_points: 0,
            spent_points: 0,
            target: 1000,
            level: 1,
            phase: GamePhase::Playing,
            is_farming: false, // Start normally
            beryl_texture,
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
        
        for (rx, ry) in to_remove { self.grid[rx][ry].kind = TileType::Empty; }
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

                // 1. MOUSE LOGIC FOR GRID
                if is_mouse_button_pressed(MouseButton::Left) {
                    let (mx, my) = mouse_position();
                    
                    // CHECK IF CLICKING DESCEND BUTTON (Only if farming)
                    let mut clicked_button = false;
                    if self.is_farming {
                        if mx >= 150.0 && mx <= 350.0 && my >= 650.0 && my <= 700.0 {
                             self.phase = GamePhase::LevelTransition;
                             clicked_button = true;
                        }
                    }

                    if !clicked_button {
                        // GRID LOGIC
                        let gx = ((mx - GRID_OFFSET_X) / TILE_SIZE).floor() as isize;
                        let gy = ((my - GRID_OFFSET_Y) / TILE_SIZE).floor() as isize;

                        if gx >= 0 && gx < GRID_WIDTH as isize && gy >= 0 && gy < GRID_HEIGHT as isize {
                            match self.selected {
                                None => self.selected = Some((gx as usize, gy as usize)),
                                Some((sx, sy)) => {
                                    let gx = gx as usize; let gy = gy as usize;
                                    let dx = (gx as isize - sx as isize).abs(); let dy = (gy as isize - sy as isize).abs();
                                    if (dx == 1 && dy == 0) || (dx == 0 && dy == 1) {
                                        let temp = self.grid[sx][sy]; self.grid[sx][sy] = self.grid[gx][gy]; self.grid[gx][gy] = temp;
                                        // Cascade: resolve matches and apply gravity until stable
                                        loop {
                                            let had_matches = self.resolve_matches();
                                            if had_matches {
                                                self.apply_gravity();
                                            } else {
                                                break;
                                            }
                                        }
                                        self.selected = None;
                                    } else { self.selected = Some((gx, gy)); }
                                }
                            }
                        } else { self.selected = None; }
                    }
                }

                // 2. CHECK LEVEL THRESHOLD
                let current_threshold = self.level * 2000;
                
                // If target met AND we aren't already farming, trigger transition
                if self.total_points >= current_threshold && !self.is_farming {
                    self.phase = GamePhase::LevelTransition;
                }
            }
            
            GamePhase::LevelTransition => {
                // OPTION A: Next Level
                if is_key_pressed(KeyCode::Enter) {
                    if self.level % LEVELS_PER_SET == 0 { 
                        self.phase = GamePhase::Shop; 
                    } else { 
                        self.level += 1; 
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
                if is_key_pressed(KeyCode::Enter) { self.level += 1; self.phase = GamePhase::Playing; }
                // Debug spending
                if is_key_pressed(KeyCode::Space) && self.get_leaves_wallet() >= 500 {
                    self.spent_points += 500;
                }
            }
        }
    }

    pub fn draw(&self) {
        // Biome Background Logic
        let set_index = (self.level - 1) / LEVELS_PER_SET;
        let bg_color = match set_index { 0 => BLACK, 1 => color_u8!(20, 10, 40, 255), _ => color_u8!(30, 0, 0, 255) };
        clear_background(bg_color);

        if self.phase == GamePhase::Shop {
            draw_text("THE SHRINE", 200.0, 100.0, 60.0, PURPLE);
            draw_text(&format!("Leaves: {}", self.get_leaves_wallet()), 250.0, 200.0, 40.0, GOLD);
            draw_text("[SPACE] Spend 500 | [ENTER] Next Biome", 150.0, 400.0, 30.0, WHITE);
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
                let draw_x = GRID_OFFSET_X + x as f32 * TILE_SIZE;
                let draw_y = GRID_OFFSET_Y + y as f32 * TILE_SIZE + tile.offset_y;
                
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
                                dest_size: Some(vec2(TILE_SIZE, TILE_SIZE)),
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
                    
                    _ => {
                        // Draw everything else normally
                        draw_rectangle(draw_x, draw_y, TILE_SIZE-2.0, TILE_SIZE-2.0, tile.get_color(self.level));
                    }
                } // <--- END OF MATCH

                // PART B: DRAW THE SELECTION (Outside the match so it works on ALL tiles)
                if let Some((sx, sy)) = self.selected {
                    if sx == x && sy == y { 
                        draw_rectangle_lines(draw_x, draw_y, TILE_SIZE-2.0, TILE_SIZE-2.0, 4.0, WHITE); 
                    }
                }
            }
        }
        // 2. DRAW UI
        match self.phase {
            GamePhase::Playing => {
                // PROGRESS BAR
                let bar_width = 300.0; let bar_x = 150.0; let bar_y = 10.0;
                let prev_threshold = (self.level - 1) * 2000;
                let level_progress = (self.total_points - prev_threshold) as f32 / 2000.0;
                
                draw_rectangle(bar_x, bar_y, bar_width, 20.0, GRAY);
                draw_rectangle(bar_x, bar_y, bar_width * level_progress.clamp(0.0, 1.0), 20.0, GOLD);
                
                draw_text(&format!("Level {}", self.level), 20.0, 30.0, 30.0, WHITE);
                draw_text(&format!("Leaves: {}", self.get_leaves_wallet()), 20.0, 70.0, 30.0, GOLD);

                // FARMING BUTTON
                if self.is_farming {
                    let btn_x = 150.0; let btn_y = 650.0;
                    draw_rectangle(btn_x, btn_y, 200.0, 50.0, DARKGREEN);
                    draw_rectangle_lines(btn_x, btn_y, 200.0, 50.0, 3.0, GREEN);
                    draw_text("DESCEND >", btn_x + 40.0, btn_y + 35.0, 30.0, WHITE);
                }
            },
            GamePhase::LevelTransition => {
                 draw_text("FOG CLEARED", 150.0, 250.0, 60.0, GREEN);
                 draw_text("[ENTER] Descend (Next Level)", 120.0, 320.0, 30.0, WHITE);
                 draw_text("[F] Farm (Stay & Collect)", 120.0, 360.0, 30.0, GOLD);
            },
            _ => {}
        }
    }
}