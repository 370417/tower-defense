//! Temporarily/permanently abandoned -- fluid simulation might be too
//! performance intensive.

use crate::map::{Tile, MAP_HEIGHT, MAP_WIDTH, TRUE_MAP_WIDTH};

/// Fluid simulation takes place on a coarse grid, but not quite as coarse as
/// the tile grid for placing towers. Resolution configures how many times finer
/// the fluid grid is compared to the tile grid.
const FLUID_RESOLUTION: usize = 2;

// The fluid grid is like the visible tile grid but with increased resolution
// and with an extra 1-cell border around the outside. Borders on all four
// sides translates to width and height having 2 extra cells.

const FLUID_WIDTH: usize = 2 + FLUID_RESOLUTION * MAP_WIDTH;
const FLUID_HEIGHT: usize = 2 + FLUID_RESOLUTION * MAP_HEIGHT;
const MAX_VISIBLE_ROW: usize = FLUID_HEIGHT - 2;
const MAX_VISIBLE_COL: usize = FLUID_WIDTH - 2;
const FLUID_VISIBLE_AREA: usize = FLUID_RESOLUTION * FLUID_RESOLUTION * MAP_WIDTH * MAP_HEIGHT;

type FluidArr = [f32; FLUID_WIDTH * FLUID_HEIGHT];

pub struct FluidGrid {
    density: FluidArr,
    dx: FluidArr,
    dy: FluidArr,
}

impl FluidGrid {
    pub fn new() -> FluidGrid {
        FluidGrid {
            density: [0.0; FLUID_WIDTH * FLUID_HEIGHT],
            dx: [0.0; FLUID_WIDTH * FLUID_HEIGHT],
            dy: [0.0; FLUID_WIDTH * FLUID_HEIGHT],
        }
    }
}

fn add_density(grid: &mut FluidGrid) {
    grid.density[fluid_pos_to_index(10, 10)] = 20.0;
}

const DIFFUSION_FACTOR: f32 = 1.0;
const TIME_PER_TICK: f32 = 1.0;
const RELAXATION_ITERATIONS: u32 = 20;

fn diffuse(arr: &mut FluidArr, old_arr: &FluidArr, map: &Vec<Tile>) {
    let a = DIFFUSION_FACTOR * TIME_PER_TICK * FLUID_VISIBLE_AREA as f32;
    let ix = fluid_pos_to_index;

    let in_bounds = |row, col| match map[fluid_pos_to_true_tile_index(row, col)] {
        Tile::Empty | Tile::OutOfBounds => false,
        _ => true,
    };

    for _ in 0..RELAXATION_ITERATIONS {
        for row in 1..=MAX_VISIBLE_ROW {
            for col in 1..=MAX_VISIBLE_COL {
                if !in_bounds(row, col) {
                    continue;
                }
                // The Jos Stam paper uses arr instead of old_arr for the
                // terms being multiplied by a because it is iteratively
                // adjusting arr over and over again. This means iteration
                // order affects our results (since we are too cheap to
                // double buffer). To assuage this a bit, we can alternate
                // iteration order.
                let (row, col) = if (row + col) % 2 == 0 {
                    (row, col)
                } else {
                    (MAX_VISIBLE_ROW - row + 1, MAX_VISIBLE_COL - col + 1)
                };
                arr[ix(row, col)] = old_arr[ix(row, col)]
                    * a
                    * (if in_bounds(row + 1, col) {
                        arr[ix(row + 1, col)]
                    } else {
                        arr[ix(row, col)]
                    } + if in_bounds(row - 1, col) {
                        arr[ix(row - 1, col)]
                    } else {
                        arr[ix(row, col)]
                    } + if in_bounds(row, col + 1) {
                        arr[ix(row, col + 1)]
                    } else {
                        arr[ix(row, col)]
                    } + if in_bounds(row, col - 1) {
                        arr[ix(row, col - 1)]
                    } else {
                        arr[ix(row, col)]
                    })
                    / (1.0 + 4.0 * a);
            }
        }
    }
}

/// Update densities by following velocity backwards in time and averaging the
/// closest four densities.
fn advect(
    density: &mut FluidArr,
    old_density: &FluidArr,
    dx: &FluidArr,
    dy: &FluidArr,
    map: &Vec<Tile>,
) {
    let ix = fluid_pos_to_index;

    for row in 1..=MAX_VISIBLE_ROW {
        for col in 1..=MAX_VISIBLE_COL {
            let x = col as f32 - TIME_PER_TICK * MAX_VISIBLE_COL as f32 * dx[ix(row, col)];
            let y = row as f32 - TIME_PER_TICK * MAX_VISIBLE_ROW as f32 * dy[ix(row, col)];
            let x = x.max(0.5).min(MAX_VISIBLE_COL as f32 + 0.5);
            let y = y.max(0.5).min(MAX_VISIBLE_ROW as f32 + 0.5);
            let col0 = x as usize;
            let col1 = col0 + 1;
            let row0 = y as usize;
            let row1 = row0 + 1;
            let s1 = x - col0 as f32;
            let s0 = 1.0 - s1;
            let t1 = y - row0 as f32;
            let t0 = 1.0 - t1;
            density[ix(row, col)] = s0
                * (t0 * old_density[ix(row0, col0)] + t1 * old_density[ix(row1, col0)])
                + s1 * (t0 * old_density[ix(row0, col1)] + t1 * old_density[ix(row1, col1)]);
        }
    }
    // Boundary conditions: set out of bounds densities or velocities to 0

    let in_bounds = |row, col| match map[fluid_pos_to_true_tile_index(row, col)] {
        Tile::Empty | Tile::OutOfBounds => false,
        _ => true,
    };
    for row in 0..FLUID_HEIGHT {
        for col in 0..FLUID_WIDTH {
            if !in_bounds(row, col) {
                density[ix(row, col)] = 0.0;
            }
        }
    }
}

fn update_density(grid: &mut FluidGrid, old_grid: &mut FluidGrid, map: &Vec<Tile>) {
    add_density(grid);
    // Diffuse puts updated values into old_grid
    diffuse(&mut old_grid.density, &grid.density, map);
    // Advect puts density values back into grid from old_grid
    advect(
        &mut grid.density,
        &old_grid.density,
        &grid.dx,
        &grid.dy,
        map,
    );
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Bounce {
    NoBounce = 0,
    BounceX = 1,
    BounceY = 2,
}

// Jos Stam calls bounce b. Not sure if the name bounce is accurate.
fn set_boundary(bounce: Bounce, arr: &mut FluidArr) {
    let ix = fluid_pos_to_index;

    for row in 1..=MAX_VISIBLE_ROW {
        let sign = if bounce == Bounce::BounceX { -1.0 } else { 1.0 };
        arr[ix(row, 0)] = sign * arr[ix(row, 1)];
        arr[ix(row, MAX_VISIBLE_COL + 1)] = sign * arr[ix(row, MAX_VISIBLE_COL)];
    }

    for col in 1..=MAX_VISIBLE_COL {
        let sign = if bounce == Bounce::BounceY { -1.0 } else { 1.0 };
        arr[ix(0, col)] = sign * arr[ix(1, col)];
        arr[ix(MAX_VISIBLE_ROW + 1, col)] = sign * arr[ix(MAX_VISIBLE_ROW, col)];
    }
}

/// Return the tile index (in a TRUE_MAP_WIDTH x TRUE_MAP_HEIGHT grid) that
/// contains a certain fluid cell given by row and col.
pub fn fluid_pos_to_true_tile_index(row: usize, col: usize) -> usize {
    let resolution = FLUID_RESOLUTION as isize;
    true_tile_pos_to_index(
        (2 + (row as isize - 1) / resolution) as usize,
        (2 + (col as isize - 1) / resolution) as usize,
    )
}

/// Return the top left cell in a given tile. This uses visible tile
/// coordinates, not true tile coordinates.
pub fn tile_pos_to_fluid_index(row: usize, col: usize) -> usize {
    fluid_pos_to_index(1 + row * FLUID_RESOLUTION, 1 + col * FLUID_RESOLUTION)
}

fn true_tile_pos_to_index(row: usize, col: usize) -> usize {
    row * TRUE_MAP_WIDTH + col
}

fn fluid_pos_to_index(row: usize, col: usize) -> usize {
    row * FLUID_WIDTH + col
}
