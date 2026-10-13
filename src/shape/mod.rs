pub mod fill;

use crate::bounds::Bounds;
use crate::color::Hsv;
use crate::mark::Mark;
use crate::shape::fill::Fill;
use crate::stroke::Point;
const SAMPLE_DEPTH: f32 = 0.5;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spoke {
    pub angle: f32,
    pub reach: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Shape {
    pub centre: Point,
    pub radius: f32,
    pub rotation: f32,
    pub aspect: f32,
    pub soft: f32,
    pub spokes: Vec<Spoke>,
    pub fill: Fill<f32>,
    pub color: Hsv,
    pub opacity: f32,
}

impl Shape {
    pub fn vertices(&self) -> Vec<Point> {
        let (sin, cos) = self.rotation.sin_cos();
        self.spokes
            .iter()
            .map(|spoke| {
                let (across, along) = spoke.angle.sin_cos();
                let (long, short) = (along * self.radius * spoke.reach, across * self.radius * spoke.reach * self.aspect);
                Point { x: self.centre.x + long * cos - short * sin, y: self.centre.y + long * sin + short * cos }
            })
            .collect()
    }

    pub fn samples(&self) -> Vec<Point> {
        std::iter::once(self.centre).chain(self.vertices().into_iter().map(|vertex| self.centre.lerp(vertex, SAMPLE_DEPTH))).collect()
    }

    pub fn bounds(&self, width: usize, height: usize) -> Bounds {
        Bounds::around(self.centre.x, self.centre.y, self.radius + self.fill.reach() + self.edge() + 1.0, width, height)
    }

    pub fn mark(&self, width: usize, height: usize) -> Mark {
        let bounds = self.bounds(width, height);
        let vertices = self.vertices();
        let (sin, cos) = self.fill.frame(self.rotation).sin_cos();
        let edge = self.edge();
        let mut mark = Mark::new(bounds);
        mark.add(bounds, self.color.to_rgb(), |x, y| {
            let (dx, dy) = (x - self.centre.x, y - self.centre.y);
            let (u, v) = (dx * cos + dy * sin, dy * cos - dx * sin);
            self.fill.coverage(signed_distance(&vertices, Point { x, y }), edge, u, v) * self.opacity
        });
        mark
    }

    fn edge(&self) -> f32 {
        self.soft / 2.0 + 0.5
    }
}

fn signed_distance(vertices: &[Point], point: Point) -> f32 {
    let (mut inside, mut nearest) = (false, f32::MAX);
    for (index, a) in vertices.iter().enumerate() {
        let b = vertices[(index + 1) % vertices.len()];
        if (a.y > point.y) != (b.y > point.y) && point.x < a.x + (point.y - a.y) / (b.y - a.y) * (b.x - a.x) {
            inside = !inside;
        }
        nearest = nearest.min(segment_distance(point, *a, b));
    }
    if inside { -nearest } else { nearest }
}

fn segment_distance(point: Point, a: Point, b: Point) -> f32 {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let along = ((point.x - a.x) * dx + (point.y - a.y) * dy) / (dx * dx + dy * dy).max(f32::EPSILON);
    point.distance(a.lerp(b, along.clamp(0.0, 1.0)))
}
