// 1. CLEARING
// Any tile in a match group gets its kind set to TileType::Empty.

// 2. GRAVITY (The Shuffle)
// For each column:
//   Scan from bottom to top.
//   If you find an Empty slot:
//     Look for the first Non-Empty tile above it.
//     Move that Tile's data into the Empty slot.
//     Set the Tile's visual 'offset_y' to -(distance_moved * tile_size).
//     Set the old top slot to Empty.

// 3. FILLING (The Materialization)
// For any slot that remains Empty:
//   Spawn a new random Tile.
//   Set its 'offset_y' to be way above the screen (e.g., -TILE_SIZE * 5).
//   This makes it "rain" into the grid.