use crate::color::{Hsv, Swatch, pick_swatch};
use crate::flow::Flow;
use crate::painting::length::Length;
use crate::reference::Reference;
use crate::shape::fill::Span;
use crate::stroke::{Curve, Point, Pressure, Stroke};
use rand::{Rng, RngExt};
use serde::{Deserialize, Serialize};
use std::f32::consts::{PI, TAU};

const TURN: f32 = TAU / 24.0;
const TRACE_STEPS: usize = 12;
const STRETCH: [f32; 2] = [0.8, 1.25];
const SHIFT: f32 = 0.5;
const REBEND: f32 = 0.25;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct StrokeType {
    pub radius: [f32; 2],
    pub length: Length,
    #[serde(default = "default_bend")]
    pub bend: [f32; 2],
    #[serde(default)]
    pub pressure: PressureShape,
    #[serde(default = "default_jitter")]
    pub jitter: f32,
    #[serde(default = "one")]
    pub contrast: f32,
    #[serde(default = "one")]
    pub saturation: f32,
    #[serde(default = "one")]
    pub value: f32,
    #[serde(default)]
    pub palette: Vec<Swatch>,
    #[serde(default)]
    pub follow: f32,
    #[serde(default = "one")]
    pub opacity: f32,
    pub aim: Option<[f32; 2]>,
    pub angle: Option<Span>,
    #[serde(default = "half")]
    pub length_bias: f32,
}

fn one() -> f32 {
    1.0
}

fn half() -> f32 {
    0.5
}

fn default_bend() -> [f32; 2] {
    [0.0, 0.3]
}

fn default_jitter() -> f32 {
    0.05
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PressureShape {
    #[default]
    Hand,
    Flat,
    Fading,
}

impl PressureShape {
    pub fn profile(self) -> Pressure {
        Pressure::new(match self {
            Self::Hand => vec![0.3, 0.85, 1.0, 0.8, 0.35],
            Self::Flat => vec![1.0],
            Self::Fading => vec![0.9, 1.0, 0.7, 0.35, 0.1],
        })
    }
}

impl StrokeType {
    pub fn validate(&self, label: &str) -> Vec<String> {
        let mut problems = Vec::new();
        for (key, [min, max]) in [("radius", Some(self.radius)), ("length", self.length.range())].into_iter().filter_map(|(key, range)| Some((key, range?))) {
            if !(min > 0.0 && min <= max) {
                problems.push(format!("{label}: {key} must be [min, max] with 0 < min <= max"));
            }
        }
        if !(self.bend[0] >= 0.0 && self.bend[0] <= self.bend[1]) {
            problems.push(format!("{label}: bend must be [min, max] with 0 <= min <= max"));
        }
        for (key, value) in [("jitter", self.jitter), ("follow", self.follow), ("opacity", self.opacity)] {
            if !(0.0..=1.0).contains(&value) {
                problems.push(format!("{label}: {key} must be between 0 and 1"));
            }
        }
        if !(self.length_bias > 0.0 && self.length_bias < 1.0) {
            problems.push(format!("{label}: length_bias must be between 0 and 1, exclusive"));
        }
        if !(self.opacity > 0.0 && self.opacity <= 1.0) {
            problems.push(format!("{label}: opacity must be greater than 0 and at most 1"));
        }
        for (key, value) in [("contrast", self.contrast), ("saturation", self.saturation), ("value", self.value)] {
            if value < 0.0 {
                problems.push(format!("{label}: {key} must not be negative"));
            }
        }
        problems
    }

    pub fn random(&self, start: Point, reference: &Reference, flow: &Flow, rng: &mut impl Rng) -> Stroke {
        self.drawn(start, None, reference, flow, rng)
    }

    pub fn passing(&self, point: Point, reference: &Reference, flow: &Flow, rng: &mut impl Rng) -> Stroke {
        let along = rng.random_range(0.0..=1.0);
        self.drawn(point, Some(along), reference, flow, rng)
    }

    fn drawn(&self, start: Point, along: Option<f32>, reference: &Reference, flow: &Flow, rng: &mut impl Rng) -> Stroke {
        let mut pick = |range: [f32; 2]| rng.random_range(range[0]..=range[1]);
        let radius = pick(self.radius);
        let length = self.length.range().map(|range| lerp(range, biased(pick([0.0, 1.0]), self.length_bias)));
        let (tangent, strength) = flow.at(start);
        let attraction = self.follow * strength;
        let angle = match self.angle {
            Some(span) => pick(span.range()).to_radians(),
            None => tangent + if pick([0.0, 1.0]) < 0.5 { PI } else { 0.0 } + pick([-1.0, 1.0]) * (1.0 - attraction) * PI,
        };
        let canvas = [reference.width as f32, reference.height as f32];
        let Some(length) = length else {
            let path = Curve::across(start, angle, radius, canvas);
            return self.coloured(path, radius, reference, rng);
        };
        let traced = self.trace(start, angle, length, flow);
        let chord = traced[0].distance(traced[3]).max(f32::EPSILON);
        let (nx, ny) = (-(traced[3].y - traced[0].y) / chord, (traced[3].x - traced[0].x) / chord);
        let mut path = through(traced);
        for control in &mut path.control {
            let bend = pick(self.bend) * length * (1.0 - attraction) * if pick([0.0, 1.0]) < 0.5 { -1.0 } else { 1.0 };
            *control = Point { x: control.x + nx * bend, y: control.y + ny * bend };
        }
        if let Some(along) = along {
            let anchor = path.at(along);
            let from = Point { x: (path.from.x + start.x - anchor.x).clamp(-radius, canvas[0] + radius), y: (path.from.y + start.y - anchor.y).clamp(-radius, canvas[1] + radius) };
            let (dx, dy) = (from.x - path.from.x, from.y - path.from.y);
            path = path.map(|p| Point { x: p.x + dx, y: p.y + dy });
        }
        self.coloured(path, radius, reference, rng)
    }

    fn coloured(&self, path: Curve, radius: f32, reference: &Reference, rng: &mut impl Rng) -> Stroke {
        let mut pick = |range: [f32; 2]| rng.random_range(range[0]..=range[1]);
        let base = pick_swatch(&self.palette, &mut pick).unwrap_or_else(|| Hsv::from_rgb(reference.mean_along(&path)));
        let color = base.saturated(self.saturation).dimmed(self.value).contrasted(self.contrast).jittered(self.jitter, &mut pick);
        Stroke { path, pressure: self.pressure.profile(), color, radius, opacity: self.opacity, seed: rng.random() }
    }

    pub fn typical_length(&self) -> Option<f32> {
        self.length.range().map(|range| lerp(range, self.length_bias))
    }

    fn trace(&self, start: Point, mut direction: f32, length: f32, flow: &Flow) -> [Point; 4] {
        let step = length / TRACE_STEPS as f32;
        let mut point = start;
        let mut passed = [start; 4];
        for i in 1..=TRACE_STEPS {
            let (tangent, strength) = flow.aligned(point, direction);
            let pull = self.follow * strength;
            direction += ((tangent - direction + PI).rem_euclid(TAU) - PI) * pull;
            point = Point { x: point.x + direction.cos() * step, y: point.y + direction.sin() * step };
            if i % (TRACE_STEPS / 3) == 0 {
                passed[i / (TRACE_STEPS / 3)] = point;
            }
        }
        passed
    }

    pub fn mutate(&self, stroke: &Stroke, canvas: [f32; 2], rng: &mut impl Rng) -> Stroke {
        let (operator, seed) = (rng.random_range(0..7), rng.random());
        let mut pick = |range: [f32; 2]| rng.random_range(range[0]..=range[1]);
        let from = stroke.path.from;
        let mut mutant = stroke.clone();
        match operator {
            0 => {
                let (dx, dy) = (pick([-SHIFT, SHIFT]) * stroke.radius, pick([-SHIFT, SHIFT]) * stroke.radius);
                mutant.path = stroke.path.map(|p| Point { x: p.x + dx, y: p.y + dy });
            }
            1 => {
                let heading = (stroke.path.to.y - from.y).atan2(stroke.path.to.x - from.x);
                let (sin, cos) = (self.turned(heading, pick([-TURN, TURN])) - heading).sin_cos();
                mutant.path = stroke.path.map(|p| Point { x: from.x + (p.x - from.x) * cos - (p.y - from.y) * sin, y: from.y + (p.x - from.x) * sin + (p.y - from.y) * cos });
            }
            2 => {
                let Some([shortest, longest]) = self.length.range() else { return mutant };
                let chord = from.distance(stroke.path.to).max(f32::EPSILON);
                let factor = (pick(STRETCH) * chord).clamp(shortest, longest) / chord;
                mutant.path = stroke.path.map(|p| Point { x: from.x + (p.x - from.x) * factor, y: from.y + (p.y - from.y) * factor });
            }
            3 => {
                let chord = from.distance(stroke.path.to).max(f32::EPSILON);
                let (nx, ny) = (-(stroke.path.to.y - from.y) / chord, (stroke.path.to.x - from.x) / chord);
                let offset = pick([-REBEND, REBEND]) * chord * self.bend[1];
                let which = pick([0.0, 1.0]) < 0.5;
                let control = &mut mutant.path.control[which as usize];
                *control = Point { x: control.x + nx * offset, y: control.y + ny * offset };
            }
            4 => mutant.radius = (stroke.radius * pick(STRETCH)).clamp(self.radius[0], self.radius[1]),
            5 => mutant.color = stroke.color.jittered(self.jitter, &mut pick),
            _ => mutant.seed = seed,
        }
        if self.length.range().is_none() {
            let path = mutant.path;
            mutant.path = Curve::across(path.at(0.5), (path.to.y - path.from.y).atan2(path.to.x - path.from.x), mutant.radius, canvas);
        }
        mutant
    }

    fn turned(&self, heading: f32, turn: f32) -> f32 {
        let Some(span) = self.angle else { return heading + turn };
        let [low, high] = span.range().map(f32::to_radians);
        let centre = (low + high) / 2.0;
        (centre + (heading + turn - centre + PI).rem_euclid(TAU) - PI).clamp(low, high)
    }
}

fn lerp([low, high]: [f32; 2], t: f32) -> f32 {
    low + (high - low) * t
}

fn biased(t: f32, bias: f32) -> f32 {
    t / ((1.0 / bias - 2.0) * (1.0 - t) + 1.0)
}

fn through([p0, p1, p2, p3]: [Point; 4]) -> Curve {
    let mix = |a: f32, b: f32, c: f32, d: f32| (-5.0 * a + 18.0 * b - 9.0 * c + 2.0 * d) / 6.0;
    Curve {
        from: p0,
        control: [Point { x: mix(p0.x, p1.x, p2.x, p3.x), y: mix(p0.y, p1.y, p2.y, p3.y) }, Point { x: mix(p3.x, p2.x, p1.x, p0.x), y: mix(p3.y, p2.y, p1.y, p0.y) }],
        to: p3,
    }
}
