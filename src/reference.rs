use crate::color::OPAQUE;
use crate::stroke::{Curve, Point};
use image::imageops::FilterType;
use std::error::Error;
use std::path::Path;

const CURVE_SAMPLES: usize = 5;
const SMOOTH_PASSES: usize = 3;
const SMOOTH_RANGE: f32 = 0.1;

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

    pub fn pixels(&self) -> &[[u16; 3]] {
        &self.rgb
    }

    pub fn pixel(&self, x: usize, y: usize) -> [u16; 3] {
        self.rgb[y.min(self.height - 1) * self.width + x.min(self.width - 1)]
    }

    pub fn sample(&self, point: Point) -> [u16; 3] {
        self.pixel(point.x.max(0.0) as usize, point.y.max(0.0) as usize)
    }

    pub fn mean_along(&self, curve: &Curve) -> [f32; 3] {
        let samples = (0..CURVE_SAMPLES).map(|i| self.sample(curve.at(i as f32 / (CURVE_SAMPLES - 1) as f32)));
        mean(samples, CURVE_SAMPLES)
    }

    pub fn mean(&self) -> [f32; 3] {
        mean(self.rgb.iter().copied(), self.rgb.len())
    }

    pub fn blurred(&self, sigma: f32) -> Self {
        if sigma <= 0.0 {
            return Self::from_pixels(self.width, self.height, self.rgb.clone());
        }
        let reach = (sigma * 3.0).ceil() as usize;
        let kernel: Vec<f32> = (0..=2 * reach).map(|i| (-((i as f32 - reach as f32).powi(2)) / (2.0 * sigma * sigma)).exp()).collect();
        let total: f32 = kernel.iter().sum();
        let rows = smear(&self.rgb, self.width, self.height, &kernel, total, |x, y, k| (x as isize + k, y as isize));
        let both = smear(&rows, self.width, self.height, &kernel, total, |x, y, k| (x as isize, y as isize + k));
        Self::from_pixels(self.width, self.height, both)
    }

    pub fn smoothed(&self, sigma: f32) -> Self {
        if sigma <= 0.0 {
            return Self::from_pixels(self.width, self.height, self.rgb.clone());
        }
        let reach = (sigma * 2.0).ceil() as usize;
        let kernel: Vec<f32> = (0..=2 * reach).map(|i| (-((i as f32 - reach as f32).powi(2)) / (2.0 * sigma * sigma)).exp()).collect();
        let mut rgb = self.rgb.clone();
        for _ in 0..SMOOTH_PASSES {
            let rows = keep_edges(&rgb, self.width, self.height, &kernel, |x, y, k| (x as isize + k, y as isize));
            rgb = keep_edges(&rows, self.width, self.height, &kernel, |x, y, k| (x as isize, y as isize + k));
        }
        Self::from_pixels(self.width, self.height, rgb)
    }

    pub fn graded(&self, contrast: f32, saturation: f32, levels: u8) -> Self {
        let unit = OPAQUE as f32;
        let rgb = self
            .rgb
            .iter()
            .map(|pixel| {
                let [r, g, b] = pixel.map(|c| c as f32 / unit);
                let grey = r * 0.3 + g * 0.5 + b * 0.2;
                [r, g, b].map(|c| {
                    let saturated = grey + (c - grey) * saturation;
                    let contrasted = 0.5 + (saturated - 0.5) * contrast;
                    let stepped = if levels >= 2 { (contrasted * (levels - 1) as f32).round() / (levels - 1) as f32 } else { contrasted };
                    (stepped.clamp(0.0, 1.0) * unit).round() as u16
                })
            })
            .collect();
        Self::from_pixels(self.width, self.height, rgb)
    }

    pub fn edges(&self) -> Vec<f32> {
        let luminance = |x: usize, y: usize| {
            let [r, g, b] = self.pixel(x, y);
            r as f32 * 0.3 + g as f32 * 0.5 + b as f32 * 0.2
        };
        let mut strength: Vec<f32> = (0..self.width * self.height)
            .map(|i| {
                let (x, y) = (i % self.width, i / self.width);
                let dx = luminance(x + 1, y) - luminance(x.saturating_sub(1), y);
                let dy = luminance(x, y + 1) - luminance(x, y.saturating_sub(1));
                dx.hypot(dy)
            })
            .collect();
        let peak = strength.iter().cloned().fold(0.0, f32::max).max(f32::EPSILON);
        strength.iter_mut().for_each(|s| *s /= peak);
        strength
    }
}

fn smear(pixels: &[[u16; 3]], width: usize, height: usize, kernel: &[f32], total: f32, tap: impl Fn(usize, usize, isize) -> (isize, isize)) -> Vec<[u16; 3]> {
    let reach = (kernel.len() / 2) as isize;
    (0..width * height)
        .map(|index| {
            let (x, y) = (index % width, index / width);
            let mut sum = [0.0f32; 3];
            for (offset, weight) in kernel.iter().enumerate() {
                let (sx, sy) = tap(x, y, offset as isize - reach);
                let (sx, sy) = (sx.clamp(0, width as isize - 1) as usize, sy.clamp(0, height as isize - 1) as usize);
                for (channel, value) in sum.iter_mut().zip(pixels[sy * width + sx]) {
                    *channel += value as f32 * weight;
                }
            }
            sum.map(|channel| (channel / total).round() as u16)
        })
        .collect()
}

fn keep_edges(pixels: &[[u16; 3]], width: usize, height: usize, kernel: &[f32], tap: impl Fn(usize, usize, isize) -> (isize, isize)) -> Vec<[u16; 3]> {
    let reach = (kernel.len() / 2) as isize;
    let range = SMOOTH_RANGE * OPAQUE as f32;
    (0..width * height)
        .map(|index| {
            let (x, y) = (index % width, index / width);
            let centre = pixels[index];
            let (mut sum, mut total) = ([0.0f32; 3], 0.0f32);
            for (offset, weight) in kernel.iter().enumerate() {
                let (sx, sy) = tap(x, y, offset as isize - reach);
                let (sx, sy) = (sx.clamp(0, width as isize - 1) as usize, sy.clamp(0, height as isize - 1) as usize);
                let sample = pixels[sy * width + sx];
                let difference: f32 = (0..3).map(|c| (sample[c] as f32 - centre[c] as f32).powi(2)).sum::<f32>().sqrt();
                let closeness = weight * (-(difference * difference) / (2.0 * range * range)).exp();
                for (channel, value) in sum.iter_mut().zip(sample) {
                    *channel += value as f32 * closeness;
                }
                total += closeness;
            }
            sum.map(|channel| (channel / total).round() as u16)
        })
        .collect()
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
