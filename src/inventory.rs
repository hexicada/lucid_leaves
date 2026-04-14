#![allow(dead_code)]

pub const INVENTORY_SLOTS: usize = 8;
pub const STACK_MAX: u32 = 10;

// --- Item Types ---

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ItemType {
    // Garden resources (board drops)
    WateringCan,
    SeedDay,            // from Sun tile
    SeedNight,          // from Moon tile
    MoonbloomEssence,   // rare Moon drop
    Fertilizer,         // from Exotic tile

    // Board-modifying items (sold by Bagira)
    BoardModifier(BoardModifierKind),

    // Food/buff items (given by Tarquin)
    FoodBuff(FoodBuffKind),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BoardModifierKind {
    // Placeholders — expand per biome shop sprint
    TileWeightShift,    // shift spawn odds of one tile type
    IllegalCostCap,     // cap illicit move cost at a fixed max this level
    CascadeBonus,       // bonus points on cascades of 4+
    FogClear,           // removes one boss debuff tile from board
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum FoodBuffKind {
    // Placeholders — Tarquin free food items
    MintLeaf,    // small leaf bonus on next match resolution
    HoneyBread,  // temporary drop rate boost for one level
    SpicedNut,   // illicit move cost reduced by 50 for next level
}

// --- Inventory Slot ---

#[derive(Clone, Copy, Debug)]
pub struct InventorySlot {
    pub item: Option<ItemType>,
    pub count: u32, // 0..=STACK_MAX
}

impl InventorySlot {
    pub const EMPTY: Self = Self { item: None, count: 0 };
}

// --- Inventory ---

#[derive(Clone, Debug)]
pub struct Inventory {
    pub slots: [InventorySlot; INVENTORY_SLOTS],
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            slots: [InventorySlot::EMPTY; INVENTORY_SLOTS],
        }
    }

    /// Add one unit of an item. Returns true on success.
    /// Returns false if full — caller routes overflow to Bagira consignment.
    pub fn push(&mut self, item: ItemType) -> bool {
        // Try to stack onto an existing slot first
        for slot in self.slots.iter_mut() {
            if slot.item == Some(item) && slot.count < STACK_MAX {
                slot.count += 1;
                return true;
            }
        }
        // Try an empty slot
        for slot in self.slots.iter_mut() {
            if slot.item.is_none() {
                slot.item = Some(item);
                slot.count = 1;
                return true;
            }
        }
        false
    }

    /// Remove one unit from slot_index. Returns the item type if successful.
    /// Caller routes the returned item to Bagira consignment on player discard.
    pub fn discard_one(&mut self, slot_index: usize) -> Option<ItemType> {
        let slot = &mut self.slots[slot_index];
        if let Some(item) = slot.item {
            slot.count -= 1;
            if slot.count == 0 {
                slot.item = None;
            }
            return Some(item);
        }
        None
    }
}
