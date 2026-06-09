use std::f32::consts::PI;

use super::terrain::Terrain;

/// Probe the terrain ahead of the ant (center + two whiskers). If a wall is
/// detected, returns a steering vector toward the most open direction.
pub fn wall_avoidance(
    terrain: &Terrain,
    x: f32,
    y: f32,
    heading: f32,
    probe_dist: f32,
) -> Option<(f32, f32)> {
    let probe = |angle: f32, dist: f32| -> bool {
        terrain.is_solid_at(x + angle.cos() * dist, y + angle.sin() * dist)
    };

    let center = probe(heading, probe_dist);
    let left = probe(heading - 0.7, probe_dist * 0.85);
    let right = probe(heading + 0.7, probe_dist * 0.85);

    if !center && !left && !right {
        return None;
    }

    if center {
        // wall straight ahead: find the most open escape direction
        for &a in &[
            heading - 1.3,
            heading + 1.3,
            heading - 2.2,
            heading + 2.2,
            heading + PI,
        ] {
            if !probe(a, probe_dist * 0.85) {
                return Some((a.cos(), a.sin()));
            }
        }
        let back = heading + PI;
        return Some((back.cos(), back.sin()));
    }

    // only a whisker hit: nudge away from the blocked side
    let away = if left { heading + 0.9 } else { heading - 0.9 };
    Some((away.cos(), away.sin()))
}

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
