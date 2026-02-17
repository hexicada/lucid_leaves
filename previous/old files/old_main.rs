use macroquad::prelude::*;


const GRID_WIDTH: usize = 8;
const GRID_HEIGHT: usize = 8;
const TILE_SIZE: f32 = 64.0;
const GRID_OFFSET_X: f32 = 100.0;
const GRID_OFFSET_Y: f32 = 100.0;

// The flavors of our "Sunflower Goth" tiles
#[derive(Clone, Copy, PartialEq, Debug)]
enum TileType {
    Sun,    // Yellow
    Moon,   // Blue
    Skull,  // White
    Leaf,   // Green
    Eye,    // Purple
    Empty,  // For falling logic
}

// The Tile Struct
#[derive(Clone, Copy, Debug)]
struct Tile {
    kind: TileType,
    // We will add "status" here later for Vines/Ice
    is_vined: bool, 
    // For animation (shimmer/drop offset)
    offset_y: f32,
}

impl Tile {
    fn new_random() -> Self {
        // We use Macroquad's built-in gen_range(min, max)
        let kind = match rand::gen_range(0, 5) {
            0 => TileType::Sun,
            1 => TileType::Moon,
            2 => TileType::Skull,
            3 => TileType::Leaf,
            _ => TileType::Eye,
        };
        
        Tile {
            kind,
            is_vined: false,
            offset_y: 0.0,
        }
    }

    // A helper to get the color until we have sprites
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

struct GameState {
    grid: [[Tile; GRID_HEIGHT]; GRID_WIDTH],
    // Selected tile for swapping (Grid Coords)
    selected: Option<(usize, usize)>,
    score: i32,
    target: i32,       // The Goal for this Level
    level: i32,        
    turns_taken: i32
}

impl GameState {
    fn new() -> Self {
        let mut grid = [[Tile { kind: TileType::Empty, is_vined: false, offset_y: 0.0 }; GRID_HEIGHT]; GRID_WIDTH];
        
        // Initial Fill
        for x in 0..GRID_WIDTH {
            for y in 0..GRID_HEIGHT {
                grid[x][y] = Tile::new_random();
            }
        }
       
        GameState {
            grid,
            selected: None,
            score: 0,
            target: 1300,
            level: 1,
            turns_taken: 0,
        }
    }
    fn resolve_matches(&mut self) -> bool { // Return true if we found matches
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

        // 3. Deduplicate! (Remove duplicates so we don't count the same tile twice)
        to_remove.sort();
        to_remove.dedup();

        // 4. Scoring
        if to_remove.is_empty() {
            return false;
        }

        // 10 Points per tile
        let points = to_remove.len() as i32 * 10;
        self.score += points;
        println!("Matches found! +{} points", points); // Debug info

        // 5. Delete them
        for (rx, ry) in to_remove {
            self.grid[rx][ry].kind = TileType::Empty;
        }

        true
    }
    fn apply_gravity(&mut self) {
        for x in 0..GRID_WIDTH {
            // 1. Collect all the non-empty tiles in this column
            let mut valid_tiles = vec![];
            for y in 0..GRID_HEIGHT {
                if self.grid[x][y].kind != TileType::Empty {
                    valid_tiles.push(self.grid[x][y]);
                }
            }

            // 2. Calculate how many we need to spawn
            let missing_count = GRID_HEIGHT - valid_tiles.len();

            // 3. Fill the top with NEW random tiles
            for y in 0..missing_count {
                self.grid[x][y] = Tile::new_random();
                // (Optional: Set y-offset high here later for falling animation)
            }

            // 4. Place the existing tiles back in, below the new ones
            for (i, tile) in valid_tiles.into_iter().enumerate() {
                self.grid[x][missing_count + i] = tile;
            }
        }
    }
    fn update(&mut self) {
        // MOUSE INPUT LOGIC
        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            
            // Convert screen pixels to grid index
            let gx = ((mx - GRID_OFFSET_X) / TILE_SIZE).floor() as isize;
            let gy = ((my - GRID_OFFSET_Y) / TILE_SIZE).floor() as isize;

            // Check bounds
            if gx >= 0 && gx < GRID_WIDTH as isize && gy >= 0 && gy < GRID_HEIGHT as isize {
                let gx = gx as usize;
                let gy = gy as usize;

                match self.selected {
                    None => {
                        // Select the first tile
                        self.selected = Some((gx, gy));
                    }
                  Some((sx, sy)) => {
                        let dx = (gx as isize - sx as isize).abs();
                        let dy = (gy as isize - sy as isize).abs();

                        // Check if neighbor
                        if (dx == 1 && dy == 0) || (dx == 0 && dy == 1) {
                            // It is a valid swap!
                            self.turns_taken += 1; // Increment turn counter

                            // 1. Perform the Swap
                            let temp = self.grid[sx][sy];
                            self.grid[sx][sy] = self.grid[gx][gy];
                            self.grid[gx][gy] = temp;
        
        // 2. THE CASCADE LOOP
        loop {
            if !self.resolve_matches() {
                break;
            }
            self.apply_gravity();
        }

        self.selected = None;
                        } else {
                            // Clicked far away or same tile -> Update selection
                            self.selected = Some((gx, gy));
                        }
                    }
                }
            } else {
                // Clicked outside grid
                self.selected = None;
            }
        }
    }

    // 2. Check for Level Up
    if self.score >= self.target {
        // LEVEL UP!
        self.level += 1;
        self.score = 0;
        self.target = (self.target as f32 * 1.2) as i32;
        
        // Optional: Reshuffle board? 
        // For now, let's keep the board as-is (continuous gardening)
        println!("Level {} Reached! New Target: {}", self.level, self.target);
    }
    fn draw(&self) {
        for x in 0..GRID_WIDTH {
            for y in 0..GRID_HEIGHT {
                let tile = &self.grid[x][y];
                
                let draw_x = GRID_OFFSET_X + x as f32 * TILE_SIZE;
                let draw_y = GRID_OFFSET_Y + y as f32 * TILE_SIZE + tile.offset_y;

                // Draw Tile Background
                draw_rectangle(draw_x, draw_y, TILE_SIZE - 2.0, TILE_SIZE - 2.0, tile.get_color());

                // Draw Selection Highlight
                if let Some((sx, sy)) = self.selected {
                    if sx == x && sy == y {
                        draw_rectangle_lines(draw_x, draw_y, TILE_SIZE - 2.0, TILE_SIZE - 2.0, 4.0, WHITE);
                    }
                }
            }
        }
    }
}

#[macroquad::main("Lucid Leaves")]
async fn main() {
    // Seed the RNG with the current time so every run is unique
    rand::srand(macroquad::miniquad::date::now() as u64);

    let mut game = GameState::new();

    loop {
        clear_background(BLACK);

        game.update();
        game.draw();
        
        // Draw Score
        draw_text(&format!("Leaves: {}", game.score), 20.0, 30.0, 30.0, WHITE);

        // ... Inside the drawing loop ...
    
    // DRAW UI
    let ui_y = 30.0;
    
    // Level Text
    draw_text(&format!("Level {}", game.level), 20.0, ui_y, 30.0, WHITE);
    
    // The Progress Bar Background (Gray Bar)
    let bar_width = 300.0;
    let bar_height = 20.0;
    let bar_x = 150.0;
    let bar_y = 10.0;
    draw_rectangle(bar_x, bar_y, bar_width, bar_height, GRAY);
    
    // The Fill (Yellow/Gold Bar)
    let progress = (game.score as f32 / game.target as f32).clamp(0.0, 1.0);
    draw_rectangle(bar_x, bar_y, bar_width * progress, bar_height, GOLD);
    
    // The Text over the bar
    draw_text(&format!("{}/{}", game.score, game.target), bar_x + 10.0, ui_y - 5.0, 20.0, BLACK);
        
        next_frame().await
    }
}
