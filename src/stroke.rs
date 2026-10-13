use crate::color::Hsv;

const LENGTH_SEGMENTS: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn lerp(self, other: Point, t: f32) -> Point {
        Point { x: self.x + (other.x - self.x) * t, y: self.y + (other.y - self.y) * t }
    }

    pub fn distance(self, other: Point) -> f32 {
        (other.x - self.x).hypot(other.y - self.y)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Curve {
    pub from: Point,
    pub control: [Point; 2],
    pub to: Point,
}

impl Curve {
    pub fn line(from: Point, to: Point) -> Self {
        Self { from, control: [from.lerp(to, 1.0 / 3.0), from.lerp(to, 2.0 / 3.0)], to }
    }

    pub fn at(&self, t: f32) -> Point {
        let [a, b] = self.control;
        let (ab, bc, cd) = (self.from.lerp(a, t), a.lerp(b, t), b.lerp(self.to, t));
        ab.lerp(bc, t).lerp(bc.lerp(cd, t), t)
    }

    pub fn beside(&self, t: f32, sideways: f32) -> Point {
        let (dx, dy) = self.tangent(t);
        let centre = self.at(t);
        Point { x: centre.x - dy * sideways, y: centre.y + dx * sideways }
    }

    pub fn map(&self, f: impl Fn(Point) -> Point) -> Self {
        Self { from: f(self.from), control: [f(self.control[0]), f(self.control[1])], to: f(self.to) }
    }

    pub fn extent(&self) -> [[f32; 2]; 2] {
        let points: Vec<Point> = (0..=LENGTH_SEGMENTS).map(|i| self.at(i as f32 / LENGTH_SEGMENTS as f32)).collect();
        let span = |pick: fn(&Point) -> f32| [points.iter().map(pick).fold(f32::MAX, f32::min), points.iter().map(pick).fold(f32::MIN, f32::max)];
        [span(|p| p.x), span(|p| p.y)]
    }

    pub fn length(&self) -> f32 {
        (0..LENGTH_SEGMENTS)
            .map(|segment| {
                let t = segment as f32 / LENGTH_SEGMENTS as f32;
                self.at(t).distance(self.at(t + 1.0 / LENGTH_SEGMENTS as f32))
            })
            .sum()
    }

    pub fn tangent(&self, t: f32) -> (f32, f32) {
        let step = 1.0 / LENGTH_SEGMENTS as f32;
        let (before, after) = (self.at((t - step).max(0.0)), self.at((t + step).min(1.0)));
        let length = before.distance(after).max(f32::EPSILON);
        ((after.x - before.x) / length, (after.y - before.y) / length)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Pressure {
    keys: Vec<f32>,
}

impl Pressure {
    pub fn new(keys: Vec<f32>) -> Self {
        assert!(!keys.is_empty(), "pressure needs at least one key");
        Self { keys }
    }

    pub fn constant(value: f32) -> Self {
        Self::new(vec![value])
    }

    pub fn at(&self, t: f32) -> f32 {
        let position = t.clamp(0.0, 1.0) * (self.keys.len() - 1) as f32;
        let index = (position.floor() as usize).min(self.keys.len() - 1);
        let next = (index + 1).min(self.keys.len() - 1);
        let eased = (1.0 - ((position - index as f32) * std::f32::consts::PI).cos()) / 2.0;
        self.keys[index] + (self.keys[next] - self.keys[index]) * eased
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Stroke {
    pub path: Curve,
    pub pressure: Pressure,
    pub color: Hsv,
    pub radius: f32,
    pub seed: u64,
}
