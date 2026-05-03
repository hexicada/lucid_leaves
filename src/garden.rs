#![allow(dead_code)]

pub const GROWTH_STEP_SECS: i64 = 86_400; // 1 real-world day in seconds
pub const GARDEN_PLOT_COUNT: usize = 9;   // 3x3 initial grid
pub const TIME_SKIP_GRACE_SECS: i64 = 600; // 10 minutes — anti time-skip threshold
pub const FERTILIZER_STEP_MULTIPLIER: f32 = 0.5; // next growth step is 50% duration

// --- Plant Types ---

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PlantType {
    DayBloom,   // grown from SeedDay  (Sun tile drop)
    NightBloom, // grown from SeedNight (Moon tile drop)
    // Expand with new varieties as evergreen content sprints ship
}

// --- Plant Stages ---

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PlantStage {
    Empty,
    Seeded,
    Sprouting,
    Grown,
    Blooming,
    Rare, // only reachable via MoonbloomEssence or boss drop
}

impl PlantStage {
    /// Returns the next stage in the growth chain, or None at terminal stages.
    pub fn next(self) -> Option<Self> {
        match self {
            PlantStage::Empty     => None,
            PlantStage::Seeded    => Some(PlantStage::Sprouting),
            PlantStage::Sprouting => Some(PlantStage::Grown),
            PlantStage::Grown     => Some(PlantStage::Blooming),
            PlantStage::Blooming  => None,
            PlantStage::Rare      => None,
        }
    }

    pub fn is_terminal(self) -> bool {
        matches!(self, PlantStage::Empty | PlantStage::Rare)
    }
}

// --- Garden Plot ---

#[derive(Clone, Copy, Debug)]
pub struct GardenPlot {
    pub plant_type: Option<PlantType>,
    pub stage: PlantStage,
    pub watered: bool,
    pub fertilized: bool,
    pub moonbloom_infused: bool,
    pub planted_at_unix: i64,      // UTC unix seconds
    pub next_stage_at_unix: i64,   // UTC unix seconds — when this stage completes
}

impl GardenPlot {
    pub const EMPTY: Self = Self {
        plant_type: None,
        stage: PlantStage::Empty,
        watered: false,
        fertilized: false,
        moonbloom_infused: false,
        planted_at_unix: 0,
        next_stage_at_unix: 0,
    };

    pub fn plant(&mut self, plant_type: PlantType, now_unix: i64) -> bool {
        if self.stage != PlantStage::Empty {
            return false;
        }
        self.plant_type = Some(plant_type);
        self.stage = PlantStage::Seeded;
        self.watered = false;
        self.fertilized = false;
        self.moonbloom_infused = false;
        self.planted_at_unix = now_unix;
        self.next_stage_at_unix = now_unix + GROWTH_STEP_SECS;
        true
    }

    pub fn water(&mut self) -> bool {
        if matches!(self.stage, PlantStage::Empty | PlantStage::Rare) {
            return false;
        }
        self.watered = true;
        true
    }

    pub fn fertilize(&mut self) -> bool {
        if matches!(self.stage, PlantStage::Empty | PlantStage::Rare) {
            return false;
        }
        self.fertilized = true;
        true
    }

    pub fn infuse_moonbloom(&mut self) -> bool {
        if self.stage != PlantStage::Grown {
            return false;
        }
        self.moonbloom_infused = true;
        true
    }

    /// Advance stage if the timer has elapsed and conditions are met.
    /// `now_unix` must be UTC unix seconds from system clock.
    /// Returns true if the plot advanced.
    pub fn tick(&mut self, now_unix: i64) -> bool {
        if self.stage.is_terminal() {
            return false;
        }
        if !self.watered {
            return false;
        }
        if now_unix >= self.next_stage_at_unix {
            let next = if self.stage == PlantStage::Grown && self.moonbloom_infused {
                Some(PlantStage::Rare)
            } else {
                self.stage.next()
            };

            if let Some(next) = next {
                self.stage = next;
                self.watered = false;
                let step_secs = if self.fertilized {
                    ((GROWTH_STEP_SECS as f32) * FERTILIZER_STEP_MULTIPLIER).max(1.0) as i64
                } else {
                    GROWTH_STEP_SECS
                };
                self.fertilized = false;
                self.next_stage_at_unix = now_unix + step_secs;
                return true;
            }
        }
        false
    }
}

// --- Garden ---

pub struct Garden {
    pub plots: Vec<GardenPlot>,
    pub last_seen_unix: i64, // used for anti-time-skip detection
}

impl Garden {
    pub fn new() -> Self {
        Self {
            plots: vec![GardenPlot::EMPTY; GARDEN_PLOT_COUNT],
            last_seen_unix: 0,
        }
    }

    /// Call every time the player enters the Garden screen.
    /// Catches up offline growth; freezes if clock moved backward.
    pub fn tick_all(&mut self, now_unix: i64) {
        // Anti-time-skip: if clock went backward beyond grace window, do nothing
        if self.last_seen_unix > 0 && now_unix < self.last_seen_unix - TIME_SKIP_GRACE_SECS {
            return;
        }
        for plot in self.plots.iter_mut() {
            plot.tick(now_unix);
        }
        self.last_seen_unix = now_unix;
    }
}
