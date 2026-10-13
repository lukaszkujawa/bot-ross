use crate::color::Hsv;
use crate::reference::Reference;
use crate::stroke::{Curve, Point, Pressure, Stroke};
use rand::{Rng, RngExt};
use serde::Deserialize;
use std::f32::consts::TAU;

const TURN: f32 = TAU / 24.0;
const STRETCH: [f32; 2] = [0.8, 1.25];
const SHIFT: f32 = 0.5;
const REBEND: f32 = 0.25;

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct StrokeType {
    pub radius: [f32; 2],
    pub length: [f32; 2],
    #[serde(default = "default_bend")]
    pub bend: [f32; 2],
    #[serde(default)]
    pub pressure: PressureShape,
    #[serde(default = "default_jitter")]
    pub jitter: f32,
}

fn default_bend() -> [f32; 2] {
    [0.0, 0.3]
}

fn default_jitter() -> f32 {
    0.05
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq)]
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
        for (key, [min, max]) in [("radius", self.radius), ("length", self.length)] {
            if !(min > 0.0 && min <= max) {
                problems.push(format!("{label}: {key} must be [min, max] with 0 < min <= max"));
            }
        }
        if !(self.bend[0] >= 0.0 && self.bend[0] <= self.bend[1]) {
            problems.push(format!("{label}: bend must be [min, max] with 0 <= min <= max"));
        }
        if !(0.0..=1.0).contains(&self.jitter) {
            problems.push(format!("{label}: jitter must be between 0 and 1"));
        }
        problems
    }

    pub fn random(&self, start: Point, reference: &Reference, rng: &mut impl Rng) -> Stroke {
        let mut pick = |range: [f32; 2]| rng.random_range(range[0]..=range[1]);
        let (radius, length, angle) = (pick(self.radius), pick(self.length), pick([0.0, TAU]));
        let (dx, dy) = (angle.cos(), angle.sin());
        let along = |t: f32, sideways: f32| Point { x: start.x + dx * length * t - dy * sideways, y: start.y + dy * length * t + dx * sideways };
        let sign = |rng: &mut dyn FnMut([f32; 2]) -> f32| if rng([0.0, 1.0]) < 0.5 { -1.0 } else { 1.0 };
        let bends = [pick(self.bend) * length * sign(&mut pick), pick(self.bend) * length * sign(&mut pick)];
        let path = Curve { from: start, control: [along(1.0 / 3.0, bends[0]), along(2.0 / 3.0, bends[1])], to: along(1.0, 0.0) };
        let color = self.jittered(Hsv::from_rgb(reference.mean_along(&path)), &mut pick);
        Stroke { path, pressure: self.pressure.profile(), color, radius, seed: rng.random() }
    }

    pub fn mutate(&self, stroke: &Stroke, rng: &mut impl Rng) -> Stroke {
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
                let (sin, cos) = pick([-TURN, TURN]).sin_cos();
                mutant.path = stroke.path.map(|p| Point { x: from.x + (p.x - from.x) * cos - (p.y - from.y) * sin, y: from.y + (p.x - from.x) * sin + (p.y - from.y) * cos });
            }
            2 => {
                let chord = from.distance(stroke.path.to).max(f32::EPSILON);
                let factor = (pick(STRETCH) * chord).clamp(self.length[0], self.length[1]) / chord;
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
            5 => mutant.color = self.jittered(stroke.color, &mut pick),
            _ => mutant.seed = seed,
        }
        mutant
    }

    fn jittered(&self, color: Hsv, pick: &mut impl FnMut([f32; 2]) -> f32) -> Hsv {
        let jitter = [-self.jitter, self.jitter];
        Hsv { hue: color.hue, saturation: (color.saturation + pick(jitter)).clamp(0.0, 1.0), value: (color.value + pick(jitter)).clamp(0.0, 1.0) }
    }
}
