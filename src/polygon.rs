pub fn contains(points: &[[f32; 2]], x: f32, y: f32) -> bool {
    let mut inside = false;
    for (i, a) in points.iter().enumerate() {
        let b = points[(i + 1) % points.len()];
        if (a[1] > y) != (b[1] > y) && x < a[0] + (y - a[1]) * (b[0] - a[0]) / (b[1] - a[1]) {
            inside = !inside;
        }
    }
    inside
}

pub fn distance_to_outline(points: &[[f32; 2]], x: f32, y: f32) -> f32 {
    points
        .iter()
        .enumerate()
        .map(|(i, a)| {
            let b = points[(i + 1) % points.len()];
            let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
            let t = (((x - a[0]) * dx + (y - a[1]) * dy) / (dx * dx + dy * dy).max(f32::EPSILON)).clamp(0.0, 1.0);
            (x - a[0] - t * dx).hypot(y - a[1] - t * dy)
        })
        .fold(f32::INFINITY, f32::min)
}
