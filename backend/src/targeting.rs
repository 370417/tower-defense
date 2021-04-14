use float_ord::FloatOrd;
use serde::{Deserialize, Serialize};

use crate::{
    map::distances::calc_dist_from_exit,
    mob::Mob,
    walker::Walker,
    world::{LevelState, Map},
};

pub enum Targeting {
    First,
    Close,
}

// TODO: take level state instead of map + dist.. + dist...?
pub fn find_target(
    tower_x: f32,
    tower_y: f32,
    range: f32,
    strategy: Targeting,
    walkers: &Map<u32, Walker>,
    mobs: &Map<u32, Mob>,
    level_state: &LevelState,
) -> Option<(u32, f32, f32)> {
    let dist_from_tower_squared = |mob: &Mob| {
        let dx = mob.x - tower_x;
        let dy = mob.y - tower_y;
        dx * dx + dy * dy
    };

    let enemies_in_range = walkers
        .keys()
        .filter_map(|entity| mobs.get(entity).and_then(|mob| Some((mob, entity))))
        .filter(|(mob, _)| dist_from_tower_squared(mob) < range * range);

    match strategy {
        Targeting::First => {
            let target = enemies_in_range.min_by_key(|(mob, _)| {
                FloatOrd(calc_dist_from_exit(
                    &level_state.map,
                    &level_state.dist_from_exit,
                    mob.x,
                    mob.y,
                ))
            });
            target.and_then(|(mob, &entity)| Some((entity, mob.x, mob.y)))
        }
        Targeting::Close => {
            let target =
                enemies_in_range.min_by_key(|(mob, _)| FloatOrd(dist_from_tower_squared(mob)));
            target.and_then(|(mob, entity)| Some((*entity, mob.x, mob.y)))
        }
    }
}

/// Towers that target mobs can alert the mobs by adding threat components to them.
#[derive(Serialize, Deserialize)]
pub struct Threat {}

pub const THREAT_DISTANCE: f32 = 50.0;
