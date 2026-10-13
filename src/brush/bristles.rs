use crate::brush::paint::Paint;
use crate::brush::walk::Walk;
use crate::color::Hsv;
use crate::stroke::{Point, Stroke};
use rand::{Rng, RngExt};
use serde::Deserialize;

const SAMPLES_PER_RADIUS: f32 = 2.0;
const THINNEST_DAB: f32 = 0.6;

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Spacing {
    #[default]
    Random,
    Even,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Bristles {
    pub density: f32,
    pub max: Option<usize>,
    #[serde(default)]
    pub spacing: Spacing,
    pub spread: f32,
    #[serde(default)]
    pub splay: f32,
    pub thickness: [f32; 2],
    #[serde(default = "one_pixel")]
    pub tip: f32,
    #[serde(default)]
    pub taper: [f32; 2],
    #[serde(default)]
    pub ripple: f32,
    pub opacity: [f32; 2],
    pub shade: f32,
    pub wobble: f32,
    #[serde(default)]
    pub onset: [f32; 2],
    pub reach: [f32; 2],
}

fn one_pixel() -> f32 {
    1.0
}

pub struct Hair {
    pub samples: Vec<Sample>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sample {
    pub point: Point,
    pub radius: f32,
    pub opacity: f32,
    pub pressure: f32,
    pub color: Hsv,
}

impl Bristles {
    pub fn hairs(&self, stroke: &Stroke, paint: &Paint, rng: &mut impl Rng) -> Vec<Hair> {
        let count = self.count(stroke.radius);
        (0..count).map(|index| self.hair(self.position(index, count, rng), stroke, paint, rng)).collect()
    }

    pub fn count(&self, radius: f32) -> usize {
        let wanted = (self.density * radius).round().max(1.0) as usize;
        self.max.map_or(wanted, |max| wanted.min(max))
    }

    fn position(&self, index: usize, count: usize, rng: &mut impl Rng) -> f32 {
        match self.spacing {
            Spacing::Random => rng.random_range(-1.0..=1.0),
            Spacing::Even if count > 1 => index as f32 / (count - 1) as f32 * 2.0 - 1.0,
            Spacing::Even => 0.0,
        }
    }

    fn hair(&self, position: f32, stroke: &Stroke, paint: &Paint, rng: &mut impl Rng) -> Hair {
        let mut pick = |range: [f32; 2]| rng.random_range(range[0]..=range[1]);
        let sideways = position * self.spread * stroke.radius;
        let (onset, reach) = (pick(self.onset), pick(self.reach));
        let opacity = pick(self.opacity) * stroke.opacity;
        let full = pick(self.thickness) * stroke.radius;
        let shade = pick([-self.shade, self.shade]);
        let steps = (stroke.path.length() * (reach - onset) / stroke.radius * SAMPLES_PER_RADIUS).ceil().max(1.0);
        let mut drift = Walk::new(self.wobble * stroke.radius, steps);
        let mut ripple = Walk::new(self.ripple, steps);
        let mut lightness = Walk::new(self.shade, steps);
        let mut load = Walk::new(paint.unevenness, steps);
        let mut contact = paint.contact();
        let samples = (0..=steps as usize)
            .map(|step| {
                let along = step as f32 / steps;
                let progress = onset + along * (reach - onset);
                let (remaining, pressure) = (paint.remaining(progress), stroke.pressure.at(progress));
                let deposited = remaining * (1.0 - load.next(rng).abs()) * contact.next(paint, remaining, rng);
                let splayed = sideways * (1.0 - self.splay * (1.0 - pressure)) + drift.next(rng);
                let radius = (self.tip + (full - self.tip) * self.taper_at(along)) * (1.0 + ripple.next(rng));
                Sample {
                    point: stroke.path.beside(progress, splayed),
                    radius: radius.max(THINNEST_DAB),
                    opacity: (opacity * deposited).clamp(0.0, 1.0),
                    pressure,
                    color: stroke.color.shaded(shade + lightness.next(rng)),
                }
            })
            .collect();
        Hair { samples }
    }

    fn taper_at(&self, along: f32) -> f32 {
        let [rise, fall] = self.taper;
        let rising = if rise > 0.0 { (along / rise).min(1.0) } else { 1.0 };
        let falling = if fall > 0.0 { ((1.0 - along) / fall).min(1.0) } else { 1.0 };
        let t = rising.min(falling);
        t * t * (3.0 - 2.0 * t)
    }
}
