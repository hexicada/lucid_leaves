# lucid_leaves — Agent Implementation Brief

## Project Overview

**Engine:** macroquad (Rust)  
**Repo:** github.com/hexicada/lucid_leaves  
**Player character:** Cade, a void wizard cat  

This brief describes the planned expansion of lucid_leaves from a match-3 puzzle into a three-phase game loop. Do not refactor the existing match-3 core unless explicitly asked. Build new systems additively.

---

## The Three-Phase Game Loop

Each play session cycles through three distinct modes:

```
[BOARD] → earn leaves & resources
   ↓
[GARDEN]
Garden is a zone with 2 mini games:

[TEND] → spend resources, tend plants, watch progress
   ↓
[HUNT] → mini-game, catch wizard mice for rare rewards
   ↑_____________________________|
```

---

## Phase 1: The Board (existing — extend only)

The match-3 core is already implemented. The following additions are needed:

### Resource Drop System

When tiles are matched and cleared, they have a chance to drop resources into the player's inventory in addition to awarding leaves:

| Tile Type | Resource Drop | Drop Rate (suggested) |
|-----------|--------------|----------------------|
| Leaf      | leaves (currency)         | ~30% per match       |
| Water     | Watering Can | ~30% per match       |
| Sun       | Seed (day blooming variety) | ~20% per match     |
| Moon      | Seed (night blooming) or Moonbloom Essence | ~10% per match  |
| Exotic    | Fertilizer   | ~15% per match       |

**Implementation notes:**
- Add an `Inventory` struct to `game_state.rs` with 8 slots and stackable items
- Each slot can hold up to 10 units of a single item type
- If all slots are full and no matching stack has room, item overflows into Bagira consignment instead of being lost
- On overflow/discard, Bagira auto-buys the item for leaves and stores 1 buy-back copy in shop reserve
- On match resolution, after scoring, roll per-tile drops and push to inventory
- Display inventory counts in the HUD alongside the leaves wallet
- Resources persist across levels and biomes (do not reset like illicit move cost)

### UI Additions (Playing state)

- Add "Visit Garden" button — visible at all times during Playing state
- Navigates to Garden phase without ending the level (level state is preserved on return)
- Inventory counts shown as small icons + numbers in a row (seeds, cans, tokens, etc.)
- Initial inventory is 8 slots, max stack 10 per slot
- Include a discard item function so the player can decide what to keep in inventory
- Discard UI copy: "You can't carry that. I'll take it off your hands." (Bagira)

---

## Phase 2: The Garden

A separate screen/room the player visits to spend resources and tend plants.

### Garden Data Model

```rust
pub enum PlantStage {
    Empty,
    Seeded,
    Sprouting,
    Grown,
    Blooming,
    Rare, // only reachable via Moonbloom Essence or boss drops
}

pub struct GardenPlot {
    pub plant_type: Option<PlantType>,
    pub stage: PlantStage,
    pub watered: bool,
    pub fertilized: bool,
    pub planted_at_unix: i64,
    pub next_stage_at_unix: i64, // each growth step takes 1 real-world day
}

pub struct Garden {
    pub plots: Vec<GardenPlot>, // 3x3 grid to start, expandable
}
```

### Garden Actions

| Action | Cost | Effect |
|--------|------|--------|
| Plant seed | 1 Seed | Moves plot from Empty → Seeded |
| Water | 1 Watering Can | Required to advance from Seeded → Sprouting |
| Fertilize | 1 Fertilizer | Speeds up or skips a growth stage |
| Moonbloom | 1 Moonbloom Essence | Unlocks Rare stage on any Grown plant |

### Real-Time Progression Rules

- Garden clock follows real local time (if it is 6 PM IRL, it is 6 PM in the garden)
- Any planting or stage upgrade takes 1 real-world day to complete
- On entering Garden, compare current local time against `next_stage_at_unix` and advance eligible plots
- Growth progression should be deterministic and save/load safe using unix timestamps
- Persist timestamps in UTC unix seconds; render clock/time labels in local time for player readability
- Anti-time-skip policy (v1): if system clock moves backward by more than 10 minutes, freeze growth until real time catches up

### Visual Direction

- Isometric tile-based grid (to differentiate strongly from game board)
- Each plot is a cell showing the current plant stage sprite
- player sprite is not shown 
- Peaceful, no time pressure — this is the rest phase
- Background distinct from board (soft greens, soil textures)

### Navigation

- "Return to Puzzle" button takes player back to the board (level state restored)
- "Go Hunt" button is available here to launch the Hunt mini-game

---

## Phase 3: The Hunt (Wizard Mouse Mini-Game)

The player (Cade) chases and battles wizard mice between puzzle sessions. This is a short burst of chaos - high energy contrast to the peaceful garden.

### Structure

**Regular Hunt** (available after any level completion)  
- Scrub mice scurry across the screen in patterns  
- Cade clicks to cast spells and catch them  
- Rewards: small amounts of rare resources (Moonbloom Essence, Fertilizer)  
- Duration: ~60 seconds  

**Biome Boss Wizard mouse Hunt** (triggered at end of every 3rd level / biome boundary)  
- happens after the shop where you can buy things to help you take down the Big Cheese Boss Mouse  
- A named Wizard Mouse Boss with themed magic based on current biome  
- Boss has a health bar and attack phases  
- Defeating the boss drops rare garden rewards not obtainable elsewhere  
- Failing does not block progression but costs leaves  

### Wizard Mouse Boss Themes (per biome)

| Biome | Boss Name (placeholder) | Gimmick |
|-------|------------------------|---------|
| Forest | Moswick the Green | Vines block parts of the board on next level |
| Desert | Wilfred | Fire tiles appear on board, spread if unmatched |
| Ocean | Tidewhistle | Frozen gem tiles, must match to unfreeze |
| Shadow | The Veilmouse | Fog of war — part of board hidden each turn |

> Boss gimmicks apply as a debuff to the **next** board session, not during the Hunt itself. This ties the Hunt outcome meaningfully to the puzzle experience.

### Hunt Controls

- Mouse-first controls for current PC build:
    - Left click to cast basic spell (targets nearest mouse)
    - Hold left click to charge a wide AOE spell (costs Exotic gem / Fertilizer)
    - Right click or WASD to dodge incoming mouse magic
- Mobile touch controls are a later adaptation, not current scope

### Hunt Data Model

```rust
pub struct WizardMouse {
    pub name: String,
    pub hp: f32,
    pub max_hp: f32,
    pub is_boss: bool,
    pub biome_theme: BiomeType,
    pub reward_pool: Vec<ResourceDrop>,
}
```

---

## Shop (Biome Boundary)

The shop appears at biome boundaries and has two vendors:

1. **Tarquin** (capricious human witch)
    - Free food items (small buffs)
    - Gamble option for random unidentified items
    - Free food limit: 1 item per biome visit
    - Gamble tiers: 60% common, 30% uncommon, 10% rare board modifier
2. **Bagira** (main merchant)
    - Sells known board-modifying items with visible effects
    - Shop stock: 4 visible items per visit, rerolled each biome transition
    - Price bands (leaves): common 200-400, uncommon 500-900, rare 1200+
    - Handles overflow/discard buyback service
    - Buyback reserve keeps the last 12 overflow/discard items (FIFO)
    - Buyback pricing: Bagira pays 35% of base value on sell, resells at 100% base value

Board-modifying items should plug into the match-3 condition system (tile odds, modifiers, debuff counters, etc.) without refactoring the core board loop.

---

## Game State Machine (updated)

```
Playing
  ├─ [Visit Garden] ──→ Garden
  │                        └─ [Go Hunt] ──→ Hunt ──→ Garden
  │                        └─ [Return to Puzzle] ──→ Playing
  ├─ [Level complete] ──→ LevelTransition
  │                        ├─ [Enter] ──→ Playing (next level)
  │                        ├─ [Farm]  ──→ Playing (same level, farming mode)
  │                        └─ [every 3rd level] ──→ BossHunt ──→ Shop ──→ Playing
  └─ [Keep Farming] ──→ Playing (same level)
```

Add `Garden` and `Hunt` as new variants to the `GamePhase` enum in `game_state.rs`.

---

## Narrative Context (for flavour text / UI copy)

The wizard mice have hexed the land. Cade tends the garden to restore life to the world, earns leaves by solving the cursed match-3 hex puzzles the mice left behind, and hunts wizard mice to defeat their coven one champion at a time.

Long-term progression is an evergreen power-creep loop:
- Add new flower varieties over time
- Add crafting recipes that consume flowers/resources
- Crafted items can modify board conditions in match-3 runs
- "Complete the current garden" is a milestone, not a hard end-state

Tone: cozy, whimsical, Studio Ghibli-adjacent. Never dark. Wizard mice are antagonists but they're also tiny and funny.

---

## Art Assets Needed (future sprints)

- Garden plot states per plant type (Empty → Rare) — CSP export → pack script pipeline
- Wizard mouse sprites (at least 1 scrub mouse, 4 biome bosses)
- Cade casting/walking animations for Garden and Hunt screens
- Garden background (top-down soil/grass tiles)
- Hunt background per biome
- Resource icons (seed, watering can, sunlight token, moonbloom essence, fertilizer)

---

## Implementation Order (suggested)

1. Add `Inventory` struct and resource drops to board match resolution
2. Add inventory HUD display to Playing state
3. Add `GamePhase::Garden` state and placeholder Garden screen with navigation buttons
4. Implement `GardenPlot` data model and save/load
5. Implement Garden actions (plant, water, fertilizer) with basic placeholder art
6. Add `GamePhase::Hunt` state and basic scrub mouse Hunt mini-game
7. Implement Wizard Mouse Boss Hunt at biome boundaries
8. Add boss debuff system linking Hunt outcome to next board session
9. Polish art, audio, and mobile touch input

---

*This brief was written as a design handoff. Ask the player (hexicada) before making structural changes to existing match-3 logic. When in doubt, build new, don't refactor old.*
