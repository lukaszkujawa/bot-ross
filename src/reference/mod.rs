mod filters;

use crate::color::{OPAQUE, Premultiplied, luminance};
use crate::stroke::{Curve, Point};
use image::imageops::FilterType;
use std::error::Error;
use std::path::Path;

const CURVE_SAMPLES: usize = 5;

#[derive(Clone)]
pub struct Reference {
    pub width: usize,
    pub height: usize,
    rgb: Vec<[u16; 3]>,
}

impl Reference {
    pub fn load(path: impl AsRef<Path>, longest_side: u32) -> Result<Self, Box<dyn Error>> {
        let image = image::open(path)?;
        let scale = longest_side as f32 / image.width().max(image.height()) as f32;
        let side = |pixels: u32| ((pixels as f32 * scale).round() as u32).max(1);
        let resized = image.resize_exact(side(image.width()), side(image.height()), FilterType::Lanczos3).to_rgb8();
        Ok(Self::from_rgb8(resized.width() as usize, resized.height() as usize, resized.as_raw()))
    }

    pub fn from_rgb8(width: usize, height: usize, bytes: &[u8]) -> Self {
        let fix15 = |byte: u8| (byte as u32 * OPAQUE as u32 / 255) as u16;
        Self::from_pixels(width, height, bytes.chunks_exact(3).map(|c| [fix15(c[0]), fix15(c[1]), fix15(c[2])]).collect())
    }

    pub fn from_pixels(width: usize, height: usize, rgb: Vec<[u16; 3]>) -> Self {
        assert_eq!(rgb.len(), width * height, "reference pixel count");
        Self { width, height, rgb }
    }

    pub fn glaze_over(&self, below: &[Premultiplied]) -> Self {
        let needed = |target: u16, under: u16| if under == 0 { OPAQUE } else { ((target as u32 * OPAQUE as u32 / under as u32).min(OPAQUE as u32)) as u16 };
        let rgb = self.rgb.iter().zip(below).map(|(target, under)| [needed(target[0], under.red), needed(target[1], under.green), needed(target[2], under.blue)]).collect();
        Self::from_pixels(self.width, self.height, rgb)
    }

    pub fn pixels(&self) -> &[[u16; 3]] {
        &self.rgb
    }

    pub fn pixel(&self, x: usize, y: usize) -> [u16; 3] {
        self.rgb[y.min(self.height - 1) * self.width + x.min(self.width - 1)]
    }

    pub fn luminance(&self, x: usize, y: usize) -> f32 {
        luminance(self.pixel(x, y).map(|channel| channel as f32))
    }

    pub fn sample(&self, point: Point) -> [u16; 3] {
        self.pixel(point.x.max(0.0) as usize, point.y.max(0.0) as usize)
    }

    pub fn mean_along(&self, curve: &Curve) -> [f32; 3] {
        self.mean_at((0..CURVE_SAMPLES).map(|i| curve.at(i as f32 / (CURVE_SAMPLES - 1) as f32)))
    }

    pub fn mean_at(&self, points: impl ExactSizeIterator<Item = Point>) -> [f32; 3] {
        let count = points.len();
        mean(points.map(|point| self.sample(point)), count)
    }

    pub fn mean(&self) -> [f32; 3] {
        mean(self.rgb.iter().copied(), self.rgb.len())
    }

    pub fn edges(&self) -> Vec<f32> {
        let mut strength: Vec<f32> = (0..self.width * self.height)
            .map(|i| {
                let (x, y) = (i % self.width, i / self.width);
                let dx = self.luminance(x + 1, y) - self.luminance(x.saturating_sub(1), y);
                let dy = self.luminance(x, y + 1) - self.luminance(x, y.saturating_sub(1));
                dx.hypot(dy)
            })
            .collect();
        let peak = strength.iter().cloned().fold(0.0, f32::max).max(f32::EPSILON);
        strength.iter_mut().for_each(|s| *s /= peak);
        strength
    }
}

fn mean(pixels: impl Iterator<Item = [u16; 3]>, count: usize) -> [f32; 3] {
    let mut sum = [0.0f32; 3];
    for pixel in pixels {
        for (total, channel) in sum.iter_mut().zip(pixel) {
            *total += channel as f32 / OPAQUE as f32;
        }
    }
    sum.map(|total| total / count.max(1) as f32)
}
