use crate::color::Premultiplied;
use crate::layer::Layer;
use crate::layer::tiles::Bounds;

pub struct Mark {
    bounds: Bounds,
    coverage: Vec<f32>,
    rgb: Vec<[f32; 3]>,
}

impl Mark {
    pub fn new(bounds: Bounds) -> Self {
        let area = (bounds.right - bounds.left) * (bounds.bottom - bounds.top);
        Self { bounds, coverage: vec![0.0; area], rgb: vec![[0.0; 3]; area] }
    }

    pub fn add(&mut self, bounds: Bounds, rgb: [f32; 3], coverage: impl Fn(f32, f32) -> f32) {
        for y in bounds.top.max(self.bounds.top)..bounds.bottom.min(self.bounds.bottom) {
            for x in bounds.left.max(self.bounds.left)..bounds.right.min(self.bounds.right) {
                let alpha = coverage(x as f32 + 0.5, y as f32 + 0.5);
                let index = self.index(x, y);
                if alpha > self.coverage[index] {
                    self.coverage[index] = alpha;
                    self.rgb[index] = rgb;
                }
            }
        }
    }

    pub fn bounds(&self) -> Bounds {
        self.bounds
    }

    pub fn coverage_at(&self, x: usize, y: usize) -> f32 {
        let inside = (self.bounds.left..self.bounds.right).contains(&x) && (self.bounds.top..self.bounds.bottom).contains(&y);
        if inside { self.coverage[self.index(x, y)] } else { 0.0 }
    }

    pub fn composite(&self, layer: &mut Layer) -> Bounds {
        layer.blend(self.bounds, |x, y| {
            let index = self.index(x, y);
            Premultiplied::from_rgb(self.rgb[index], self.coverage[index])
        })
    }

    fn index(&self, x: usize, y: usize) -> usize {
        (y - self.bounds.top) * (self.bounds.right - self.bounds.left) + (x - self.bounds.left)
    }
}
