use crate::color::{OPAQUE, luminance};
use crate::reference::Reference;
use rayon::prelude::*;

const SMOOTH_PASSES: usize = 3;
const SMOOTH_RANGE: f32 = 0.1;
const BLUR_REACH_SIGMAS: f32 = 3.0;
const SMOOTH_REACH_SIGMAS: f32 = 2.0;

type Tap = fn(usize, usize, isize) -> (isize, isize);

const ALONG_ROWS: Tap = |x, y, k| (x as isize + k, y as isize);
const ALONG_COLUMNS: Tap = |x, y, k| (x as isize, y as isize + k);

impl Reference {
    pub fn blurred(&self, sigma: f32) -> Self {
        if sigma <= 0.0 {
            return self.clone();
        }
        let kernel = gaussian(sigma, BLUR_REACH_SIGMAS);
        let rows = smear(&self.rgb, self.width, self.height, &kernel, ALONG_ROWS);
        let both = smear(&rows, self.width, self.height, &kernel, ALONG_COLUMNS);
        Self::from_pixels(self.width, self.height, both)
    }

    pub fn smoothed(&self, sigma: f32) -> Self {
        if sigma <= 0.0 {
            return self.clone();
        }
        let kernel = gaussian(sigma, SMOOTH_REACH_SIGMAS);
        let mut rgb = self.rgb.clone();
        for _ in 0..SMOOTH_PASSES {
            let rows = keep_edges(&rgb, self.width, self.height, &kernel, ALONG_ROWS);
            rgb = keep_edges(&rows, self.width, self.height, &kernel, ALONG_COLUMNS);
        }
        Self::from_pixels(self.width, self.height, rgb)
    }

    pub fn graded(&self, contrast: f32, saturation: f32, gamma: f32, levels: u8) -> Self {
        let unit = OPAQUE as f32;
        let rgb = self
            .rgb
            .iter()
            .map(|pixel| {
                let rgb = pixel.map(|c| c as f32 / unit);
                let grey = luminance(rgb);
                rgb.map(|c| {
                    let saturated = grey + (c - grey) * saturation;
                    let contrasted = 0.5 + (saturated - 0.5) * contrast;
                    let lifted = contrasted.clamp(0.0, 1.0).powf(1.0 / gamma);
                    let stepped = if levels >= 2 { (lifted * (levels - 1) as f32).round() / (levels - 1) as f32 } else { lifted };
                    (stepped.clamp(0.0, 1.0) * unit).round() as u16
                })
            })
            .collect();
        Self::from_pixels(self.width, self.height, rgb)
    }
}

fn gaussian(sigma: f32, reach_sigmas: f32) -> Vec<f32> {
    let reach = (sigma * reach_sigmas).ceil() as usize;
    (0..=2 * reach).map(|i| (-((i as f32 - reach as f32).powi(2)) / (2.0 * sigma * sigma)).exp()).collect()
}

fn tapped(tap: Tap, x: usize, y: usize, offset: usize, kernel_len: usize, width: usize, height: usize) -> usize {
    let (sx, sy) = tap(x, y, offset as isize - (kernel_len / 2) as isize);
    sy.clamp(0, height as isize - 1) as usize * width + sx.clamp(0, width as isize - 1) as usize
}

fn smear(pixels: &[[u16; 3]], width: usize, height: usize, kernel: &[f32], tap: Tap) -> Vec<[u16; 3]> {
    let total: f32 = kernel.iter().sum();
    (0..width * height)
        .into_par_iter()
        .map(|index| {
            let (x, y) = (index % width, index / width);
            let mut sum = [0.0f32; 3];
            for (offset, weight) in kernel.iter().enumerate() {
                for (channel, value) in sum.iter_mut().zip(pixels[tapped(tap, x, y, offset, kernel.len(), width, height)]) {
                    *channel += value as f32 * weight;
                }
            }
            sum.map(|channel| (channel / total).round() as u16)
        })
        .collect()
}

fn keep_edges(pixels: &[[u16; 3]], width: usize, height: usize, kernel: &[f32], tap: Tap) -> Vec<[u16; 3]> {
    let range = SMOOTH_RANGE * OPAQUE as f32;
    (0..width * height)
        .into_par_iter()
        .map(|index| {
            let (x, y) = (index % width, index / width);
            let centre = pixels[index];
            let (mut sum, mut total) = ([0.0f32; 3], 0.0f32);
            for (offset, weight) in kernel.iter().enumerate() {
                let sample = pixels[tapped(tap, x, y, offset, kernel.len(), width, height)];
                let distance: f32 = (0..3).map(|c| (sample[c] as f32 - centre[c] as f32).powi(2)).sum();
                let closeness = weight * (-distance / (2.0 * range * range)).exp();
                for (channel, value) in sum.iter_mut().zip(sample) {
                    *channel += value as f32 * closeness;
                }
                total += closeness;
            }
            sum.map(|channel| (channel / total).round() as u16)
        })
        .collect()
}
