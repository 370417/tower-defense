use float_ord::FloatOrd;

use crate::{
    walker::Walker,
    world::{Map, World},
};

/// A movalbe object.
#[derive(Clone)]
pub struct Mob {
    pub x: f32,
    pub y: f32,
    // Store the x and y from the last frame so that we can interpolate
    pub old_x: f32,
    pub old_y: f32,
}

impl Mob {
    pub fn new(x: f32, y: f32) -> Mob {
        Mob {
            x,
            y,
            old_x: x,
            old_y: y,
        }
    }
}

impl World {
    pub fn remember_mob_positions(&mut self) {
        for mob in self.mobs.values_mut() {
            mob.old_x = mob.x;
            mob.old_y = mob.y;
        }
    }
}

pub fn closest_walker<'a>(
    walkers: &'a Map<u32, Walker>,
    mobs: &'a Map<u32, Mob>,
    x: f32,
    y: f32,
) -> Option<(&'a u32, &'a Mob, f32)> {
    // This is one of the places where we would use the spatial index
    // when numbers get large
    walkers
        .iter()
        .filter_map(|(entity, _)| mobs.get(entity).and_then(|mob| Some((entity, mob))))
        .map(|(entity, mob)| {
            let dx = mob.x - x;
            let dy = mob.y - y;
            (entity, mob, dx * dx + dy * dy)
        })
        .min_by_key(|(_, _, distance_squared)| FloatOrd(*distance_squared))
}
