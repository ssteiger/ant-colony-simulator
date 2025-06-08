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