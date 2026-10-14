use crate::stroke::Point;
use rand::{Rng, RngExt};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Grain {
    pub angle: f32,
    pub along: Option<f32>,
}

pub fn lattice(width: usize, height: usize, spacing: f32, grain: Grain, scatter: f32, coverage: impl Fn(Point) -> f32, rng: &mut impl Rng) -> Vec<Point> {
    let (sin, cos) = grain.angle.sin_cos();
    let (w, h) = (width as f32, height as f32);
    let (us, vs): (Vec<f32>, Vec<f32>) = [[0.0, 0.0], [w, 0.0], [0.0, h], [w, h]].iter().map(|[x, y]| (x * cos + y * sin, y * cos - x * sin)).unzip();
    let extent = |values: &[f32]| (values.iter().copied().fold(f32::INFINITY, f32::min), values.iter().copied().fold(f32::NEG_INFINITY, f32::max));
    let ((u0, u1), (v0, v1)) = (extent(&us), extent(&vs));
    let at = |u: f32, v: f32| Point { x: u * cos - v * sin, y: u * sin + v * cos };
    let on_canvas = |p: Point| (0.0..w).contains(&p.x) && (0.0..h).contains(&p.y);
    let mut points = Vec::new();
    let mut v = v0 + spacing / 2.0;
    while v < v1 {
        match grain.along {
            Some(step) => {
                let mut u = u0 - rng.random_range(0.0..step);
                while u < u1 {
                    let point = at(u + rng.random_range(-0.5..0.5) * scatter * step, v + rng.random_range(-0.5..0.5) * scatter * spacing);
                    let end = Point { x: point.x + step * cos, y: point.y + step * sin };
                    if (on_canvas(point) || on_canvas(end)) && rng.random_range(0.0..1.0) < coverage(point) {
                        points.push(point);
                    }
                    u += step;
                }
            }
            None => {
                let row = v + rng.random_range(-0.5..0.5) * scatter * spacing;
                let entry = (0..=(u1 - u0).ceil() as usize).map(|offset| at(u0 + offset as f32, row)).find(|p| on_canvas(*p));
                points.extend(entry.filter(|p| rng.random_range(0.0..1.0) < coverage(*p)));
            }
        }
        v += spacing;
    }
    points
}
