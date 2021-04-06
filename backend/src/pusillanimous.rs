use crate::world::World;

pub struct Pusillanimous {
    pub duration: u32,
}

const SPEEDY_DURATION: u32 = 100;
const COOLDOWN: u32 = 300;

impl World {
    pub fn update_pusillanimity(&mut self) {
        for (entity, pusillanimous) in &mut self.pusillanimous {
            match pusillanimous.duration {
                0 => {
                    if self.threats.contains_key(entity) {
                        pusillanimous.duration = SPEEDY_DURATION + COOLDOWN - 1;
                        if let Some(walker) = self.walkers.get_mut(entity) {
                            walker.speed *= 2.5;
                        }
                    }
                }
                1 => {
                    pusillanimous.duration = 0;
                    self.threats.remove(entity);
                }
                COOLDOWN => {
                    pusillanimous.duration -= 1;
                    if let Some(walker) = self.walkers.get_mut(entity) {
                        walker.speed /= 2.5;
                    }
                }
                _ => {
                    pusillanimous.duration -= 1;
                }
            }
        }
    }
}
