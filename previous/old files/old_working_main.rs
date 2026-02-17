use macroquad::prelude::*;

const GRID_WIDTH: usize = 8;
const GRID_HEIGHT: usize = 8;
const TILE_SIZE: f32 = 64.0;
const GRID_OFFSET_X: f32 = 100.0;
const GRID_OFFSET_Y: f32 = 100.0;

// --- THE TILES ---
#[derive(Clone, Copy, PartialEq, Debug)]
enum TileType {
    Sun,    // Yellow
    Moon,   // Blue
    Skull,  // White
    Leaf,   // Green
    Eye,    // Purple
    Empty,  // The Void
}

#[derive(Clone, Copy, Debug)]
struct Tile {
    kind: TileType,
    offset_y: f32, // For future animations
}

impl Tile {
    fn new_random() -> Self {
        let kind = match rand::gen_range(0, 5) {
            0 => TileType::Sun,
            1 => TileType::Moon,
            2 => TileType::Skull,
            3 => TileType::Leaf,
            _ => TileType::Eye,
        };
        
        Tile {
            kind,
            offset_y: 0.0,
        }
    }

    fn get_color(&self) -> Color {
        match self.kind {
            TileType::Sun => GOLD,
            TileType::Moon => BLUE,
            TileType::Skull => BEIGE,
            TileType::Leaf => DARKGREEN,
            TileType::Eye => PURPLE,
            TileType::Empty => BLANK,
        }
    }
}

// --- THE GAME STATE ---
struct GameState {
    grid: [[Tile; GRID_HEIGHT]; GRID_WIDTH],
    selected: Option<(usize, usize)>,
    
    leaves: i32,       // Your Score/Currency
    target: i32,       // The Goal for this Level
    level: i32,        // Current Level
}

impl GameState {
    fn new() -> Self {
        let mut grid = [[Tile { kind: TileType::Empty, offset_y: 0.0 }; GRID_HEIGHT]; GRID_WIDTH];
        
        // Fill grid
        for x in 0..GRID_WIDTH {
            for y in 0..GRID_HEIGHT {
                grid[x][y] = Tile::new_random();
            }
        }

        GameState {
            grid,
            selected: None,
            leaves: 0,
            target: 2000, // Level 1 Goal
            level: 1,
        }
    }

    // LOGIC: Find matches, delete them, return TRUE if found
    fn resolve_matches(&mut self) -> bool {
        let mut to_remove = vec![];

        // 1. Check Horizontal
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH - 2 {
                let t1 = self.grid[x][y].kind;
                let t2 = self.grid[x+1][y].kind;
                let t3 = self.grid[x+2][y].kind;
                
                if t1 != TileType::Empty && t1 == t2 && t2 == t3 {
                    to_remove.push((x, y));
                    to_remove.push((x+1, y));
                    to_remove.push((x+2, y));
                }
            }
        }

        // 2. Check Vertical
        for x in 0..GRID_WIDTH {
            for y in 0..GRID_HEIGHT - 2 {
                let t1 = self.grid[x][y].kind;
                let t2 = self.grid[x][y+1].kind;
                let t3 = self.grid[x][y+2].kind;

                if t1 != TileType::Empty && t1 == t2 && t2 == t3 {
                    to_remove.push((x, y));
                    to_remove.push((x, y+1));
                    to_remove.push((x, y+2));
                }
            }
        }

        // 3. Deduplicate
        to_remove.sort();
        to_remove.dedup();

        if to_remove.is_empty() {
            return false;
        }

        // 4. Score Points (Leaves)
        let points = to_remove.len() as i32 * 10;
        self.leaves += points;
        
        // 5. Delete Tiles
        for (rx, ry) in to_remove {
            self.grid[rx][ry].kind = TileType::Empty;
        }

        true
    }

    // LOGIC: Drop tiles down
    fn apply_gravity(&mut self) {
        for x in 0..GRID_WIDTH {
            let mut valid_tiles = vec![];
            for y in 0..GRID_HEIGHT {
                if self.grid[x][y].kind != TileType::Empty {
                    valid_tiles.push(self.grid[x][y]);
                }
            }

            let missing_count = GRID_HEIGHT - valid_tiles.len();

            // Fill top with new tiles
            for y in 0..missing_count {
                self.grid[x][y] = Tile::new_random();
            }

            // Place old tiles below
            for (i, tile) in valid_tiles.into_iter().enumerate() {
                self.grid[x][missing_count + i] = tile;
            }
        }
    }

    // LOGIC: Main Update Loop
    fn update(&mut self) {
        // --- MOUSE INPUT ---
        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            let gx = ((mx - GRID_OFFSET_X) / TILE_SIZE).floor() as isize;
            let gy = ((my - GRID_OFFSET_Y) / TILE_SIZE).floor() as isize;

            if gx >= 0 && gx < GRID_WIDTH as isize && gy >= 0 && gy < GRID_HEIGHT as isize {
                let gx = gx as usize;
                let gy = gy as usize;

                match self.selected {
                    None => self.selected = Some((gx, gy)),
                    Some((sx, sy)) => {
                        let dx = (gx as isize - sx as isize).abs();
                        let dy = (gy as isize - sy as isize).abs();

                        // If Neighbor -> SWAP
                        if (dx == 1 && dy == 0) || (dx == 0 && dy == 1) {
                            let temp = self.grid[sx][sy];
                            self.grid[sx][sy] = self.grid[gx][gy];
                            self.grid[gx][gy] = temp;
                            
                            // CASCADE LOOP
                            loop {
                                if !self.resolve_matches() {
                                    break;
                                }
                                self.apply_gravity();
                            }

                            self.selected = None;
                        } else {
                            self.selected = Some((gx, gy));
                        }
                    }
                }
            } else {
                self.selected = None;
            }
        }

        // --- LEVEL UP CHECK ---
        if self.leaves >= self.target {
            self.level += 1;
            self.leaves = 0; 
            self.target = (self.target as f32 * 1.2) as i32; // 20% Harder
        }
    }

    // LOGIC: Draw everything
    fn draw(&self) {
        // Draw Grid
        for x in 0..GRID_WIDTH {
            for y in 0..GRID_HEIGHT {
                let tile = &self.grid[x][y];
                let draw_x = GRID_OFFSET_X + x as f32 * TILE_SIZE;
                let draw_y = GRID_OFFSET_Y + y as f32 * TILE_SIZE + tile.offset_y;

                draw_rectangle(draw_x, draw_y, TILE_SIZE - 2.0, TILE_SIZE - 2.0, tile.get_color());

                // Highlight Selection
                if let Some((sx, sy)) = self.selected {
                    if sx == x && sy == y {
                        draw_rectangle_lines(draw_x, draw_y, TILE_SIZE - 2.0, TILE_SIZE - 2.0, 4.0, WHITE);
                    }
                }
            }
        }

        // Draw UI (Progress Bar)
        let ui_y = 30.0;
        draw_text(&format!("Level {}", self.level), 20.0, ui_y, 30.0, WHITE);
        
        let bar_width = 300.0;
        let bar_height = 20.0;
        let bar_x = 150.0;
        let bar_y = 10.0;
        
        // Bar Background
        draw_rectangle(bar_x, bar_y, bar_width, bar_height, GRAY);
        
        // Bar Fill
        let progress = (self.leaves as f32 / self.target as f32).clamp(0.0, 1.0);
        draw_rectangle(bar_x, bar_y, bar_width * progress, bar_height, GOLD);
        
        // Text
        draw_text(&format!("{}/{}", self.leaves, self.target), bar_x + 10.0, ui_y - 5.0, 20.0, BLACK);
    }
}

// --- MAIN ENTRY POINT ---
#[macroquad::main("Lucid Leaves")]
async fn main() {
    rand::srand(macroquad::miniquad::date::now() as u64);
    let mut game = GameState::new();

    loop {
        clear_background(BLACK);

        game.update();
        game.draw();

        next_frame().await
    }
}