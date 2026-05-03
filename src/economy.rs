use macroquad::rand::gen_range;

use crate::inventory::{Inventory, ItemType};
use crate::shop::Shop;
use crate::tile::TileType;

pub const DROP_RATE_LEAF_LEAVES: f32 = 0.30;
pub const DROP_RATE_WATERING_CAN: f32 = 0.30;
pub const DROP_RATE_SEED_DAY: f32 = 0.20;
pub const DROP_RATE_MOON_ITEM: f32 = 0.10;
pub const DROP_RATE_FERTILIZER: f32 = 0.15;
pub const LEAF_DROP_BONUS: i32 = 30;

pub const PRICE_WATERING_CAN: i32 = 220;
pub const PRICE_SEED_DAY: i32 = 260;
pub const PRICE_SEED_NIGHT: i32 = 320;
pub const PRICE_MOONBLOOM_ESSENCE: i32 = 900;
pub const PRICE_FERTILIZER: i32 = 380;

pub fn base_price_for_item(item: ItemType) -> i32 {
    match item {
        ItemType::WateringCan => PRICE_WATERING_CAN,
        ItemType::SeedDay => PRICE_SEED_DAY,
        ItemType::SeedNight => PRICE_SEED_NIGHT,
        ItemType::MoonbloomEssence => PRICE_MOONBLOOM_ESSENCE,
        ItemType::Fertilizer => PRICE_FERTILIZER,
        // Placeholder pricing for non-resource items until full shop generation is wired.
        ItemType::BoardModifier(_) => 500,
        ItemType::FoodBuff(_) => 200,
    }
}

pub fn add_resource_or_consign(
    inventory: &mut Inventory,
    shop: &mut Shop,
    total_points: &mut i32,
    item: ItemType,
) {
    if !inventory.push(item) {
        let paid = shop.bagira.consign(item, base_price_for_item(item));
        *total_points += paid;
    }
}

pub fn roll_resource_drop(
    tile_kind: TileType,
    inventory: &mut Inventory,
    shop: &mut Shop,
    total_points: &mut i32,
) {
    let roll = gen_range(0.0, 1.0);
    match tile_kind {
        TileType::Leaf => {
            if roll < DROP_RATE_LEAF_LEAVES {
                *total_points += LEAF_DROP_BONUS;
            }
        }
        TileType::Water => {
            if roll < DROP_RATE_WATERING_CAN {
                add_resource_or_consign(inventory, shop, total_points, ItemType::WateringCan);
            }
        }
        TileType::Sun => {
            if roll < DROP_RATE_SEED_DAY {
                add_resource_or_consign(inventory, shop, total_points, ItemType::SeedDay);
            }
        }
        TileType::Moon => {
            if roll < DROP_RATE_MOON_ITEM {
                let moon_item = if gen_range(0.0, 1.0) < 0.30 {
                    ItemType::MoonbloomEssence
                } else {
                    ItemType::SeedNight
                };
                add_resource_or_consign(inventory, shop, total_points, moon_item);
            }
        }
        TileType::Exotic => {
            if roll < DROP_RATE_FERTILIZER {
                add_resource_or_consign(inventory, shop, total_points, ItemType::Fertilizer);
            }
        }
        TileType::Empty => {}
    }
}

pub fn inventory_count(inventory: &Inventory, target: ItemType) -> u32 {
    inventory
        .slots
        .iter()
        .filter_map(|slot| {
            if slot.item == Some(target) {
                Some(slot.count)
            } else {
                None
            }
        })
        .sum()
}

pub fn inventory_used_slots(inventory: &Inventory) -> usize {
    inventory
        .slots
        .iter()
        .filter(|slot| slot.item.is_some())
        .count()
}