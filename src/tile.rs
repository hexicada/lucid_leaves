use macroquad::prelude::*;
use rand::gen_range;

// We use 'pub' to make these visible to other files
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TileType {
    Sun,
    Moon,
    Skull,
    Leaf,
    Exotic, // The shape-shifter
    Empty,
}

#[derive(Clone, Copy, Debug)]
pub struct Tile {
    pub kind: TileType,
    pub offset_y: f32,
}

impl Tile {
    pub fn new_random() -> Self {
        let kind = match gen_range(0, 5) {
            0 => TileType::Sun,
            1 => TileType::Moon,
            2 => TileType::Skull,
            3 => TileType::Leaf,
            _ => TileType::Exotic,
        };
        Tile { kind, offset_y: 0.0 }
    }

    pub fn get_color(&self, level: i32) -> Color {
        match self.kind {
            TileType::Sun => GOLD,
            TileType::Moon => SKYBLUE,
            TileType::Skull => BEIGE,
            TileType::Leaf => LIME,
            TileType::Empty => BLANK,
            
            // EXOTIC LOGIC: Changes based on the Level Set
            TileType::Exotic => {
                let set_index = (level - 1) / 3; // Change every 3 levels
                match set_index {
                    0 => PINK,      // Garden (Rose)
                    1 => PURPLE,    // Thicket (Mushroom)
                    _ => RED,       // Void (Eye)
                }
            }
        }
    }
}