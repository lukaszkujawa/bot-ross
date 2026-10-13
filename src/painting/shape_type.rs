use crate::color::{Hsv, OPAQUE, Swatch, pick_swatch};
use crate::flow::Flow;
use crate::reference::Reference;
use crate::shape::{Shape, Spoke};
use crate::shape::fill::{Fill, Span};
use crate::stroke::Point;
use rand::{Rng, RngExt};
use serde::{Deserialize, Serialize};
use std::f32::consts::{PI, TAU};

const TURN: f32 = TAU / 24.0;
const STRETCH: [f32; 2] = [0.8, 1.25];
const SHIFT: f32 = 0.5;
const PATCH_TOLERANCE: f32 = 0.12;
const SHORTEST_GRAB: f32 = 0.15;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ShapeType {
    pub points: Points,
    pub radius: [f32; 2],
    #[serde(default)]
    pub irregular: f32,
    #[serde(default)]
    pub follow: f32,
    #[serde(default = "full")]
    pub aspect: Span,
    #[serde(default = "none")]
    pub soft: Span,
    #[serde(default = "yes")]
    pub overlap: bool,
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
    #[serde(default = "one")]
    pub opacity: f32,
    #[serde(default)]
    pub fill: Fill<Span>,
}

fn one() -> f32 {
    1.0
}

fn color_distance(a: [u16; 3], b: [u16; 3]) -> f32 {
    (0..3).map(|c| ((a[c] as f32 - b[c] as f32) / OPAQUE as f32).powi(2)).sum::<f32>().sqrt()
}

fn default_jitter() -> f32 {
    0.05
}

fn full() -> Span {
    Span::Fixed(1.0)
}

fn yes() -> bool {
    true
}

fn none() -> Span {
    Span::Fixed(0.0)
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum Points {
    Fixed(usize),
    Range([usize; 2]),
}

impl Points {
    pub fn range(self) -> [usize; 2] {
        match self {
            Self::Fixed(count) => [count, count],
            Self::Range(range) => range,
        }
    }
}

impl ShapeType {
    pub fn validate(&self, label: &str) -> Vec<String> {
        let mut problems = Vec::new();
        let [min, max] = self.points.range();
        if !(min >= 3 && min <= max) {
            problems.push(format!("{label}: points must be at least 3, or [min, max] with 3 <= min <= max"));
        }
        if !(self.radius[0] > 0.0 && self.radius[0] <= self.radius[1]) {
            problems.push(format!("{label}: radius must be [min, max] with 0 < min <= max"));
        }
        for (key, value) in [("irregular", self.irregular), ("follow", self.follow), ("jitter", self.jitter)] {
            if !(0.0..=1.0).contains(&value) {
                problems.push(format!("{label}: {key} must be between 0 and 1"));
            }
        }
        let [thinnest, widest] = self.aspect.range();
        if !(thinnest > 0.0 && thinnest <= widest && widest <= 1.0) {
            problems.push(format!("{label}: aspect must be a value or [min, max] with 0 < min <= max <= 1"));
        }
        let [sharpest, softest] = self.soft.range();
        if !(sharpest >= 0.0 && sharpest <= softest) {
            problems.push(format!("{label}: soft must be a value or [min, max] with 0 <= min <= max"));
        }
        if !(self.opacity > 0.0 && self.opacity <= 1.0) {
            problems.push(format!("{label}: opacity must be greater than 0 and at most 1"));
        }
        for (key, value) in [("contrast", self.contrast), ("saturation", self.saturation), ("value", self.value)] {
            if value < 0.0 {
                problems.push(format!("{label}: {key} must not be negative"));
            }
        }
        problems.extend(self.fill.validate(label));
        problems
    }

    pub fn random(&self, start: Point, reference: &Reference, flow: &Flow, rng: &mut impl Rng) -> Shape {
        let [min, max] = self.points.range();
        let count = rng.random_range(min..=max);
        let mut pick = |range: [f32; 2]| rng.random_range(range[0]..=range[1]);
        let radius = pick(self.radius);
        let (tangent, strength) = flow.at(start);
        let rotation = tangent + pick([-1.0, 1.0]) * (1.0 - self.follow * strength) * PI;
        let aspect = pick(self.aspect.range());
        let soft = pick(self.soft.range());
        let spokes = self.spokes(count, &mut pick).into_iter().map(|spoke| self.grabbed(spoke, start, rotation, radius, reference)).collect();
        let fill = self.fill.pick(&mut pick);
        let unlit = Hsv { hue: 0.0, saturation: 0.0, value: 0.0 };
        let shape = Shape { centre: start, radius, rotation, aspect, soft, spokes, fill, color: unlit, opacity: self.opacity };
        let base = pick_swatch(&self.palette, &mut pick).unwrap_or_else(|| Hsv::from_rgb(reference.mean_at(shape.samples().into_iter())));
        let color = base.saturated(self.saturation).dimmed(self.value).contrasted(self.contrast).jittered(self.jitter, &mut pick);
        Shape { color, ..shape }
    }

    pub fn mutate(&self, shape: &Shape, rng: &mut impl Rng) -> Shape {
        let operator = rng.random_range(0..7);
        let mut pick = |range: [f32; 2]| rng.random_range(range[0]..=range[1]);
        let mut mutant = shape.clone();
        match operator {
            0 => mutant.centre = Point { x: shape.centre.x + pick([-SHIFT, SHIFT]) * shape.radius, y: shape.centre.y + pick([-SHIFT, SHIFT]) * shape.radius },
            1 => mutant.rotation = shape.rotation + pick([-TURN, TURN]),
            2 => mutant.radius = (shape.radius * pick(STRETCH)).clamp(self.radius[0], self.radius[1]),
            3 => {
                let count = shape.spokes.len();
                let index = ((pick([0.0, 1.0]) * count as f32) as usize).min(count - 1);
                let before = if index == 0 { shape.spokes[count - 1].angle - TAU } else { shape.spokes[index - 1].angle };
                let after = if index + 1 == count { shape.spokes[0].angle + TAU } else { shape.spokes[index + 1].angle };
                let (middle, half) = ((before + after) / 2.0, (after - before) / 2.0 * self.irregular);
                mutant.spokes[index] = Spoke { angle: pick([middle - half, middle + half]), reach: pick(self.reach()) };
            }
            4 => mutant.color = shape.color.jittered(self.jitter, &mut pick),
            5 => {
                let [thinnest, widest] = self.aspect.range();
                mutant.aspect = (shape.aspect * pick(STRETCH)).clamp(thinnest, widest);
            }
            _ => {
                let [min, max] = self.points.range();
                let count = shape.spokes.len();
                if pick([0.0, 1.0]) < 0.5 && count > min {
                    mutant.spokes = self.spokes(count - 1, &mut pick);
                } else if count < max {
                    mutant.spokes = self.spokes(count + 1, &mut pick);
                }
            }
        }
        mutant
    }

    fn grabbed(&self, spoke: Spoke, centre: Point, rotation: f32, radius: f32, reference: &Reference) -> Spoke {
        if self.follow <= 0.0 {
            return spoke;
        }
        let (sin, cos) = (rotation + spoke.angle).sin_cos();
        let home = reference.sample(centre);
        let departed = |step: &f32| color_distance(reference.sample(Point { x: centre.x + cos * step, y: centre.y + sin * step }), home) > PATCH_TOLERANCE;
        let blocked = (1..=radius.ceil() as usize).map(|step| step as f32).find(departed);
        let patch = blocked.map_or(1.0, |step| (step / radius).max(SHORTEST_GRAB));
        Spoke { reach: spoke.reach + (patch - spoke.reach) * self.follow, ..spoke }
    }

    fn spokes(&self, count: usize, pick: &mut impl FnMut([f32; 2]) -> f32) -> Vec<Spoke> {
        let mut scattered: Vec<f32> = (0..count).map(|_| pick([0.0, TAU])).collect();
        scattered.sort_by(f32::total_cmp);
        let step = TAU / count as f32;
        scattered
            .iter()
            .enumerate()
            .map(|(index, scattered)| {
                let even = index as f32 * step;
                Spoke { angle: even + (scattered - even) * self.irregular, reach: pick(self.reach()) }
            })
            .collect()
    }

    fn reach(&self) -> [f32; 2] {
        [1.0 - self.irregular, 1.0]
    }
}
