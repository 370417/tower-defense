use std::f32::consts::PI;

use serde::{Deserialize, Serialize};

use crate::{graphics::SpriteType, world::World};

#[derive(Serialize, Deserialize)]
pub struct Health {
    pub curr_health: f32,
    pub max_health: f32,
}

#[derive(Default)]
pub struct Corpse {
    age: u32,
    x: f32,
    y: f32,
}

const MAX_CORPSE_AGE: u32 = 60;

impl Health {
    pub fn new(max_health: f32) -> Health {
        Health {
            curr_health: max_health,
            max_health,
        }
    }
}

impl World {
    pub fn handle_dead(&mut self) {
        // Age corpses
        let mut trash = Vec::new();
        for (&entity, corpse) in &mut self.render_state.corpses {
            corpse.age += 1;
            if corpse.age == MAX_CORPSE_AGE {
                trash.push(entity);
            }
        }
        for entity in trash {
            self.render_state.corpses.remove(&entity);
        }

        // Turn 0 health enemies into corpses
        let mut graveyard = Vec::new();
        for (&entity, health) in &self.core_state.health {
            if health.curr_health <= 0.0 {
                graveyard.push(entity);
                if let Some(mob) = self.core_state.mobs.get(&entity) {
                    self.render_state.corpses.insert(
                        entity,
                        Corpse {
                            age: 0,
                            x: mob.x,
                            y: mob.y,
                        },
                    );
                }
            }
        }
        for entity in graveyard {
            self.core_state.health.remove(&entity);
            self.core_state.impulses.remove(&entity);
            self.core_state.mobs.remove(&entity);
            self.core_state.health.remove(&entity);
            self.core_state.target_indicators.remove(&entity);
            self.core_state.threats.remove(&entity);
            self.core_state.walkers.remove(&entity);
        }
    }

    pub fn dump_corpses(&mut self) {
        for (_, corpse) in &self.render_state.corpses {
            let progress = corpse.age as f32 / MAX_CORPSE_AGE as f32;
            let sin_eased_progress = (progress * PI / 2.0).sin();
            self.render_state.sprite_data.push(
                SpriteType::Corpse as u8,
                corpse.x,
                corpse.y,
                0.0,
                1.0 - sin_eased_progress,
                0x000000,
            )
        }
    }
}
