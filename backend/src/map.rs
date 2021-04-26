pub mod distances;

use crate::graphics::{render_path_border, render_path_tile};

pub const MAP_WIDTH: usize = 22;
pub const MAP_HEIGHT: usize = 18;

// We use a 2-tile ring of padding around the map to make our lives easier.
// This way we can fill the edges with special terrain and avoid out of bounds
// index problems.

pub const TRUE_MAP_WIDTH: usize = MAP_WIDTH + 4;
pub const TRUE_MAP_HEIGHT: usize = MAP_HEIGHT + 4;

/// Putting constants in a trait allows for using different numerical types
/// without casting or macros.
pub trait Constants {
    const TILE_SIZE: Self;
}

impl Constants for usize {
    const TILE_SIZE: Self = 32;
}

impl Constants for f32 {
    const TILE_SIZE: Self = 32.0;
}

// Textual representation of a map.
// # represents out of bounds impassable tiles
// [space] represents non-path tiles where you can place towers
// > represents a path heading east
// < represents a path heading west
// n represents a path heading north
// v represents a path heading south
// x represents a turn in the path

pub const MAP_0: [&str; TRUE_MAP_HEIGHT] = [
    "##########################",
    "##########################",
    "##                      ##",
    "#>>>x x>>>x    x>>>x x>>>#",
    "#>>xv nx>xv    nx>xv nx>>#",
    "## vx>xn vv    nn vx>xn ##",
    "## x>>>x vv    nn x>>>x ##",
    "##       vv    nn       ##",
    "## x<<<x vv    nn x<<<x ##",
    "## vx<xn vv    nn vx<xn ##",
    "## vv nx<xv    nx<xv nn ##",
    "## vv x<<<x    x<<<x nn ##",
    "## vv                nn ##",
    "## vv                nn ##",
    "## vv  x>>>x  x>>>x  nn ##",
    "## vv  nx>xv  nx>xv  nn ##",
    "## vv  nn vx>>xn vv  nn ##",
    "## vx>>xn x>>>>x vx>>xn ##",
    "## x>>>>x        x>>>>x ##",
    "##                      ##",
    "##########################",
    "##########################",
];

pub const _MAP_1: [&str; TRUE_MAP_HEIGHT] = [
    "##########################",
    "############vv############",
    "##          vv          ##",
    "##          vv          ##",
    "##          vv          ##",
    "##          vv          ##",
    "##        x<xx>x        ##",
    "##       xx    xx       ##",
    "##      xx      xx      ##",
    "##      v        v      ##",
    "#<<<<<<<x        x>>>>>>>#",
    "#<<<<<<<x        x>>>>>>>#",
    "##      n        n      ##",
    "##      xx      xx      ##",
    "##       xx    xx       ##",
    "##        x<xx>x        ##",
    "##          nn          ##",
    "##          nn          ##",
    "##          nn          ##",
    "##          nn          ##",
    "############nn############",
    "##########################",
];

pub const _MAP_2: [&str; TRUE_MAP_HEIGHT] = [
    "##########################",
    "##########################",
    "##                      ##",
    "##      x>>>>x  x>>>>x  ##",
    "#>>>>x  nx>>xv  nx>>xv  ##",
    "#>>>xv  nn  vv  nn  vv  ##",
    "##  vv  nn  vx>>xn  vv  ##",
    "##  vx>>xn  x>>>>x  vv  ##",
    "##  x>>>>x          vv  ##",
    "##                  vv  ##",
    "##  x<<<<<<<<<<<<<<<xv  ##",
    "##  vx<<<<<<<<<<<<<<<x  ##",
    "##  vv                  ##",
    "##  vv          x>>>>x  ##",
    "##  vv  x>>>>x  nx>>xv  ##",
    "##  vv  nx>>xv  nn  vv  ##",
    "##  vv  nn  vv  nn  vx>>>#",
    "##  vx>>xn  vx>>xn  x>>>>#",
    "##  x>>>>x  x>>>>x      ##",
    "##                      ##",
    "##########################",
    "##########################",
];

pub const _MAP_3: [&str; TRUE_MAP_HEIGHT] = [
    "##########################",
    "##########################",
    "##                      ##",
    "##                      ##",
    "##                      ##",
    "##                      ##",
    "##                      ##",
    "#<<<<<<<<<<<<<<<<<<<<<<<<#",
    "#<<<<<<<<<<<<<<<<<<<<<<<<#",
    "#<<<<<<<<<<<<<<<<<<<<<<<<#",
    "##                      ##",
    "##                      ##",
    "#>>>>>>>>>>>>>>>>>>>>>>>>#",
    "#>>>>>>>>>>>>>>>>>>>>>>>>#",
    "#>>>>>>>>>>>>>>>>>>>>>>>>#",
    "##                      ##",
    "##                      ##",
    "##                      ##",
    "##                      ##",
    "##                      ##",
    "##########################",
    "##########################",
];

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    OutOfBounds,
    Empty,
    North,
    South,
    East,
    West,
    // DirectionToDirection means facing one direction and turning to
    // face another direction. It does not mean coming from one direction and
    // heading toward another.
    NorthToEast,
    NorthToWest,
    SouthToEast,
    SouthToWest,
    EastToNorth,
    EastToSouth,
    WestToNorth,
    WestToSouth,
}

/// Turn the textual representation of the map into a more convenient
/// representation for the computer. Specifically, this function figures out
/// which way corners turn.
pub fn parse(map_str: &[&str]) -> Vec<Tile> {
    let mut parsed_map = Vec::with_capacity(TRUE_MAP_WIDTH * TRUE_MAP_HEIGHT);

    for (row, row_str) in map_str.iter().enumerate() {
        for (col, char) in row_str.chars().enumerate() {
            parsed_map.push(match char {
                ' ' => Tile::Empty,
                'n' => Tile::North,
                'v' => Tile::South,
                '>' => Tile::East,
                '<' => Tile::West,
                'x' => {
                    let get_char = |row: usize, col: usize| map_str[row].as_bytes()[col] as char;
                    if get_char(row - 1, col) == 'v' {
                        if get_char(row, col - 1) == '<' {
                            Tile::SouthToWest
                        } else {
                            Tile::SouthToEast
                        }
                    } else if get_char(row + 1, col) == 'n' {
                        if get_char(row, col - 1) == '<' {
                            Tile::NorthToWest
                        } else {
                            Tile::NorthToEast
                        }
                    } else if get_char(row, col - 1) == '>' {
                        if get_char(row - 1, col) == 'n' {
                            Tile::EastToNorth
                        } else {
                            Tile::EastToSouth
                        }
                    } else {
                        if get_char(row - 1, col) == 'n' {
                            Tile::WestToNorth
                        } else {
                            Tile::WestToSouth
                        }
                    }
                }
                _ => Tile::OutOfBounds,
            });
        }
    }

    parsed_map
}

/// A tile counts as an entrance tile if it is just out of the visible area and
/// points inward.
pub fn entrances(map: &[Tile]) -> Vec<(usize, usize)> {
    let mut entrances = Vec::new();

    for row in 0..MAP_HEIGHT {
        let true_row = 2 + row;
        let true_col = 1;

        if map[true_row * TRUE_MAP_WIDTH + true_col] == Tile::East {
            entrances.push((true_row, true_col));
        }

        let true_col = MAP_WIDTH + 1;

        if map[true_row * TRUE_MAP_WIDTH + true_col] == Tile::West {
            entrances.push((true_row, true_col));
        }
    }

    for col in 0..MAP_WIDTH {
        let true_row = 1;
        let true_col = 2 + col;

        if map[true_row * TRUE_MAP_WIDTH + true_col] == Tile::South {
            entrances.push((true_row, true_col));
        }

        let true_row = MAP_HEIGHT + 1;

        if map[true_row * TRUE_MAP_WIDTH + true_col] == Tile::North {
            entrances.push((true_row, true_col));
        }
    }

    entrances
}

/// Whether two adjacent tiles are separated by a border, like between the path
/// and an empty tile or between an empty tile and the edge of the visible map.
/// In the future, we may want to allow for borders between path tiles, for
/// example at tight u-turns. But for now we can ignore that complexity.
///
/// But still, we avoid using the simpler tile_a, tile_b as parameters because
/// future implementations might need to know the positions of tile a and b.
pub fn has_border(
    map: &[Tile],
    true_row_a: usize,
    true_col_a: usize,
    true_row_b: usize,
    true_col_b: usize,
) -> bool {
    let tile_a = map.get(true_row_a * TRUE_MAP_WIDTH + true_col_a);
    let tile_b = map.get(true_row_b * TRUE_MAP_WIDTH + true_col_b);

    (tile_a == Some(&Tile::OutOfBounds))
        || (tile_b == Some(&Tile::OutOfBounds))
        || ((tile_a == Some(&Tile::Empty)) ^ (tile_b == Some(&Tile::Empty)))
}

/// Call the external render functions. Only do this once per level, not once
/// per frame.
pub fn render_map(map: &[Tile]) {
    // Render the map
    for row in 0..MAP_HEIGHT {
        for col in 0..MAP_WIDTH {
            let true_row = row + 2;
            let true_col = col + 2;
            match map[true_row * TRUE_MAP_WIDTH + true_col] {
                Tile::Empty => {}
                _ => render_path_tile(row, col),
            }
        }
    }
    // And the borders of the path
    for row in 1..MAP_HEIGHT {
        for col in 0..MAP_WIDTH {
            let true_row = row + 2;
            let true_col = col + 2;
            if has_border(map, true_row - 1, true_col, true_row, true_col) {
                render_path_border(row, col, true);
            }
        }
    }
    for row in 0..MAP_HEIGHT {
        for col in 1..MAP_WIDTH {
            let true_row = row + 2;
            let true_col = col + 2;
            if has_border(map, true_row, true_col - 1, true_row, true_col) {
                render_path_border(row, col, false);
            }
        }
    }
}

/// Returns the (true_row, true_col) corresponding to an x, y position. Note
/// that x and y can be negative, so we add before dividing.
pub fn true_row_col(x: f32, y: f32) -> (usize, usize) {
    (
        (y + 2.0 * f32::TILE_SIZE) as usize / usize::TILE_SIZE,
        (x + 2.0 * f32::TILE_SIZE) as usize / usize::TILE_SIZE,
    )
}

pub fn tile_center(row: usize, col: usize) -> (f32, f32) {
    let x = (col as f32 + 0.5) * f32::TILE_SIZE;
    let y = (row as f32 + 0.5) * f32::TILE_SIZE;
    (x, y)
}

pub fn true_tile_center(true_row: usize, true_col: usize) -> (f32, f32) {
    let x = (true_col as f32 - 1.5) * f32::TILE_SIZE;
    let y = (true_row as f32 - 1.5) * f32::TILE_SIZE;
    (x, y)
}

pub fn in_bounds(x: f32, y: f32) -> bool {
    let width = MAP_WIDTH as f32 * f32::TILE_SIZE;
    let height = MAP_HEIGHT as f32 * f32::TILE_SIZE;
    x >= 0.0 && y >= 0.0 && x < width && y < height
}
