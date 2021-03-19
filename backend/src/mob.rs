use crate::world::World;

/// A movalbe object.
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
