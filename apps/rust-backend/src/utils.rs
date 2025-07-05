// Utility functions for the ant colony simulator

pub fn distance(pos1: (f32, f32), pos2: (f32, f32)) -> f32 {
    let dx = pos1.0 - pos2.0;
    let dy = pos1.1 - pos2.1;
    (dx * dx + dy * dy).sqrt()
}

pub fn normalize_angle(angle: f32) -> f32 {
    let two_pi = 2.0 * std::f32::consts::PI;
    let mut normalized = angle % two_pi;
    if normalized < 0.0 {
        normalized += two_pi;
    }
    normalized
}

pub fn clamp(value: f32, min: f32, max: f32) -> f32 {
    value.max(min).min(max)
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn angle_between_points(from: (f32, f32), to: (f32, f32)) -> f32 {
    (to.1 - from.1).atan2(to.0 - from.0)
}

// ============================================================================
// WORLD CONSTANTS
// ============================================================================

/// Default world dimensions for the simulation
pub const WORLD_WIDTH: f32 = 1000.0;
pub const WORLD_HEIGHT: f32 = 1000.0;

/// Calculate the center of the world
pub fn world_center() -> (f32, f32) {
    (WORLD_WIDTH / 2.0, WORLD_HEIGHT / 2.0)
} 