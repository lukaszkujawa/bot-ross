use crate::reference::Reference;
use crate::stroke::Point;
use std::f32::consts::{FRAC_PI_2, PI};

const SMOOTHING: usize = 3;
const STRONG_EDGE_PERCENTILE: f32 = 0.9;

pub struct Flow {
    width: usize,
    height: usize,
    tangent: Vec<f32>,
    strength: Vec<f32>,
}

impl Flow {
    pub fn of(reference: &Reference) -> Self {
        let (width, height) = (reference.width, reference.height);
        let luminance = |x: usize, y: usize| reference.luminance(x, y);
        let tensor: Vec<[f32; 3]> = (0..width * height)
            .map(|i| {
                let (x, y) = (i % width, i / width);
                let gx = luminance(x + 1, y) - luminance(x.saturating_sub(1), y);
                let gy = luminance(x, y + 1) - luminance(x, y.saturating_sub(1));
                [gx * gx, gy * gy, gx * gy]
            })
            .collect();
        let smoothed = box_average(&tensor, width, height, SMOOTHING);
        let magnitudes: Vec<f32> = smoothed.iter().map(|[xx, yy, _]| (xx + yy).sqrt()).collect();
        let mut ranked = magnitudes.clone();
        ranked.sort_by(|a, b| a.total_cmp(b));
        let strong = ranked[((ranked.len() - 1) as f32 * STRONG_EDGE_PERCENTILE) as usize].max(f32::EPSILON);
        let tangent = smoothed.iter().map(|[xx, yy, xy]| 0.5 * (2.0 * xy).atan2(xx - yy) + FRAC_PI_2).collect();
        let strength = magnitudes.iter().map(|magnitude| (magnitude / strong).min(1.0)).collect();
        Self { width, height, tangent, strength }
    }

    pub fn flat(width: usize, height: usize) -> Self {
        Self { width, height, tangent: vec![0.0; width * height], strength: vec![0.0; width * height] }
    }

    pub fn radial(width: usize, height: usize, centre: [f32; 2]) -> Self {
        let tangent = (0..width * height).map(|i| (centre[1] - (i / width) as f32).atan2(centre[0] - (i % width) as f32)).collect();
        Self { width, height, tangent, strength: vec![1.0; width * height] }
    }

    pub fn at(&self, point: Point) -> (f32, f32) {
        let x = (point.x.max(0.0) as usize).min(self.width - 1);
        let y = (point.y.max(0.0) as usize).min(self.height - 1);
        (self.tangent[y * self.width + x], self.strength[y * self.width + x])
    }

    pub fn aligned(&self, point: Point, direction: f32) -> (f32, f32) {
        let (tangent, strength) = self.at(point);
        let flipped = tangent + PI;
        let closeness = |angle: f32| ((angle - direction + PI).rem_euclid(2.0 * PI) - PI).abs();
        (if closeness(flipped) < closeness(tangent) { flipped } else { tangent }, strength)
    }
}

fn box_average(values: &[[f32; 3]], width: usize, height: usize, radius: usize) -> Vec<[f32; 3]> {
    let pass = |source: &[[f32; 3]], len: usize, other: usize, index: &dyn Fn(usize, usize) -> usize| {
        let mut out = vec![[0.0; 3]; source.len()];
        for o in 0..other {
            for l in 0..len {
                let mut sum = [0.0; 3];
                let mut count = 0.0;
                for k in l.saturating_sub(radius)..(l + radius + 1).min(len) {
                    let v = source[index(k, o)];
                    sum = [sum[0] + v[0], sum[1] + v[1], sum[2] + v[2]];
                    count += 1.0;
                }
                out[index(l, o)] = sum.map(|s| s / count);
            }
        }
        out
    };
    let rows = pass(values, width, height, &|x, y| y * width + x);
    pass(&rows, height, width, &|y, x| y * width + x)
}
