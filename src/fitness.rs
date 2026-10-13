use crate::mark::Mark;
use crate::color::Premultiplied;
use crate::image::Image;
use crate::layer::Layer;
use crate::bounds::Bounds;
use crate::reference::Reference;

const CHANNEL_WEIGHTS: [u32; 3] = [3, 4, 2];

pub struct Fitness {
    pub width: usize,
    pub height: usize,
    reference: Vec<[u16; 3]>,
    below: Vec<Premultiplied>,
    current: Vec<Premultiplied>,
    composite: Vec<Premultiplied>,
    error: Vec<u32>,
    weight: Vec<f32>,
    total: u64,
}

impl Fitness {
    pub fn new(reference: &Reference, below: Vec<Premultiplied>) -> Self {
        let area = reference.width * reference.height;
        assert_eq!(below.len(), area, "layer below must match the reference size");
        let mut fitness = Self {
            width: reference.width,
            height: reference.height,
            reference: reference.pixels().to_vec(),
            composite: below.clone(),
            below,
            current: vec![Premultiplied::TRANSPARENT; area],
            error: vec![0; area],
            weight: vec![1.0; area],
            total: 0,
        };
        fitness.recompute(Bounds::whole(fitness.width, fitness.height));
        fitness
    }

    pub fn weigh(&mut self, weight: impl Fn(usize, usize) -> f32) {
        for (index, slot) in self.weight.iter_mut().enumerate() {
            *slot = weight(index % self.width, index / self.width);
        }
        self.recompute(Bounds::whole(self.width, self.height));
    }

    pub fn total(&self) -> u64 {
        self.total
    }

    pub fn error_at(&self, x: usize, y: usize) -> u32 {
        self.error[y * self.width + x]
    }

    pub fn delta(&self, mark: &Mark, rgb: [f32; 3]) -> i64 {
        let bounds = mark.bounds();
        let (mut before, mut after) = (0u64, 0u64);
        for y in bounds.top..bounds.bottom {
            for x in bounds.left..bounds.right {
                let coverage = mark.coverage_at(x, y);
                if coverage <= 0.0 {
                    continue;
                }
                let index = y * self.width + x;
                let painted = Premultiplied::from_rgb(rgb, coverage).over(self.current[index]).over(self.below[index]);
                after += pixel_error(painted, self.reference[index], self.weight[index]) as u64;
                before += self.error[index] as u64;
            }
        }
        after as i64 - before as i64
    }

    pub fn refresh(&mut self, layer: &Layer, bounds: Bounds) {
        layer.read(bounds, |x, y, pixel| self.current[y * self.width + x] = pixel);
        self.recompute(bounds);
    }

    pub fn image(&self) -> Image {
        Image { width: self.width, height: self.height, argb: self.composite.iter().map(|pixel| pixel.to_argb()).collect() }
    }

    fn recompute(&mut self, bounds: Bounds) {
        for y in bounds.top..bounds.bottom {
            for x in bounds.left..bounds.right {
                let index = y * self.width + x;
                self.composite[index] = self.current[index].over(self.below[index]);
                let error = pixel_error(self.composite[index], self.reference[index], self.weight[index]);
                self.total = self.total + error as u64 - self.error[index] as u64;
                self.error[index] = error;
            }
        }
    }
}

fn pixel_error(painted: Premultiplied, target: [u16; 3], weight: f32) -> u32 {
    let channels = [painted.red, painted.green, painted.blue];
    let raw: u32 = (0..3).map(|c| (channels[c] as i32 - target[c] as i32).unsigned_abs() * CHANNEL_WEIGHTS[c]).sum();
    (raw as f32 * weight) as u32
}
