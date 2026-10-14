use crate::color::Hsv;
use crate::polygon::{contains, distance_to_outline};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    #[default]
    Add,
    Subtract,
    Intersect,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields, default)]
pub struct Selection {
    pub mode: Mode,
    pub feather: Option<f32>,
    pub ellipse: Option<Ellipse>,
    pub rectangle: Option<Rectangle>,
    pub polygon: Option<Vec<[f32; 2]>>,
    pub luminance: Option<[f32; 2]>,
    pub hue: Option<[f32; 2]>,
    pub saturation: Option<[f32; 2]>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Ellipse {
    pub centre: [f32; 2],
    pub radius: f32,
    #[serde(default = "one")]
    pub aspect: f32,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Rectangle {
    pub from: [f32; 2],
    pub to: [f32; 2],
}

#[derive(Clone, Copy)]
pub struct Frame {
    pub x: f32,
    pub y: f32,
    pub size: f32,
}

pub struct Prepared {
    pub mode: Mode,
    feather: f32,
    shapes: Vec<Shape>,
    ranges: Vec<Range>,
}

enum Shape {
    Ellipse { centre: [f32; 2], radius: f32, aspect: f32 },
    Polygon { points: Vec<[f32; 2]>, size: f32 },
}

struct Range {
    channel: Channel,
    low: f32,
    high: f32,
}

#[derive(Clone, Copy)]
enum Channel {
    Luminance,
    Hue,
    Saturation,
}

fn one() -> f32 {
    1.0
}

impl Selection {
    pub fn validate(&self, label: &str) -> Vec<String> {
        let mut problems = Vec::new();
        if self.ellipse.is_none() && self.rectangle.is_none() && self.polygon.is_none() && self.luminance.is_none() && self.hue.is_none() && self.saturation.is_none() {
            problems.push(format!("{label}: a selection needs a shape or a range"));
        }
        if self.ellipse.is_some_and(|ellipse| ellipse.radius <= 0.0 || ellipse.aspect <= 0.0) {
            problems.push(format!("{label}: ellipse radius and aspect must be greater than 0"));
        }
        if self.rectangle.is_some_and(|rectangle| rectangle.from[0] >= rectangle.to[0] || rectangle.from[1] >= rectangle.to[1]) {
            problems.push(format!("{label}: rectangle from must lie above and left of to"));
        }
        if self.polygon.as_ref().is_some_and(|points| points.len() < 3) {
            problems.push(format!("{label}: a polygon needs at least 3 points"));
        }
        for (key, range) in [("luminance", self.luminance), ("saturation", self.saturation)] {
            if range.is_some_and(|[low, high]| !(0.0..=1.0).contains(&low) || !(0.0..=1.0).contains(&high) || low > high) {
                problems.push(format!("{label}: {key} must be [low, high] between 0 and 1"));
            }
        }
        if self.hue.is_some_and(|[low, high]| !(0.0..=1.0).contains(&low) || !(0.0..=1.0).contains(&high)) {
            problems.push(format!("{label}: hue must be [from, to] between 0 and 1, wrapping past 1 when from is above to"));
        }
        if self.feather.is_some_and(|feather| feather < 0.0) {
            problems.push(format!("{label}: feather must not be negative"));
        }
        problems
    }

    pub fn prepare(&self, frame: Frame, feather: f32) -> Prepared {
        let place = |[x, y]: [f32; 2]| [x * frame.x, y * frame.y];
        let mut shapes = Vec::new();
        if let Some(ellipse) = self.ellipse {
            shapes.push(Shape::Ellipse { centre: place(ellipse.centre), radius: ellipse.radius * frame.size, aspect: ellipse.aspect });
        }
        if let Some(rectangle) = self.rectangle {
            let corners = vec![rectangle.from, [rectangle.to[0], rectangle.from[1]], rectangle.to, [rectangle.from[0], rectangle.to[1]]];
            shapes.push(Shape::polygon(corners.into_iter().map(place).collect()));
        }
        if let Some(points) = &self.polygon {
            shapes.push(Shape::polygon(points.iter().copied().map(place).collect()));
        }
        let ranges = [(Channel::Luminance, self.luminance), (Channel::Hue, self.hue), (Channel::Saturation, self.saturation)].into_iter().filter_map(|(channel, range)| range.map(|[low, high]| Range { channel, low, high })).collect();
        Prepared { mode: self.mode, feather: self.feather.unwrap_or(feather), shapes, ranges }
    }
}

impl Prepared {
    pub fn coverage(&self, x: f32, y: f32, rgb: [f32; 3]) -> f32 {
        let shapes = self.shapes.iter().map(|shape| shape.coverage(x, y, self.feather));
        let ranges = self.ranges.iter().map(|range| range.coverage(rgb, self.feather));
        shapes.chain(ranges).product()
    }
}

impl Shape {
    fn polygon(points: Vec<[f32; 2]>) -> Self {
        let span = |axis: usize| points.iter().map(|p| p[axis]).fold(f32::INFINITY, f32::min)..points.iter().map(|p| p[axis]).fold(f32::NEG_INFINITY, f32::max);
        let (xs, ys) = (span(0), span(1));
        Self::Polygon { points, size: (xs.end - xs.start).min(ys.end - ys.start) / 2.0 }
    }

    fn coverage(&self, x: f32, y: f32, feather: f32) -> f32 {
        let (beyond, size) = match self {
            Self::Ellipse { centre, radius, aspect } => ((x - centre[0]).hypot((y - centre[1]) / aspect) - radius, *radius),
            Self::Polygon { points, size } => (if contains(points, x, y) { 0.0 } else { distance_to_outline(points, x, y) }, *size),
        };
        fade(beyond, size * feather)
    }
}

impl Range {
    fn coverage(&self, rgb: [f32; 3], feather: f32) -> f32 {
        let value = match self.channel {
            Channel::Luminance => crate::color::luminance(rgb),
            Channel::Hue => Hsv::from_rgb(rgb).hue,
            Channel::Saturation => Hsv::from_rgb(rgb).saturation,
        };
        let (beyond, width) = match self.channel {
            Channel::Hue => {
                let width = (self.high - self.low).rem_euclid(1.0);
                let offset = (value - self.low).rem_euclid(1.0);
                (if offset <= width { 0.0 } else { (offset - width).min(1.0 - offset) }, width)
            }
            _ => ((self.low - value).max(value - self.high).max(0.0), self.high - self.low),
        };
        fade(beyond, width * feather)
    }
}

fn fade(beyond: f32, ramp: f32) -> f32 {
    let t = (beyond / ramp.max(f32::EPSILON)).clamp(0.0, 1.0);
    1.0 - t * t * (3.0 - 2.0 * t)
}
