//! Smoke trails emitted from missiles.

use std::{collections::VecDeque, f32::consts::TAU};

use crate::{
    distance::fast_distance,
    graphics::{create_smoke_trail, recycle_smoke_trail, render_smoke_trail},
    world::{EntityIds, Map, World},
};

const SMOKE_TRAIL_LEN: usize = 24;

pub struct SmokeTrail {
    parent: u32,
    parent_is_valid: bool,
    // Store parent's x and y so that we can use that info after the missile has
    // disappeared.
    parent_x: f32,
    parent_y: f32,
    max_len: usize,
    // Smoke particles are ordered oldest to newest.
    particles: VecDeque<SmokeParticle>,
    renderers: Vec<SmokeTrailRenderer>,
}

/// A single missile has a unique history of positions/velocities, hence it has
/// a unique smoke trail. But it can have multiple visible smoke trails that
/// share the same data but differ in period/wavelength.
///
/// SmokeTrailRenderer stores these individual differences without duplicating
/// shared SmokeTrail data. It also stores x and y coordinate information for
/// the renderer so that it doesn't get dropped.
pub struct SmokeTrailRenderer {
    // Each renderer has a unique id so that it can be rendered. This id
    // isn't really an entity though, since renderers are stored in smoke
    // trails, not directly in the world.
    id: u32,
    xs: Vec<f32>,
    ys: Vec<f32>,
    period_offset: f32,
    frequency: f32,
}

struct SmokeParticle {
    birth_tick: u32,
    x: f32,
    y: f32,
    normal_x: f32,
    normal_y: f32,
}

pub fn spawn_smoke_trail(
    entity_ids: &mut EntityIds,
    smoke_trails: &mut Map<u32, SmokeTrail>,
    parent: u32,
) {
    let renderers = vec![
        SmokeTrailRenderer {
            id: entity_ids.next(),
            xs: vec![0.0; SMOKE_TRAIL_LEN],
            ys: vec![0.0; SMOKE_TRAIL_LEN],
            period_offset: 0.0,
            frequency: 0.2,
        },
        SmokeTrailRenderer {
            id: entity_ids.next(),
            xs: vec![0.0; SMOKE_TRAIL_LEN],
            ys: vec![0.0; SMOKE_TRAIL_LEN],
            period_offset: TAU / 3.0,
            frequency: 0.2,
        },
        SmokeTrailRenderer {
            id: entity_ids.next(),
            xs: vec![0.0; SMOKE_TRAIL_LEN],
            ys: vec![0.0; SMOKE_TRAIL_LEN],
            period_offset: TAU * 2.0 / 3.0,
            frequency: 0.12,
        },
    ];
    for renderer in &renderers {
        create_smoke_trail(renderer.id, SMOKE_TRAIL_LEN)
    }
    smoke_trails.insert(
        entity_ids.next(),
        SmokeTrail {
            parent,
            parent_is_valid: true,
            // x and y will get updated in update_smoke, no need to init here
            parent_x: 0.0,
            parent_y: 0.0,
            max_len: SMOKE_TRAIL_LEN,
            particles: VecDeque::with_capacity(SMOKE_TRAIL_LEN),
            renderers,
        },
    );
}

impl SmokeTrail {
    pub fn render(&mut self, frame_fudge: f32, tick: u32) {
        if let Some(last) = self.particles.back() {
            for i in 0..self.max_len {
                let particle = self.particles.get(i).unwrap_or(last);
                let age = (tick - particle.birth_tick) as f32 + frame_fudge;
                let mut x = particle.x;
                let mut y = particle.y;
                // Push smoke away from this missile's explosion (doesn't
                // interact with other missile explosions)
                if !self.parent_is_valid {
                    let dx = particle.x - self.parent_x;
                    let dy = particle.y - self.parent_y;
                    let magnitude = fast_distance(dx, dy);
                    if magnitude > 0.0 {
                        let time = (self.max_len - self.particles.len()) as f32 + frame_fudge;
                        x += 0.4 * time * dx / magnitude;
                        y += 0.4 * time * dy / magnitude;
                    }
                }
                for renderer in &mut self.renderers {
                    renderer.xs[i] = x
                        + (3.0 + 0.6 * age)
                            * (renderer.period_offset
                                + particle.birth_tick as f32 * renderer.frequency)
                                .sin()
                            * particle.normal_x;
                    renderer.ys[i] = y
                        + (3.0 + 0.6 * age)
                            * (renderer.period_offset
                                + particle.birth_tick as f32 * renderer.frequency)
                                .sin()
                            * particle.normal_y;
                }
            }
            for renderer in &self.renderers {
                render_smoke_trail(renderer.id, renderer.xs.as_ptr(), renderer.ys.as_ptr());
            }
        }
    }
}

impl World {
    pub fn update_smoke(&mut self) {
        let mut trash = Vec::new();
        for (entity, smoke_trail) in &mut self.smoke_trails {
            let parent_mob = self.mobs.get(&smoke_trail.parent);
            let parent_missile = self.missiles.get(&smoke_trail.parent);
            if let (Some(mob), Some(missile)) = (parent_mob, parent_missile) {
                smoke_trail.particles.push_back(SmokeParticle {
                    birth_tick: self.tick,
                    x: mob.x,
                    y: mob.y,
                    normal_x: -missile.rotation.sin(),
                    normal_y: missile.rotation.cos(),
                });
                smoke_trail.parent_x = mob.x;
                smoke_trail.parent_y = mob.y;
                if smoke_trail.particles.len() > smoke_trail.max_len {
                    smoke_trail.particles.pop_front();
                }
            } else {
                // Instead of letting smoke gradually disperse after the
                // parent missile disappears, delete it all right away
                for renderer in &smoke_trail.renderers {
                    recycle_smoke_trail(renderer.id);
                }
                trash.push(*entity);

                // If we want the smoke to gradually disperse:

                // smoke_trail.parent_is_valid = false;
                // if smoke_trail.particles.pop_front().is_none() {
                //     for renderer in &smoke_trail.renderers {
                //         unsafe { recycle_smoke_trail(renderer.id) }
                //     }
                //     trash.push(*entity);
                // }
            }
        }
        for entity in trash {
            self.smoke_trails.remove(&entity);
        }
    }
}
