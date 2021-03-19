/// From Fast Approximate Distance Functions by Rafael Baptista
pub fn fast_distance(dx: f32, dy: f32) -> f32 {
    let dx = dx.abs();
    let dy = dy.abs();
    1007.0 / 1024.0 * f32::max(dx, dy) + 441.0 / 1024.0 * f32::min(dx, dy)
}
