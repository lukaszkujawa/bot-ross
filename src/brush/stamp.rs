use crate::brush::field::Field;
use crate::mark::Mark;
use crate::brush::paint::Paint;
use crate::brush::tip::Tip;
use crate::brush::walk::Walk;
use crate::color::{OPAQUE, Premultiplied};
use crate::layer::Layer;
use crate::bounds::Bounds;
use crate::stroke::{Point, Stroke};
use rand::{Rng, RngExt};
use serde::Deserialize;
use std::path::PathBuf;

const STREAK_BANDS: usize = 48;
const CARRY_RATE: f32 = 0.5;
const PATCH_STRETCH: f32 = 5.0;
const PATCH_EDGE: f32 = 0.015;

#[derive(Clone, Debug, Deserialize)]
pub struct Stamp {
    pub tip: PathBuf,
    pub spacing: f32,
    pub ripple: f32,
    pub wobble: f32,
    pub streaks: f32,
    pub smear: f32,
}

impl Stamp {
    pub fn paint(&self, tip: &Tip, layer: &mut Layer, stroke: &Stroke, paint: &Paint, rng: &mut impl Rng) -> Bounds {
        let length = stroke.path.length();
        let steps = (length / (stroke.radius * self.spacing)).ceil().max(1.0);
        let streaks = Streaks::new(self.streaks, rng);
        let patch = stroke.radius / paint.grain.max(f32::EPSILON);
        let patches = Field::new(rng.random(), [patch * PATCH_STRETCH, patch]);
        let mut width = Walk::new(self.ripple, steps);
        let mut drift = Walk::new(self.wobble * stroke.radius, steps);
        let mut load = Walk::new(paint.unevenness, steps);
        let mut carried = Premultiplied::TRANSPARENT;
        let rgb = stroke.color.to_rgb();
        let mut mark = Mark::new(self.extent(stroke, layer));
        for step in 0..=steps as usize {
            let progress = step as f32 / steps;
            let pressure = stroke.pressure.at(progress);
            let remaining = paint.remaining(progress);
            let thickness = remaining * (1.0 - load.next(rng).abs()) * (0.5 + 0.5 * pressure);
            let opacity = covered(thickness) * stroke.opacity;
            let bare = paint.skip + (1.0 - remaining) * (1.0 - paint.skip);
            let across = stroke.radius * (1.0 + width.next(rng));
            let along = across * tip.aspect();
            let centre = stroke.path.beside(progress, drift.next(rng));
            let (dx, dy) = stroke.path.tangent(progress);
            carried = carried.mix(layer.average(Point { x: centre.x + dx * along, y: centre.y + dy * along }, along.max(1.0)), CARRY_RATE);
            let color = smeared(rgb, carried, self.smear);
            let bounds = Bounds::around(centre.x, centre.y, across.max(along) + 1.0, layer.width, layer.height);
            mark.add(bounds, color, |x, y| {
                let (rx, ry) = (x - centre.x, y - centre.y);
                let u = (rx * dx + ry * dy) / (2.0 * along) + 0.5;
                let v = (ry * dx - rx * dy) / (2.0 * across) + 0.5;
                let contact = patches.at(progress * length + (u - 0.5) * 2.0 * along, (v - 0.5) * 2.0 * across);
                tip.coverage(u, v) * opacity * streaks.at(v) * deposited(contact, bare)
            });
        }
        mark.composite(layer)
    }

    fn extent(&self, stroke: &Stroke, layer: &Layer) -> Bounds {
        let [xs, ys] = stroke.path.extent();
        let reach = stroke.radius * (1.0 + 2.0 * self.ripple + self.wobble) + 2.0;
        Bounds::enclosing(xs, ys, reach, layer.width, layer.height)
    }
}

fn deposited(contact: f32, bare: f32) -> f32 {
    if bare <= 0.0 { 1.0 } else { ((contact - bare) / PATCH_EDGE + 0.5).clamp(0.0, 1.0) }
}

fn covered(thickness: f32) -> f32 {
    1.0 - (1.0 - thickness.clamp(0.0, 1.0)).powi(3)
}

fn smeared(rgb: [f32; 3], carried: Premultiplied, smear: f32) -> [f32; 3] {
    let weight = smear * carried.alpha as f32 / OPAQUE as f32;
    let pushed = carried.to_rgb();
    [0, 1, 2].map(|index| rgb[index] + (pushed[index] - rgb[index]) * weight)
}

struct Streaks {
    bands: Vec<f32>,
    strength: f32,
}

impl Streaks {
    fn new(strength: f32, rng: &mut impl Rng) -> Self {
        let raw: Vec<f32> = (0..STREAK_BANDS).map(|_| rng.random_range(0.0..=1.0)).collect();
        let smooth = |values: &[f32]| (0..values.len()).map(|i| (values[i.saturating_sub(1)] + values[i] + values[(i + 1).min(values.len() - 1)]) / 3.0).collect::<Vec<_>>();
        Self { bands: smooth(&smooth(&raw)), strength }
    }

    fn at(&self, v: f32) -> f32 {
        let band = ((v * self.bands.len() as f32) as usize).min(self.bands.len() - 1);
        1.0 - self.strength * self.bands[band]
    }
}
