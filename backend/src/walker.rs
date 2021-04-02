use crate::{
    distance::fast_distance,
    graphics::{SpriteData, WALKER_ID},
    map::{true_row_col, Constants, Tile, TRUE_MAP_WIDTH},
    mob::Mob,
    world::{Map, World},
};

pub const STANDARD_ENEMY_RADIUS: f32 = 0.3 * f32::TILE_SIZE;

/// A walker is an entity that travels along the map's path.
pub struct Walker {
    pub speed: f32,
}

impl Walker {
    pub fn dump(&self, id: &u32, data: &mut SpriteData, mobs: &Map<u32, Mob>, frame_fudge: f32) {
        if let Some(mob) = mobs.get(id) {
            data.push(
                WALKER_ID,
                mob.x + frame_fudge * (mob.x - mob.old_x),
                mob.y + frame_fudge * (mob.y - mob.old_y),
                0.0,
                1.0,
                0x777777,
            );
        }
    }
}

impl World {
    pub fn walk(&mut self) {
        for (entity, walker) in &mut self.walkers {
            if let Some(mob) = self.mobs.get_mut(entity) {
                let (true_row, true_col) = true_row_col(mob.x, mob.y);

                let mut speed = walker.speed;

                // Walk slower if under the effects of an external impulse
                if let Some(impulse) = self.impulses.get(entity) {
                    let magnitude = fast_distance(impulse.dx, impulse.dy);
                    speed *= 1.0 / (1.0 + 0.0 * magnitude);
                }

                walk_tile(&self.map, true_row, true_col, &mut mob.x, &mut mob.y, speed);
            }
        }
    }
}

/// Unit vectors pointing in cardinal directions. A representation of direction
/// that can be worked with mathematically.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Velocity {
    pub dx: f32,
    pub dy: f32,
}

/// Given a position on the map, return the direction that a walker should try
/// to travel in. This function returns as if the walker is trying to go
/// forwards (positive speed).
pub fn walk_direction(map: &[Tile], x: f32, y: f32) -> Velocity {
    let (true_row, true_col) = true_row_col(x, y);

    let (entrance_direction, exit_direction) = match map.get(true_row * TRUE_MAP_WIDTH + true_col) {
        Some(Tile::North) => return NORTH,
        Some(Tile::South) => return SOUTH,
        Some(Tile::East) => return EAST,
        Some(Tile::West) => return WEST,
        Some(Tile::NorthToEast) => (NORTH, EAST),
        Some(Tile::NorthToWest) => (NORTH, WEST),
        Some(Tile::SouthToEast) => (SOUTH, EAST),
        Some(Tile::SouthToWest) => (SOUTH, WEST),
        Some(Tile::EastToNorth) => (EAST, NORTH),
        Some(Tile::EastToSouth) => (EAST, SOUTH),
        Some(Tile::WestToNorth) => (WEST, NORTH),
        Some(Tile::WestToSouth) => (WEST, SOUTH),
        _ => return Velocity { dx: 0.0, dy: 0.0 },
    };

    let entrance_x = (true_col as f32 - 1.5 - 0.5 * entrance_direction.dx) * f32::TILE_SIZE;
    let entrance_y = (true_row as f32 - 1.5 - 0.5 * entrance_direction.dy) * f32::TILE_SIZE;

    let exit_x = (true_col as f32 - 1.5 + 0.5 * exit_direction.dx) * f32::TILE_SIZE;
    let exit_y = (true_row as f32 - 1.5 + 0.5 * exit_direction.dy) * f32::TILE_SIZE;

    let dist_from_entrance = (x - entrance_x).abs().max((y - entrance_y).abs());
    let dist_from_exit = (x - exit_x).abs().max((y - exit_y).abs());

    if dist_from_entrance < dist_from_exit {
        entrance_direction
    } else {
        exit_direction
    }
}

pub fn walk_tile(
    map: &[Tile],
    mut true_row: usize,
    mut true_col: usize,
    x: &mut f32,
    y: &mut f32,
    mut speed: f32,
) {
    let mut direction = walk_direction(map, *x, *y);

    while speed.abs() > 0.005 {
        let tile_x = (true_col as f32 - 2.0) * f32::TILE_SIZE;
        let tile_y = (true_row as f32 - 2.0) * f32::TILE_SIZE;

        let center_x = tile_x + f32::TILE_SIZE / 2.0;
        let center_y = tile_y + f32::TILE_SIZE / 2.0;

        let tile = match map.get(true_row * TRUE_MAP_WIDTH + true_col) {
            Some(&tile) => tile,
            None => break,
        };
        let (new_x, new_y) = match tile {
            Tile::OutOfBounds | Tile::Empty => return,
            Tile::North | Tile::South | Tile::East | Tile::West => {
                // Walk straight

                (*x + speed * direction.dx, *y + speed * direction.dy)
            }
            _ => {
                // Walk around the corner

                let new_x = *x + speed * direction.dx;
                let new_y = *y + speed * direction.dy;

                let entrance_direction = match tile {
                    Tile::NorthToEast | Tile::NorthToWest => NORTH,
                    Tile::SouthToEast | Tile::SouthToWest => SOUTH,
                    Tile::EastToNorth | Tile::EastToSouth => EAST,
                    Tile::WestToNorth | Tile::WestToSouth => WEST,
                    _ => unreachable!(),
                };

                let exit_direction = match tile {
                    Tile::NorthToEast | Tile::SouthToEast => EAST,
                    Tile::NorthToWest | Tile::SouthToWest => WEST,
                    Tile::EastToNorth | Tile::WestToNorth => NORTH,
                    Tile::EastToSouth | Tile::WestToSouth => SOUTH,
                    _ => unreachable!(),
                };

                // Direction from inner corner to outer corner (or vice versa)
                let corner_direction = Velocity {
                    dx: entrance_direction.dy - exit_direction.dy,
                    dy: entrance_direction.dx - exit_direction.dx,
                };

                // Find the intersection between the mob's movement vector and
                // the corner vector extending from the center of the tile.

                // [x,y] + a * [direction] === [center] + b * [-corner direction]
                // a * direction.dx + b * corner_direction.dx === center_x - x
                // a * direction.dy + b * corner_direction.dy === center_y - y

                // determinant will never be 0
                let determinant =
                    direction.dx * corner_direction.dy - corner_direction.dx * direction.dy;
                let term_a =
                    corner_direction.dy * (center_x - *x) - corner_direction.dx * (center_y - *y);
                // let term_b = direction.dx * (center_y - *y) - direction.dy * (center_x - *x);
                let a = term_a / determinant;
                // let b = term_b / determinant;

                if a * speed.signum() <= 0.0 {
                    // We have already moved past the turn
                    direction = if speed > 0.0 {
                        exit_direction
                    } else {
                        entrance_direction
                    };
                    (new_x, new_y)
                } else if speed.abs() < a.abs() {
                    // We won't reach the turn yet
                    direction = if speed > 0.0 {
                        entrance_direction
                    } else {
                        exit_direction
                    };
                    (new_x, new_y)
                } else if a.abs() == speed.abs() {
                    // We've reached the turn exactly
                    direction = if speed > 0.0 {
                        exit_direction
                    } else {
                        entrance_direction
                    };
                    (new_x, new_y)
                } else {
                    // We have crossed the turning point
                    speed -= a;
                    let turn_x = *x + a * direction.dx;
                    let turn_y = *y + a * direction.dy;
                    direction = if speed > 0.0 {
                        exit_direction
                    } else {
                        entrance_direction
                    };
                    (turn_x + speed * direction.dx, turn_y + speed * direction.dy)
                }
            }
        };

        let clamped_x = new_x.max(tile_x).min(tile_x + f32::TILE_SIZE);
        let clamped_y = new_y.max(tile_y).min(tile_y + f32::TILE_SIZE);

        speed = speed.signum() * ((new_x - clamped_x).abs() + (new_y - clamped_y).abs());

        *x = clamped_x;
        *y = clamped_y;

        // If speed got reduced to 0, we don't actually move to a new
        // tile, but since a speed of 0 exits the while loop, there's
        // no need for an if check here.

        true_row = (true_row as isize + (speed.signum() * direction.dy) as isize) as usize;
        true_col = (true_col as isize + (speed.signum() * direction.dx) as isize) as usize;
    }
}

pub const NORTH: Velocity = Velocity { dx: 0.0, dy: -1.0 };
pub const SOUTH: Velocity = Velocity { dx: 0.0, dy: 1.0 };
pub const EAST: Velocity = Velocity { dx: 1.0, dy: 0.0 };
pub const WEST: Velocity = Velocity { dx: -1.0, dy: 0.0 };

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::{parse, MAP_0};

    fn walk(map: &[Tile], x: &mut f32, y: &mut f32, speed: f32) {
        let true_row = (*y as usize + 2 * usize::TILE_SIZE) / usize::TILE_SIZE;
        let true_col = (*x as usize + 2 * usize::TILE_SIZE) / usize::TILE_SIZE;

        // Use an inner walk_tile function to disambiguate when on
        // an edge between two tiles.
        walk_tile(map, true_row, true_col, x, y, speed);
    }

    #[test]
    fn walk_around_corner() {
        let map = parse(&MAP_0);

        // middle of the road test
        let tile_offset_x = 13.0 * f32::TILE_SIZE;
        let tile_offset_y = 13.0 * f32::TILE_SIZE;
        let mut x = tile_offset_x + 16.0;
        let mut y = tile_offset_y + 24.0;
        walk(&map, &mut x, &mut y, 20.0);
        assert_eq!(x, tile_offset_x + 28.0);
        assert_eq!(y, tile_offset_y + 16.0);

        // off center test
        let mut x = tile_offset_x + 12.0;
        let mut y = tile_offset_y + 24.0;
        walk(&map, &mut x, &mut y, 20.0);
        assert_eq!(x, tile_offset_x + 20.0);
        assert_eq!(y, tile_offset_y + 12.0);

        // different tile, off center, negative speed
        let tile_offset_x = 2.0 * f32::TILE_SIZE;
        let tile_offset_y = 1.0 * f32::TILE_SIZE;
        let mut x = tile_offset_x + 30.0;
        let mut y = tile_offset_y + 14.0;
        walk(&map, &mut x, &mut y, -30.0);
        assert_eq!(x, tile_offset_x + 12.0);
        assert_eq!(y, tile_offset_y + 2.0);

        // same as before but positive speed, going the other way
        let tile_offset_x = 2.0 * f32::TILE_SIZE;
        let tile_offset_y = 1.0 * f32::TILE_SIZE;
        let mut x = tile_offset_x + 12.0;
        let mut y = tile_offset_y + 2.0;
        walk(&map, &mut x, &mut y, 30.0);
        assert_eq!(x, tile_offset_x + 30.0);
        assert_eq!(y, tile_offset_y + 14.0);

        // entire track along the center line
        let tile_offset_x = -1.0 * f32::TILE_SIZE;
        let tile_offset_y = 2.0 * f32::TILE_SIZE;
        let mut x = tile_offset_x + 0.0;
        let mut y = tile_offset_y + 0.0;
        walk(&map, &mut x, &mut y, 102.0 * 32.0);
        assert_eq!(x, f32::TILE_SIZE * 23.0);
        assert_eq!(y, f32::TILE_SIZE * 2.0);

        // second corner test
        let tile_offset_x = 2.0 * f32::TILE_SIZE;
        let tile_offset_y = 3.0 * f32::TILE_SIZE;
        let mut x = tile_offset_x + 16.0;
        let mut y = tile_offset_y + 15.5;
        walk(&map, &mut x, &mut y, 1.0);
        assert_eq!(x, tile_offset_x + 16.5);
        assert_eq!(y, tile_offset_y + 16.0);

        // second corner test on diagonal
        let tile_offset_x = 2.0 * f32::TILE_SIZE;
        let tile_offset_y = 3.0 * f32::TILE_SIZE;
        let mut x = tile_offset_x + 16.0;
        let mut y = tile_offset_y + 16.0;
        walk(&map, &mut x, &mut y, 1.0);
        assert_eq!(x, tile_offset_x + 17.0);
        assert_eq!(y, tile_offset_y + 16.0);

        let mut x = 80.5;
        let mut y = 48.5;
        walk(&map, &mut x, &mut y, 1.0);
    }
}
