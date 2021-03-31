use std::f32::consts::{PI, TAU};

pub fn accel_geometric(x: &mut f32, dx: &mut f32, max_dx: f32, ddx: f32) {
    // Speed gets multiplied by r every tick. Ranges from 0 to 1.
    // Named after the r in a geometric series, and after resistance.
    let r = max_dx / (max_dx + ddx);

    // Don't care about overshooting

    *dx = r * (*dx + ddx);
    *x += *dx;
}

pub fn ease_to_x_geometric(
    x: &mut f32,
    dx: &mut f32,
    target_x: f32,
    max_dx: f32,
    ddx: f32,
    domain: Domain,
) {
    // Speed gets multiplied by r every tick. Ranges from 0 to 1.
    // Named after the r in a geometric series, and after resistance.
    let mut r = max_dx / (max_dx + ddx);

    // let mut dist_to_target = target_x - *x;
    // dist_to_target %= TAU;
    // if dist_to_target > PI {
    //     dist_to_target -= TAU;
    // } else if dist_to_target < PI {
    //     dist_to_target += TAU;
    // }
    // let target_x = *x + dist_to_target;

    // If we apply no acceleration, where will x come to rest?
    let brake_distance = *dx / (1.0 - r);
    let x_at_rest = *x + brake_distance;

    // Accelerate so that x_at_rest becomes closer to target_x
    let mut ideal_ddx = target_x - x_at_rest;

    // Keep angles small
    if let Domain::Radian = domain {
        ideal_ddx %= TAU;
        if ideal_ddx > PI {
            ideal_ddx -= TAU;
        } else if ideal_ddx < -PI {
            ideal_ddx += TAU;
        }

        if ideal_ddx.abs() > PI / 2.0 {
            // Increase turning radius after missing the target.
            // This gives a better approach angle after turning back to face
            // the target.
            r *= 0.95;
        }
    }

    // Clamp instead of normalizing acceleration (ie don't increase low
    // acceleration). This helps mitigate overshooting but won't prevent it
    // entirely, as accelerating increases future brake distance when not
    // moving at top speed. That's okay though, as overshooting acceleration
    // will be compensated for on the next frame.
    let clamped_ddx = ideal_ddx.min(ddx).max(-ddx);

    *dx = r * (*dx + clamped_ddx);
    *x += *dx;

    *x %= TAU;
    if *x > PI {
        *x -= TAU;
    } else if *x < -PI {
        *x += TAU;
    }
}

pub enum Domain {
    /// -inf to +inf
    NumberLine,
    /// -pi to +pi
    Radian,
}
