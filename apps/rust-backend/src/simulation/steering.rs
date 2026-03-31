use std::f32::consts::PI;

/// Craig Reynolds-style wander: small random perturbation of heading each tick.
/// Returns (dx, dy) unit direction vector.
pub fn wander_direction(
    heading: f32,
    wander_angle: &mut f32,
    strength: f32,
    rng_val: f32,
) -> (f32, f32) {
    *wander_angle += (rng_val - 0.5) * strength;
    *wander_angle *= 0.92;
    let angle = heading + *wander_angle;
    (angle.cos(), angle.sin())
}

/// Returns unit vector pointing from (fx, fy) toward (tx, ty).
pub fn seek(fx: f32, fy: f32, tx: f32, ty: f32) -> (f32, f32) {
    let dx = tx - fx;
    let dy = ty - fy;
    let dist = (dx * dx + dy * dy).sqrt();
    if dist < 0.001 {
        return (0.0, 0.0);
    }
    (dx / dist, dy / dist)
}

/// Soft repulsion force that grows quadratically as the agent approaches a world edge.
pub fn boundary_avoidance(x: f32, y: f32, w: f32, h: f32, margin: f32) -> (f32, f32) {
    let mut fx = 0.0f32;
    let mut fy = 0.0f32;

    if x < margin {
        fx += (1.0 - x / margin).powi(2);
    } else if x > w - margin {
        fx -= (1.0 - (w - x) / margin).powi(2);
    }

    if y < margin {
        fy += (1.0 - y / margin).powi(2);
    } else if y > h - margin {
        fy -= (1.0 - (h - y) / margin).powi(2);
    }

    (fx, fy)
}

/// Normalize an angle to [-PI, PI].
pub fn normalize_angle(angle: f32) -> f32 {
    let mut a = angle;
    while a > PI {
        a -= 2.0 * PI;
    }
    while a < -PI {
        a += 2.0 * PI;
    }
    a
}

/// Levy flight: returns true if a long-distance jump should occur this tick.
/// When it triggers, a random heading and a speed boost are applied.
/// `rng_val` should be uniform [0,1).
pub fn levy_should_jump(rng_val: f32, cooldown: u32) -> bool {
    cooldown == 0 && rng_val < 0.003
}
