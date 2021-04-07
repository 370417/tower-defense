//! A tile-based index intended to store entities. There's no need to use this
//! until we reach a high number of entities, probably ~3k or so. For now, I'm
//! leaving the implementation unfinished. Eventually, I might want to
//! dynamically choose between using an index and just brute-force distance
//! queries based on number of mobs in game.

use std::hash::Hash;

use crate::{
    map::{tile_center, true_row_col, Constants, TRUE_MAP_HEIGHT, TRUE_MAP_WIDTH},
    world::Map,
};

pub struct SpatialIndex<T> {
    items: Vec<Map<T, (f32, f32)>>,
}

fn pos_to_index(x: f32, y: f32) -> usize {
    let (true_row, true_col) = true_row_col(x, y);
    true_row * TRUE_MAP_WIDTH + true_col
}

impl<T: Eq + Hash> SpatialIndex<T> {
    pub fn insert(&mut self, item: T, x: f32, y: f32) {
        self.items[pos_to_index(x, y)].insert(item, (x, y));
    }

    pub fn remove(&mut self, item: T, x: f32, y: f32) {
        self.items[pos_to_index(x, y)].remove(&item);
    }

    pub fn update_pos(&mut self, item: T, old_x: f32, old_y: f32, x: f32, y: f32) {
        let old_index = pos_to_index(old_x, old_y);
        let index = pos_to_index(x, y);
        if index == old_index {
            self.items[index]
                .entry(item)
                .and_modify(|pos| *pos = (x, y));
        } else {
            self.items[old_index].remove(&item);
            self.items[index].insert(item, (x, y));
        }
    }

    /// Get the distance squared from (x, y) to the farthest corner of a certain tile.
    pub fn outer_dist_squared(x: f32, y: f32, true_row: usize, true_col: usize) -> f32 {
        let center_x = (true_col as f32 - 1.5) * f32::TILE_SIZE;
        let center_y = (true_row as f32 - 1.5) * f32::TILE_SIZE;
        let corner_x = center_x + 0.5 * f32::TILE_SIZE * (center_x - x).signum();
        let corner_y = center_y + 0.5 * f32::TILE_SIZE * (center_y - y).signum();
        let dx = corner_x - x;
        let dy = corner_y - y;
        dx * dx + dy * dy
    }

    /// Get the distance squared from (x, y) to the closest corner of a certain tile.
    pub fn inner_dist_squared(x: f32, y: f32, true_row: usize, true_col: usize) -> f32 {
        let center_x = (true_col as f32 - 1.5) * f32::TILE_SIZE;
        let center_y = (true_row as f32 - 1.5) * f32::TILE_SIZE;
        let corner_x = center_x - 0.5 * f32::TILE_SIZE * (center_x - x).signum();
        let corner_y = center_y - 0.5 * f32::TILE_SIZE * (center_y - y).signum();
        let dx = corner_x - x;
        let dy = corner_y - y;
        dx * dx + dy * dy
    }

    pub fn closest_item(&self, center_x: f32, center_y: f32, radius: f32) -> Option<&T> {
        let mut min_dist_squared = f32::INFINITY;
        for true_row in 0..TRUE_MAP_HEIGHT {
            for true_col in 0..TRUE_MAP_WIDTH {
                let i = true_row * TRUE_MAP_WIDTH + true_col;
                let dist_squared = Self::outer_dist_squared(center_x, center_y, true_row, true_col);
                if !self.items[i].is_empty() && dist_squared < min_dist_squared {
                    min_dist_squared = dist_squared;
                }
            }
        }

        let epsilon = 1.0;
        let min_corner_dist_squared = min_dist_squared;
        let mut best_item = None;
        for true_row in 0..TRUE_MAP_HEIGHT {
            for true_col in 0..TRUE_MAP_WIDTH {
                let i = true_row * TRUE_MAP_WIDTH + true_col;
                let corner_dist_squared =
                    Self::outer_dist_squared(center_x, center_y, true_row, true_col);
                if corner_dist_squared <= min_corner_dist_squared + epsilon {
                    for (item, (x, y)) in &self.items[i] {
                        let dx = x - center_x;
                        let dy = y - center_y;
                        let dist_squared = dx * dx + dy * dy;
                        if dist_squared < min_dist_squared {
                            min_dist_squared = dist_squared;
                            best_item = Some(item);
                        }
                    }
                }
            }
        }
        best_item
    }

    pub fn items_within_circular_range<'a>(
        &'a self,
        center_x: f32,
        center_y: f32,
        radius: f32,
    ) -> impl Iterator<Item = &'a T> {
        self.items
            .iter()
            .enumerate()
            .filter(move |(i, _)| {
                let true_row = i / TRUE_MAP_WIDTH;
                let true_col = i % TRUE_MAP_WIDTH;
                let corner_dist_squared =
                    Self::inner_dist_squared(center_x, center_y, true_row, true_col);
                corner_dist_squared <= radius * radius
            })
            .flat_map(|(_, cell)| cell.iter())
            .filter_map(move |(item, (x, y))| {
                let dx = x - center_x;
                let dy = y - center_y;
                let dist_squared = dx * dx + dy * dy;
                if dist_squared <= radius * radius {
                    Some(item)
                } else {
                    None
                }
            })
    }
}
