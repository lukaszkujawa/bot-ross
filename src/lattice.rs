use crate::stroke::Point;
use rand::{Rng, RngExt};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Grain {
    pub angle: f32,
    pub along: f32,
}

pub fn lattice(width: usize, height: usize, spacing: f32, grain: Grain, scatter: f32, coverage: impl Fn(Point) -> f32, rng: &mut impl Rng) -> Vec<Point> {
    let (sin, cos) = grain.angle.sin_cos();
    let (w, h) = (width as f32, height as f32);
    let (along, across): (Vec<f32>, Vec<f32>) = [[0.0, 0.0], [w, 0.0], [0.0, h], [w, h]].iter().map(|[x, y]| (x * cos + y * sin, y * cos - x * sin)).unzip();
    let extent = |values: &[f32]| (values.iter().copied().fold(f32::INFINITY, f32::min), values.iter().copied().fold(f32::NEG_INFINITY, f32::max));
    let ((u0, u1), (v0, v1)) = (extent(&along), extent(&across));
    let mut points = Vec::new();
    let mut v = v0 + spacing / 2.0;
    while v < v1 {
        let mut u = u0 + rng.random_range(0.0..grain.along);
        while u < u1 {
            let su = u + rng.random_range(-0.5..0.5) * scatter * grain.along;
            let sv = v + rng.random_range(-0.5..0.5) * scatter * spacing;
            let point = Point { x: su * cos - sv * sin, y: su * sin + sv * cos };
            if (0.0..w).contains(&point.x) && (0.0..h).contains(&point.y) && rng.random_range(0.0..1.0) < coverage(point) {
                points.push(point);
            }
            u += grain.along;
        }
        v += spacing;
    }
    points
}
