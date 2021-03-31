use float_ord::FloatOrd;

use crate::{
    map::{
        distances::{calc_dist_from_exit, Distances},
        Tile,
    },
    mob::Mob,
    tower::Range,
    walker::Walker,
    world::Map,
};

pub enum Targeting {
    First,
    Close,
}

pub fn find_target(
    tower_x: f32,
    tower_y: f32,
    range: Range,
    strategy: Targeting,
    walkers: &Map<u32, Walker>,
    mobs: &Map<u32, Mob>,
    map: &[Tile],
    _dist_from_entrance: &Distances,
    dist_from_exit: &Distances,
) -> Option<(u32, f32, f32)> {
    match range {
        Range::Circle { radius } => {
            let dist_from_tower_squared = |mob: &Mob| {
                let dx = mob.x - tower_x;
                let dy = mob.y - tower_y;
                dx * dx + dy * dy
            };

            let enemies_in_range = walkers
                .keys()
                .filter_map(|entity| mobs.get(entity).and_then(|mob| Some((mob, entity))))
                .filter(|(mob, _)| dist_from_tower_squared(mob) < radius * radius);

            match strategy {
                Targeting::First => {
                    let target = enemies_in_range.min_by_key(|(mob, _)| {
                        FloatOrd(calc_dist_from_exit(map, dist_from_exit, mob.x, mob.y))
                    });
                    target.and_then(|(mob, &entity)| Some((entity, mob.x, mob.y)))
                }
                Targeting::Close => {
                    let target = enemies_in_range
                        .min_by_key(|(mob, _)| FloatOrd(dist_from_tower_squared(mob)));
                    target.and_then(|(mob, entity)| Some((*entity, mob.x, mob.y)))
                }
            }
        }
    }
}
