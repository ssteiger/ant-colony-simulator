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

/// Default window dimensions for the simulation
pub const DEFAULT_WINDOW_WIDTH: f32 = 1200.0;
pub const DEFAULT_WINDOW_HEIGHT: f32 = 800.0;

/// Calculate the center of the window/world (window center is world center)
pub fn world_center() -> (f32, f32) {
    (0.0, 0.0) // Center at origin for proper camera handling
}

/// Get world bounds that match the window size centered around origin
/// Bevy uses coordinates where (0,0) is the center of the screen
pub fn get_world_bounds(window_width: f32, window_height: f32) -> (f32, f32) {
    (window_width, window_height)
}

/// Get world bounds as min/max coordinates centered around origin
pub fn get_centered_world_bounds(window_width: f32, window_height: f32) -> (f32, f32, f32, f32) {
    let half_width = window_width / 2.0;
    let half_height = window_height / 2.0;
    (-half_width, half_width, -half_height, half_height) // (min_x, max_x, min_y, max_y)
} 