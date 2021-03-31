//! Calculate and store distances to exits and entrances along a map's path.
//! These distances are measured like a track and field oval in that walkers
//! move along lanes to reach their goal. This means we need to keep track of
//! how many turns walkers have been through.

use crate::walker::{walk_direction, Velocity, EAST, NORTH, SOUTH, WEST};

use super::{true_row_col, Constants, Tile, TRUE_MAP_HEIGHT, TRUE_MAP_WIDTH};

/// Return the tile that a path tile points toward (if it is not a corner tile).
/// If the tile is not a path tile, return the original position unchanged.
fn destination_tile(map: &[Tile], true_row: usize, true_col: usize) -> (usize, usize) {
    match map[true_row * TRUE_MAP_WIDTH + true_col] {
        Tile::North => (true_row - 1, true_col),
        Tile::South => (true_row + 1, true_col),
        Tile::East => (true_row, true_col + 1),
        Tile::West => (true_row, true_col - 1),
        _ => (true_row, true_col),
    }
}

/// Whether or not a tile is a certain distance from the outer edge of the true
/// map. The outer edge of the true map has distance 0. The outer edge of the
/// visible map has distance 2. Entrances and exits should have distance 1.
fn is_at_dist_from_edge(distance: usize, true_row: usize, true_col: usize) -> bool {
    true_row == distance
        || true_col == distance
        || true_row == TRUE_MAP_HEIGHT - 1 - distance
        || true_col == TRUE_MAP_WIDTH - 1 - distance
}

/// A tile is an entrance tile if it is a non-corner path tile that points
/// towards the map and is right outside the outer edge of the visible map.
fn is_entrance(map: &[Tile], true_row: usize, true_col: usize) -> bool {
    if !is_at_dist_from_edge(1, true_row, true_col) {
        // Return early so that we don't overflow off the edges of the map.
        return false;
    }
    let (dest_row, dest_col) = destination_tile(map, true_row, true_col);
    is_at_dist_from_edge(2, dest_row, dest_col)
}

/// A tile is an exit tile if it is a non-corner path tile that points away
/// from the map and is right outside the outer edge of the visible map.
fn is_exit(map: &[Tile], true_row: usize, true_col: usize) -> bool {
    if !is_at_dist_from_edge(1, true_row, true_col) {
        // Return early so that we don't overflow off the edges of the map.
        return false;
    }
    let (dest_row, dest_col) = destination_tile(map, true_row, true_col);
    is_at_dist_from_edge(0, dest_row, dest_col)
}

#[derive(Clone, Copy)]
pub struct Distance {
    /// How many tiles away we are from the entrace/exit.
    pub tiles: u16,
    /// How many quarter-turns we need to make on the way to the entrance/exit.
    /// Clockwise and counterclockwise turns cancel each other out. Sign follows
    /// math convention, so positive is counterclockwise. Well, counterclockwise
    /// if the y-axis goes down instead of up, so not quite conventional.
    pub rotation: i16,
}

pub type Distances = Vec<Option<Distance>>;

impl Default for Distance {
    fn default() -> Self {
        Distance {
            tiles: 0,
            rotation: 0,
        }
    }
}

/// We count counterclockwise rotations as positive and clockwise rotations
/// as negative. Clockwise and counterclockwise are measured by the direction
/// of the path, from entrance to exit, not by the direction of movement
/// along the path. So east to north corners result in positive rotation for
/// walkers both with positive and with negative speed. We do this to be
/// consistent with how tiles work: add up the tile distance going toward
/// the entrance and going toward the exit, and you get a fixed value. Adding
/// up the rotation should act similarly. It should give the net rotation from
/// entrance to exit (with the edge case of double counting the current tile).
fn rotation_amount(tile: Tile) -> i16 {
    match tile {
        Tile::EastToNorth | Tile::NorthToWest | Tile::WestToSouth | Tile::SouthToEast => 1,
        Tile::NorthToEast | Tile::WestToNorth | Tile::SouthToWest | Tile::EastToSouth => -1,
        _ => 0,
    }
}

/// Associate each path tile with the distance to the exit.
/// Used for targeting enemies moving toward the exit.
pub fn generate_dist_from_exit(map: &[Tile]) -> Distances {
    let mut distances: Distances = map.iter().map(|_| None).collect();

    // BFS

    let mut frontier = Vec::new();

    for true_row in 0..TRUE_MAP_HEIGHT {
        for true_col in 0..TRUE_MAP_WIDTH {
            if is_exit(map, true_row, true_col) {
                distances[true_row * TRUE_MAP_WIDTH + true_col] = Some(Default::default());
                frontier.push((true_row, true_col));
            }
        }
    }

    while !frontier.is_empty() {
        let mut new_frontier = Vec::new();
        for (true_row, true_col) in frontier {
            let distance = distances[true_row * TRUE_MAP_WIDTH + true_col].unwrap_or_default();
            let (prev_row, prev_col) = match map[true_row * TRUE_MAP_WIDTH + true_col] {
                Tile::North | Tile::NorthToEast | Tile::NorthToWest => (true_row + 1, true_col),
                Tile::South | Tile::SouthToEast | Tile::SouthToWest => (true_row - 1, true_col),
                Tile::East | Tile::EastToNorth | Tile::EastToSouth => (true_row, true_col - 1),
                Tile::West | Tile::WestToNorth | Tile::WestToSouth => (true_row, true_col + 1),
                _ => continue,
            };
            let rotation = rotation_amount(map[true_row * TRUE_MAP_WIDTH + true_col]);
            if distances[prev_row * TRUE_MAP_WIDTH + prev_col].is_none() {
                distances[prev_row * TRUE_MAP_WIDTH + prev_col] = Some(Distance {
                    tiles: distance.tiles + 1,
                    rotation: distance.rotation + rotation,
                });
                new_frontier.push((prev_row, prev_col));
            }
        }
        frontier = new_frontier;
    }

    distances
}

/// Associate each path tile with the distance to the entrance.
/// Used for targeting enemies moving backwards toward the entrance.
pub fn generate_dist_from_entrance(map: &[Tile]) -> Distances {
    let mut distances: Distances = map.iter().map(|_| None).collect();

    // BFS

    let mut frontier = Vec::new();

    for true_row in 0..TRUE_MAP_HEIGHT {
        for true_col in 0..TRUE_MAP_WIDTH {
            if is_entrance(map, true_row, true_col) {
                distances[true_row * TRUE_MAP_WIDTH + true_col] = Some(Default::default());
                frontier.push((true_row, true_col));
            }
        }
    }

    while !frontier.is_empty() {
        let mut new_frontier = Vec::new();
        for (true_row, true_col) in frontier {
            let distance = distances[true_row * TRUE_MAP_WIDTH + true_col].unwrap_or_default();
            let (next_row, next_col) = match map[true_row * TRUE_MAP_WIDTH + true_col] {
                Tile::North | Tile::EastToNorth | Tile::WestToNorth => (true_row - 1, true_col),
                Tile::South | Tile::EastToSouth | Tile::WestToSouth => (true_row + 1, true_col),
                Tile::East | Tile::NorthToEast | Tile::SouthToEast => (true_row, true_col + 1),
                Tile::West | Tile::NorthToWest | Tile::SouthToWest => (true_row, true_col - 1),
                _ => continue,
            };
            let rotation = rotation_amount(map[true_row * TRUE_MAP_WIDTH + true_col]);
            if distances[next_row * TRUE_MAP_WIDTH + next_col].is_none() {
                distances[next_row * TRUE_MAP_WIDTH + next_col] = Some(Distance {
                    tiles: distance.tiles + 1,
                    rotation: distance.rotation + rotation,
                });
                new_frontier.push((next_row, next_col));
            }
        }
        frontier = new_frontier;
    }

    distances
}

/// Rotate a velocity vector clockwise, assuming the x-axis points left and the
/// y-axis points down.
fn rotate_clockwise(direction: Velocity) -> Velocity {
    Velocity {
        dx: -direction.dy,
        dy: direction.dx,
    }
}

pub fn calc_dist_from_exit(map: &[Tile], distances: &Distances, x: f32, y: f32) -> f32 {
    let direction = walk_direction(map, x, y);
    let right_direction = rotate_clockwise(direction);

    let (true_row, true_col) = true_row_col(x, y);

    let x_remainder = x - true_col as f32 * f32::TILE_SIZE;
    let y_remainder = y - true_row as f32 * f32::TILE_SIZE;

    // We want the remainders to be relative to the center of the tile.
    let x_remainder = x_remainder - 0.5;
    let y_remainder = y_remainder - 0.5;

    let forward_distance = x_remainder * direction.dx + y_remainder * direction.dy;
    let lateral_distance = x_remainder * right_direction.dx + y_remainder * right_direction.dy;

    if let Some(Some(discrete_distance)) = distances.get(true_row * TRUE_MAP_WIDTH + true_col) {
        let tile = map[true_row * TRUE_MAP_WIDTH + true_col];

        // At corners, the rotation depends on which direction we are traveling.
        let corner_rotation_diff = match (tile, direction) {
            (Tile::EastToNorth, v) if v == EAST => 1,
            (Tile::EastToSouth, v) if v == EAST => -1,
            (Tile::WestToSouth, v) if v == WEST => 1,
            (Tile::WestToNorth, v) if v == WEST => -1,
            (Tile::NorthToWest, v) if v == NORTH => 1,
            (Tile::NorthToEast, v) if v == NORTH => -1,
            (Tile::SouthToEast, v) if v == SOUTH => 1,
            (Tile::SouthToWest, v) if v == SOUTH => -1,
            _ => 0,
        };

        let rotation = discrete_distance.rotation + corner_rotation_diff;

        // Distance if we are at the center of the tile
        discrete_distance.tiles as f32 * f32::TILE_SIZE
            // Getting closer reduces the distance
            - forward_distance // getting closer reduces the distance
            // Being farther to the right means traveling farther around
            // counterclockwise turns
            + lateral_distance * rotation as f32
    } else {
        f32::INFINITY
    }
}

pub fn calc_dist_from_entrance(map: &[Tile], distances: &Distances, x: f32, y: f32) -> f32 {
    let direction = walk_direction(map, x, y);
    // We are rotating clockwise, but since we are moving backwards towards the
    // entrance, remember that this points to the left relative to movement
    let right_direction = rotate_clockwise(direction);

    let (true_row, true_col) = true_row_col(x, y);

    let x_remainder = x - true_col as f32 * f32::TILE_SIZE;
    let y_remainder = y - true_row as f32 * f32::TILE_SIZE;

    // We want the remainders to be relative to the center of the tile.
    let x_remainder = x_remainder - 0.5;
    let y_remainder = y_remainder - 0.5;

    let forward_distance = x_remainder * direction.dx + y_remainder * direction.dy;
    let lateral_distance = x_remainder * right_direction.dx + y_remainder * right_direction.dy;

    if let Some(Some(discrete_distance)) = distances.get(true_row * TRUE_MAP_WIDTH + true_col) {
        let tile = map[true_row * TRUE_MAP_WIDTH + true_col];

        // At corners, the rotation depends on which direction we are traveling.
        let corner_rotation_diff = match (tile, direction) {
            (Tile::EastToNorth, v) if v == NORTH => 1,
            (Tile::WestToNorth, v) if v == NORTH => -1,
            (Tile::EastToSouth, v) if v == SOUTH => -1,
            (Tile::WestToSouth, v) if v == SOUTH => 1,
            (Tile::NorthToEast, v) if v == EAST => -1,
            (Tile::SouthToEast, v) if v == EAST => 1,
            (Tile::NorthToWest, v) if v == WEST => 1,
            (Tile::SouthToWest, v) if v == WEST => -1,
            _ => 0,
        };

        let rotation = discrete_distance.rotation + corner_rotation_diff;

        // Distance if we are at the center of the tile
        discrete_distance.tiles as f32 * f32::TILE_SIZE
            // Getting farther increases the distance
            + forward_distance // getting closer reduces the distance
            // Being farther to the right means we haved traveled farther around
            // counterclockwise turns
            + lateral_distance * rotation as f32
    } else {
        f32::INFINITY
    }
}

#[cfg(test)]
mod tests {
    use super::super::{parse, MAP_0, MAP_HEIGHT, MAP_WIDTH};
    use super::*;

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn total_distance_should_be_constant(
            x in 0_f32..MAP_WIDTH as f32 * f32::TILE_SIZE,
            y in 0_f32..MAP_HEIGHT as f32 * f32::TILE_SIZE,
        ) {
            let map = &parse(&MAP_0);

            let entrance_distances = generate_dist_from_entrance(map);
            let entrance_dist = calc_dist_from_entrance(map, &entrance_distances, x, y);

            let exit_distances = generate_dist_from_exit(map);
            let exit_dist = calc_dist_from_exit(map, &exit_distances, x, y);

            prop_assert!((entrance_dist == f32::INFINITY) == (exit_dist == f32::INFINITY),
                "Entrance and exit distances must both be infinity or neither may be infinity.");

            prop_assume!(entrance_dist != f32::INFINITY);

            let net_distance = entrance_dist + exit_dist;
            let correct_distance = 101_f32 * f32::TILE_SIZE;
            let epsilon = 1e-3;
            prop_assert!((net_distance - correct_distance).abs() < epsilon,
                "Net distance was {}, should be {}", net_distance, correct_distance);
        }
    }
}
