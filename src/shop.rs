#![allow(dead_code)]

use std::collections::VecDeque;
use crate::inventory::ItemType;

// --- Economy constants ---

pub const BAGIRA_STOCK_SIZE: usize = 4;          // visible items per visit
pub const BAGIRA_BUYBACK_LIMIT: usize = 12;      // FIFO reserve cap
pub const BAGIRA_BUYBACK_RATE: f32 = 0.35;       // pays 35% of base on consign
pub const BAGIRA_RESELL_RATE: f32 = 1.0;         // resells at 100% of base
pub const TARQUIN_FREE_FOOD_LIMIT: usize = 1;    // free food items per biome visit

// Gamble odds (must sum to 1.0)
pub const TARQUIN_ODDS_COMMON: f32 = 0.60;
pub const TARQUIN_ODDS_UNCOMMON: f32 = 0.30;
pub const TARQUIN_ODDS_RARE: f32 = 0.10;

// Bagira price bands (leaves)
pub const BAGIRA_PRICE_COMMON_MIN: i32 = 200;
pub const BAGIRA_PRICE_COMMON_MAX: i32 = 400;
pub const BAGIRA_PRICE_UNCOMMON_MIN: i32 = 500;
pub const BAGIRA_PRICE_UNCOMMON_MAX: i32 = 900;
pub const BAGIRA_PRICE_RARE_MIN: i32 = 1200;

// --- Shared types ---

#[derive(Clone, Debug)]
pub struct ShopItem {
    pub item: ItemType,
    pub base_price: i32,
}

#[derive(Clone, Debug)]
pub struct BuybackEntry {
    pub item: ItemType,
    pub buyback_price: i32, // what the player pays to reclaim (100% of base)
}

// --- Bagira ---

pub struct BagiraStock {
    pub visible: Vec<ShopItem>,          // up to BAGIRA_STOCK_SIZE items on display
    pub buyback: VecDeque<BuybackEntry>, // FIFO reserve of consigned/discarded items
}

impl BagiraStock {
    pub fn new() -> Self {
        Self {
            visible: Vec::new(),
            buyback: VecDeque::new(),
        }
    }

    /// Called when a player overflows or discards an item.
    /// Quotes a buyback entry into reserve and returns the leaves paid to player.
    pub fn consign(&mut self, item: ItemType, base_price: i32) -> i32 {
        let paid = (base_price as f32 * BAGIRA_BUYBACK_RATE).floor() as i32;
        let buyback_price = (base_price as f32 * BAGIRA_RESELL_RATE).floor() as i32;

        if self.buyback.len() >= BAGIRA_BUYBACK_LIMIT {
            self.buyback.pop_front(); // evict oldest entry (FIFO)
        }
        self.buyback.push_back(BuybackEntry { item, buyback_price });
        paid
    }

    /// Reroll visible stock. Called each biome transition.
    /// Stock generation logic is a placeholder — fill in per sprint.
    pub fn reroll(&mut self) {
        self.visible.clear();
        // TODO: populate with generated items based on current biome/level
    }
}

// --- Tarquin ---

pub struct TarquinStock {
    pub free_food_remaining: usize, // resets to TARQUIN_FREE_FOOD_LIMIT each visit
    pub gamble_available: bool,
}

impl TarquinStock {
    pub fn new() -> Self {
        Self {
            free_food_remaining: TARQUIN_FREE_FOOD_LIMIT,
            gamble_available: true,
        }
    }

    /// Reset vendor state for a new biome visit.
    pub fn reset_for_visit(&mut self) {
        self.free_food_remaining = TARQUIN_FREE_FOOD_LIMIT;
        self.gamble_available = true;
    }

    /// Claim free food item. Returns true if one was available.
    pub fn claim_free_food(&mut self) -> bool {
        if self.free_food_remaining > 0 {
            self.free_food_remaining -= 1;
            true
        } else {
            false
        }
    }
}

// --- Shop ---

pub struct Shop {
    pub bagira: BagiraStock,
    pub tarquin: TarquinStock,
}

impl Shop {
    pub fn new() -> Self {
        Self {
            bagira: BagiraStock::new(),
            tarquin: TarquinStock::new(),
        }
    }

    /// Called at each biome boundary before the shop screen renders.
    pub fn open_for_biome(&mut self) {
        self.bagira.reroll();
        self.tarquin.reset_for_visit();
    }
}
