use macroquad::prelude::*;
use macroquad::rand::gen_range;

use crate::tile::{Tile, TileType};
use crate::ui_layout::Layout;

pub type MatchCell = (usize, usize, TileType);

#[derive(Clone, Copy)]
pub struct GemParticle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub life: f32,
    pub max_life: f32,
    pub size: f32,
    pub color: Color,
}

pub fn tile_particle_color(kind: TileType) -> Color {
    match kind {
        TileType::Sun => color_u8!(255, 215, 110, 255),
        TileType::Moon => color_u8!(180, 205, 255, 255),
        TileType::Water => color_u8!(90, 170, 255, 255),
        TileType::Leaf => color_u8!(150, 235, 150, 255),
        TileType::Exotic => color_u8!(230, 140, 255, 255),
        TileType::Empty => WHITE,
    }
}

pub fn pending_match_kind_at(pending_matches: &[MatchCell], x: usize, y: usize) -> Option<TileType> {
    pending_matches
        .iter()
        .find(|(mx, my, _)| *mx == x && *my == y)
        .map(|(_, _, kind)| *kind)
}

pub fn find_matches<const W: usize, const H: usize>(grid: &[[Tile; H]; W]) -> Vec<MatchCell> {
    let mut to_remove = vec![];

    for y in 0..H {
        for x in 0..W.saturating_sub(2) {
            let t1 = grid[x][y].kind;
            let t2 = grid[x + 1][y].kind;
            let t3 = grid[x + 2][y].kind;
            if t1 != TileType::Empty && t1 == t2 && t2 == t3 {
                to_remove.push((x, y, t1));
                to_remove.push((x + 1, y, t2));
                to_remove.push((x + 2, y, t3));
            }
        }
    }

    for x in 0..W {
        for y in 0..H.saturating_sub(2) {
            let t1 = grid[x][y].kind;
            let t2 = grid[x][y + 1].kind;
            let t3 = grid[x][y + 2].kind;
            if t1 != TileType::Empty && t1 == t2 && t2 == t3 {
                to_remove.push((x, y, t1));
                to_remove.push((x, y + 1, t2));
                to_remove.push((x, y + 2, t3));
            }
        }
    }

    to_remove.sort_by_key(|(x, y, _)| (*x, *y));
    to_remove.dedup_by_key(|(x, y, _)| (*x, *y));
    to_remove
}

pub fn spawn_match_particles<const W: usize, const H: usize>(
    grid: &[[Tile; H]; W],
    particles: &mut Vec<GemParticle>,
    matches: &[MatchCell],
    layout: &Layout,
) {
    for (x, y, kind) in matches {
        let center_x = layout.grid_offset_x + *x as f32 * layout.tile_size + layout.tile_size * 0.5;
        let center_y =
            layout.grid_offset_y + *y as f32 * layout.tile_size + grid[*x][*y].offset_y + layout.tile_size * 0.5;
        let color = tile_particle_color(*kind);
        for _ in 0..8 {
            let life = gen_range(0.18, 0.35);
            particles.push(GemParticle {
                x: center_x + gen_range(-8.0, 8.0),
                y: center_y + gen_range(-8.0, 8.0),
                vx: gen_range(-18.0, 18.0),
                vy: gen_range(-34.0, -10.0),
                life,
                max_life: life,
                size: gen_range(3.0, 7.0),
                color,
            });
        }
    }
}

pub fn update_match_effects(particles: &mut Vec<GemParticle>, cascade_pulse: &mut f32, delta: f32) {
    for particle in particles.iter_mut() {
        particle.life -= delta;
        particle.x += particle.vx * delta;
        particle.y += particle.vy * delta;
        particle.vx *= 0.98;
        particle.vy -= 4.0 * delta;
    }
    particles.retain(|particle| particle.life > 0.0);
    *cascade_pulse = (*cascade_pulse - delta * 4.0).max(0.0);
}