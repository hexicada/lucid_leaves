use macroquad::prelude::*;
use crate::tile::*; // Import our neighbor, the Tile

pub const GRID_WIDTH: usize = 8;
pub const GRID_HEIGHT: usize = 8;
pub const TILE_SIZE: f32 = 64.0;
pub const GRID_OFFSET_X: f32 = 100.0;
pub const GRID_OFFSET_Y: f32 = 100.0;
pub const LEVELS_PER_SET: i32 = 3;

#[derive(PartialEq)]
pub enum GamePhase {
    Playing,
    LevelTransition,
    Shop,
}

pub struct GameState {
    pub grid: [[Tile; GRID_HEIGHT]; GRID_WIDTH],
    pub selected: Option<(usize, usize)>,
    
    // --- THE BACKEND VARIABLES ---
    pub total_points: i32, // The "Level" Bar (Lifetime accumulated points)
    pub spent_points: i32, // How much you spent at the shop
    
    pub target: i32,       // Points needed to clear fog
    pub level: i32,
    pub phase: GamePhase,
}

impl GameState {
    pub fn new() -> Self {
        let mut grid = [[Tile { kind: TileType::Empty, offset_y: 0.0 }; GRID_HEIGHT]; GRID_WIDTH];
        for x in 0..GRID_WIDTH { for y in 0..GRID_HEIGHT { grid[x][y] = Tile::new_random(); }}

        GameState {
            grid,
            selected: None,
            total_points: 0,
            spent_points: 0,
            target: 1000,
            level: 1,
            phase: GamePhase::Playing,
        }
    }

    // CALCULATED PROPERTY: This is your "Leaves" balance
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

        // SCORE UPDATE: We add to total_points only.
        // The wallet (leaves) updates automatically because it's calculated from total_points.
        let points = to_remove.len() as i32 * 10;
        self.total_points += points;

        for (rx, ry) in to_remove { self.grid[rx][ry].kind = TileType::Empty; }
        true
    }

    pub fn apply_gravity(&mut self) {
        for x in 0..GRID_WIDTH {
            let mut valid_tiles = vec![];
            for y in 0..GRID_HEIGHT { if self.grid[x][y].kind != TileType::Empty { valid_tiles.push(self.grid[x][y]); }}
            let missing = GRID_HEIGHT - valid_tiles.len();
            for y in 0..missing { self.grid[x][y] = Tile::new_random(); }
            for (i, tile) in valid_tiles.into_iter().enumerate() { self.grid[x][missing + i] = tile; }
        }
    }

    pub fn update(&mut self) {
        match self.phase {
            GamePhase::Playing => {
                if is_mouse_button_pressed(MouseButton::Left) {
                    let (mx, my) = mouse_position();
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
                                    loop { if !self.resolve_matches() { break; } self.apply_gravity(); }
                                    self.selected = None;
                                } else { self.selected = Some((gx, gy)); }
                            }
                        }
                    } else { self.selected = None; }
                }
                
                // CHECK LEVEL THRESHOLD
                let current_threshold = self.level * 2000;
                if self.total_points >= current_threshold {
                    self.phase = GamePhase::LevelTransition;
                }
            }
            GamePhase::LevelTransition => {
                if is_key_pressed(KeyCode::Enter) {
                    if self.level % LEVELS_PER_SET == 0 { self.phase = GamePhase::Shop; } 
                    else { self.level += 1; self.phase = GamePhase::Playing; }
                }
            }
            GamePhase::Shop => {
                if is_key_pressed(KeyCode::Enter) { self.level += 1; self.phase = GamePhase::Playing; }
                // DEBUG SPENDING
                if is_key_pressed(KeyCode::Space) && self.get_leaves_wallet() >= 500 {
                    self.spent_points += 500;
                }
            }
        }
    }

    pub fn draw(&self) {
        let set_index = (self.level - 1) / LEVELS_PER_SET;
        let bg_color = match set_index { 0 => BLACK, 1 => color_u8!(20, 10, 40, 255), _ => color_u8!(30, 0, 0, 255) };
        clear_background(bg_color);

        // Draw Grid
        if self.phase != GamePhase::Shop {
            for x in 0..GRID_WIDTH {
                for y in 0..GRID_HEIGHT {
                    let tile = &self.grid[x][y];
                    let draw_x = GRID_OFFSET_X + x as f32 * TILE_SIZE;
                    let draw_y = GRID_OFFSET_Y + y as f32 * TILE_SIZE + tile.offset_y;
                    
                    // Pass level to get color (for Exotic tile)
                    draw_rectangle(draw_x, draw_y, TILE_SIZE-2.0, TILE_SIZE-2.0, tile.get_color(self.level));
                    
                    if let Some((sx, sy)) = self.selected {
                        if sx == x && sy == y { draw_rectangle_lines(draw_x, draw_y, TILE_SIZE-2.0, TILE_SIZE-2.0, 4.0, WHITE); }
                    }
                }
            }
        }

        // UI
        match self.phase {
            GamePhase::Playing => {
                // PROGRESS BAR
                let bar_width = 300.0; let bar_x = 150.0; let bar_y = 10.0;
                let prev_threshold = (self.level - 1) * 2000;
                let level_progress = (self.total_points - prev_threshold) as f32 / 2000.0;
                
                draw_rectangle(bar_x, bar_y, bar_width, 20.0, GRAY);
                draw_rectangle(bar_x, bar_y, bar_width * level_progress.clamp(0.0, 1.0), 20.0, GOLD);
                draw_text(&format!("Leaves: {}", self.get_leaves_wallet()), 20.0, 70.0, 30.0, GOLD);
            },
            GamePhase::LevelTransition => {
                 draw_text("FOG CLEARED. [ENTER] TO DESCEND.", 100.0, 300.0, 40.0, GREEN);
            },
            GamePhase::Shop => {
                 draw_text("THE SHRINE", 200.0, 100.0, 60.0, PURPLE);
                 draw_text(&format!("Leaves: {}", self.get_leaves_wallet()), 250.0, 200.0, 40.0, GOLD);
                 draw_text("[SPACE] Buy Item (500) | [ENTER] Leave", 150.0, 400.0, 30.0, WHITE);
            }
        }
    }
}