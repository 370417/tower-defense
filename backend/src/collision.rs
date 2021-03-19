//! Functions for finding collisions and intersections between geometric objects
//! and between game entities.

use crate::map::{has_border, Constants, Tile};

/// Resolve collisions between a circular entity and the edges of the path
/// in a certain map. If the entity overlaps a wall in multiple dimensions,
/// it will be pushed out along the shorter overlap.
///
/// Assumption: radius <= 0.5 * tile size
///
/// Return the velocity that was applied to the entity to make it no longer
/// collide.
pub fn resolve_collisions(map: &[Tile], x: &mut f32, y: &mut f32, radius: f32) -> (f32, f32) {
    let low_row = (*y / f32::TILE_SIZE + 1.5) as usize;
    let low_col = (*x / f32::TILE_SIZE + 1.5) as usize;
    let high_row = low_row + 1;
    let high_col = low_col + 1;

    let border_x = (high_col as f32 - 2.0) * f32::TILE_SIZE;
    let border_y = (high_row as f32 - 2.0) * f32::TILE_SIZE;

    let has_low_vert_border = has_border(map, low_row, low_col, low_row, high_col);
    let has_high_vert_border = has_border(map, high_row, low_col, high_row, high_col);

    let has_low_horiz_border = has_border(map, low_row, low_col, high_row, low_col);
    let has_high_horiz_border = has_border(map, low_row, high_col, high_row, high_col);

    // We might hit vertical and horizontal borders on the same tick.
    // In this case, we should handle the smaller overlap first.
    // The order in which we resolve collisions (x first or y first) only
    // matters when hitting an acute corner (I think).

    let border_x_dist = (border_x - *x).abs();
    let border_y_dist = (border_y - *y).abs();

    let overlaps_low_x = *x < border_x;
    let overlaps_high_x = *x > border_x;

    let overlaps_low_y = *y < border_y;
    let overlaps_high_y = *y > border_y;

    let resolve_x = |x: &mut f32| {
        if border_x_dist < radius {
            if overlaps_low_y && has_low_vert_border {
                *x = border_x + (*x - border_x).signum() * radius;
            } else if overlaps_high_y && has_high_vert_border {
                *x = border_x + (*x - border_x).signum() * radius;
            }
        }
    };

    let resolve_y = |y: &mut f32| {
        if border_y_dist < radius {
            if overlaps_low_x && has_low_horiz_border {
                *y = border_y + (*y - border_y).signum() * radius;
            } else if overlaps_high_x && has_high_horiz_border {
                *y = border_y + (*y - border_y).signum() * radius;
            }
        }
    };

    let old_x = *x;
    let old_y = *y;

    // let mut correction_distance = 0.0;

    // First resolve orthogonal collisions (ones where the entity ends up
    // tangent to an edge)

    if border_x_dist < border_y_dist {
        resolve_x(x);
        resolve_y(y);
    } else {
        resolve_y(y);
        resolve_x(x);
    }

    // Then resolve corner collisions (ones where the entity ends up tangent
    // to a corner)

    if has_high_horiz_border || has_high_vert_border || has_low_horiz_border || has_low_vert_border
    {
        let distance_from_center_squared =
            border_x_dist * border_x_dist + border_y_dist * border_y_dist;
        if distance_from_center_squared < radius * radius {
            let distance_from_center = distance_from_center_squared.sqrt();
            *x = border_x + (*x - border_x) / distance_from_center * radius;
            *y = border_y + (*y - border_y) / distance_from_center * radius;
        }
    }

    (*x - old_x, *y - old_y)
}

/// Find the two points were a circle and line (defined by two points)
/// intersect. If they do not touch, or if they are tangent to each other,
/// return None.
///
/// Assumption: (x1, y1) != (x2, y2)
pub fn circle_line_intersection(
    circle_x: f32,
    circle_y: f32,
    circle_r: f32,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
) -> Option<((f32, f32), (f32, f32))> {
    // Variable names from https://mathworld.wolfram.com/Circle-LineIntersection.html

    // Try f64 to see if precision is a problem

    let circle_x = circle_x as f64;
    let circle_y = circle_y as f64;
    let circle_r = circle_r as f64;
    let x1 = x1 as f64;
    let y1 = y1 as f64;
    let x2 = x2 as f64;
    let y2 = y2 as f64;

    // Move the plane so that the circle is centered on the origin
    let x1 = x1 - circle_x;
    let y1 = y1 - circle_y;
    let x2 = x2 - circle_x;
    let y2 = y2 - circle_y;

    let dx = x2 - x1;
    let dy = y2 - y1;
    let dr_squared = dx * dx + dy * dy;
    let determinant = x1 * y2 - x2 * y1;

    let discriminant = circle_r * circle_r * dr_squared - determinant * determinant;

    // Because of floating point, ignore the possibility of tangents
    if discriminant > 0.0 {
        let discriminant_sqrt = discriminant.sqrt();

        let x1 = (determinant * dy + dy.signum() * dx * discriminant_sqrt) / dr_squared;
        let x2 = (determinant * dy - dy.signum() * dx * discriminant_sqrt) / dr_squared;

        let y1 = (-determinant * dx + dy.abs() * discriminant_sqrt) / dr_squared;
        let y2 = (-determinant * dx - dy.abs() * discriminant_sqrt) / dr_squared;

        // Move the plane back to its original location

        let x1 = x1 + circle_x;
        let x2 = x2 + circle_x;

        let y1 = y1 + circle_y;
        let y2 = y2 + circle_y;

        Some(((x1 as f32, y1 as f32), (x2 as f32, y2 as f32)))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use proptest::prelude::*;

    proptest! {
        /// Intersection points should be at a distance `radius` from the
        /// center of the circle. And intersection points should lie along the
        /// line (tested via determinant/cross product).
        #[test]
        fn intersects_circle_and_line(
            circle_x in -5_000_f32..5_000_f32,
            circle_y in -5_000_f32..5_000_f32,
            circle_r in 0_f32..5_000_f32,
            x1 in -5_000_f32..5_000_f32,
            y1 in -5_000_f32..5_000_f32,
            x2 in -5_000_f32..5_000_f32,
            y2 in -5_000_f32..5_000_f32,
            t in 0_f32..1_f32,
        ) {
            let epsilon = 1e-3_f32;

            // Don't test degenerate lines
            let x_neq = (x1 - x2).abs() > epsilon;
            let y_neq = (y1 - y2).abs() > epsilon;
            prop_assume!(x_neq || y_neq);

            let intersections = circle_line_intersection(circle_x, circle_y, circle_r, x1, y1, x2, y2);

            if let Some(((ix1, iy1), (ix2, iy2))) = intersections {
                // Points must lie on the circle

                let dx = ix1 - circle_x;
                let dy = iy1 - circle_y;
                let distance_squared = dx * dx + dy * dy;
                prop_assert!(distance_squared > (circle_r - epsilon) * (circle_r - epsilon), "Point {},{} was too close to circle center. Expected distance: {}. Actual distance: {}", ix1, iy1, circle_r, distance_squared.sqrt());
                prop_assert!(distance_squared < (circle_r + epsilon) * (circle_r + epsilon), "Point {},{} was too far from circle center. Expected distance: {}. Actual distance: {}", ix1, iy1, circle_r, distance_squared.sqrt());

                let dx = ix2 - circle_x;
                let dy = iy2 - circle_y;
                let distance_squared = dx * dx + dy * dy;
                prop_assert!(distance_squared > (circle_r - epsilon) * (circle_r - epsilon), "Point {},{} was too close to circle center. Expected distance: {}. Actual distance: {}", ix2, iy2, circle_r, distance_squared.sqrt());
                prop_assert!(distance_squared < (circle_r + epsilon) * (circle_r + epsilon), "Point {},{} was too far from circle center. Expected distance: {}. Actual distance: {}", ix2, iy2, circle_r, distance_squared.sqrt());

                // Points must lie on the line.
                // (check if the determinant is around 0)

                let dx = x2 - x1;
                let dy = y2 - y1;

                // Divide the determinant by the distance squared between
                // x1,y1 and x2,y2 to sort of normalize error.
                // Because of this normalization, use a stricter epsilon
                let epsilon = 1e-6_f32;

                let dx1 = ix1 - x1;
                let dy1 = iy1 - y1;

                let determinant = (dx * dy1 - dx1 * dy) / (dx * dx + dy * dy);
                prop_assert!(determinant.abs() < epsilon, "Point {},{} was not on the line. Determinant: {}", ix1, iy1, determinant);

                let dx2 = ix2 - x1;
                let dy2 = iy2 - y1;

                let determinant = (dx * dy2 - dx2 * dy) / (dx * dx + dy * dy);
                prop_assert!(determinant.abs() < epsilon, "Point {},{} was not on the line. Determinant: {}", ix2, iy2, determinant);
            } else {
                // Pick a point along the line and test its distance to the
                // center of the circle.
                let x = x1 + (x2 - x1) * t;
                let y = y1 + (y2 - y1) * t;
                let distance_squared = (circle_x - x) * (circle_x - x) + (circle_y - y) * (circle_y - y);
                prop_assert!(distance_squared + epsilon > circle_r * circle_r);
            }
        }
    }
}
