use crate::bounds::Bounds;
use crate::brush::field::Field;
use crate::brush::paint::Paint;
use crate::color::{Hsv, OPAQUE, Premultiplied};
use crate::layer::Layer;
use crate::stroke::{Point, Stroke};
use rand::{Rng, RngExt};
use serde::Deserialize;

const STEPS_PER_RADIUS: f32 = 2.0;
const PAPER_SEED: u64 = 0x5A9E_7B13;
const PAPER_CELL: f32 = 6.0;
const BLOOM_CELL: f32 = 1.6;
const FADE_CELL: f32 = 1.2;
const FADE_VARIATION: f32 = 0.7;
const DRIFT_CELL: f32 = 2.5;
const RIM_BAND: f32 = 0.2;
const RIM_DARKENING: f32 = 0.25;
const WET_BLEED: f32 = 0.6;

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Wash {
    pub bloom: f32,
    pub softness: f32,
    pub rim: f32,
    pub granulation: f32,
    pub pooling: f32,
    pub drift: f32,
}

impl Default for Wash {
    fn default() -> Self {
        Self { bloom: 0.4, softness: 0.12, rim: 0.8, granulation: 0.3, pooling: 0.3, drift: 0.04 }
    }
}

#[derive(Clone, Copy)]
struct Site {
    wet: f32,
    edge: f32,
    softness: f32,
}

#[derive(Clone, Copy)]
struct Deposit {
    pigment: f32,
    inside: f32,
}

struct Sample {
    centre: Point,
    radius: f32,
    remaining: f32,
}

impl Wash {
    pub fn paint(&self, layer: &mut Layer, stroke: &Stroke, paint: &Paint, rng: &mut impl Rng) -> Bounds {
        let bounds = self.extent(stroke, layer);
        let width = bounds.right - bounds.left;
        let index = |x: usize, y: usize| (y - bounds.top) * width + (x - bounds.left);
        let bloom = Field::new(rng.random(), [stroke.radius * BLOOM_CELL; 2]);
        let fade = Field::new(rng.random(), [stroke.radius * FADE_CELL; 2]);
        let mut sites = vec![Site { wet: 0.0, edge: 1.0, softness: self.softness }; bounds.area()];
        layer.read(bounds, |x, y, pixel| {
            let wet = pixel.alpha as f32 / OPAQUE as f32;
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            let softness = self.softness * (1.0 + FADE_VARIATION * lobed(fade.at(px, py))) + wet * WET_BLEED;
            sites[index(x, y)] = Site { wet, edge: 1.0 + self.bloom * lobed(bloom.at(px, py)), softness };
        });
        let deposits = self.sweep(stroke, paint, bounds, &sites);
        let paper = Field::new(PAPER_SEED, [PAPER_CELL, PAPER_CELL]);
        let drift = Field::new(rng.random(), [stroke.radius * DRIFT_CELL; 2]);
        layer.blend(bounds, |x, y| {
            let (site, deposit) = (sites[index(x, y)], deposits[index(x, y)]);
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            let settling = ((deposit.inside - 1.0 + site.softness + RIM_BAND) / RIM_BAND).clamp(0.0, 1.0) * (1.0 - site.wet);
            let pooled = centred(drift.at(px, py));
            let texture = (1.0 + self.granulation * centred(paper.at(px, py))) * (1.0 + self.pooling * pooled);
            let pigment = deposit.pigment * (1.0 + self.rim * settling) * texture;
            let color = self.tinted(stroke.color, pooled, pigment);
            Premultiplied::from_rgb(color.to_rgb(), 1.0 - (1.0 - stroke.opacity).powf(pigment.max(0.0)))
        })
    }

    fn tinted(&self, color: Hsv, pooled: f32, pigment: f32) -> Hsv {
        let concentrated = (pigment - 1.0).max(0.0);
        Hsv { hue: (color.hue + self.drift * pooled).rem_euclid(1.0), ..color }.dimmed(1.0 - RIM_DARKENING * concentrated)
    }

    fn sweep(&self, stroke: &Stroke, paint: &Paint, bounds: Bounds, sites: &[Site]) -> Vec<Deposit> {
        let width = bounds.right - bounds.left;
        let mut deposits = vec![Deposit { pigment: 0.0, inside: f32::MAX }; bounds.area()];
        let steps = (stroke.path.length() / stroke.radius * STEPS_PER_RADIUS).ceil().max(1.0) as usize;
        for step in 0..=steps {
            let progress = step as f32 / steps as f32;
            let sample = Sample { centre: stroke.path.at(progress), radius: stroke.radius * (0.5 + 0.5 * stroke.pressure.at(progress)), remaining: paint.remaining(progress) };
            let reach = Bounds::around(sample.centre.x, sample.centre.y, sample.radius * (1.0 + self.bloom) + 1.0, bounds.right, bounds.bottom);
            for y in reach.top.max(bounds.top)..reach.bottom {
                for x in reach.left.max(bounds.left)..reach.right {
                    let index = (y - bounds.top) * width + (x - bounds.left);
                    let candidate = sample.deposit(Point { x: x as f32 + 0.5, y: y as f32 + 0.5 }, sites[index]);
                    let deposit = &mut deposits[index];
                    deposit.pigment = deposit.pigment.max(candidate.pigment);
                    deposit.inside = deposit.inside.min(candidate.inside);
                }
            }
        }
        deposits
    }

    fn extent(&self, stroke: &Stroke, layer: &Layer) -> Bounds {
        let [xs, ys] = stroke.path.extent();
        Bounds::enclosing(xs, ys, stroke.radius * (1.0 + self.bloom) + 2.0, layer.width, layer.height)
    }
}

impl Sample {
    fn deposit(&self, point: Point, site: Site) -> Deposit {
        let inside = point.distance(self.centre) / self.radius / site.edge;
        let body = ease(((1.0 - inside) / site.softness).clamp(0.0, 1.0));
        Deposit { pigment: body * self.remaining, inside }
    }
}

fn centred(noise: f32) -> f32 {
    noise * 2.0 - 1.0
}

fn lobed(noise: f32) -> f32 {
    (2.0 * centred(noise)).clamp(-1.0, 1.0)
}

fn ease(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}
