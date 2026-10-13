use resvg::{tiny_skia, usvg};
use std::error::Error;
use std::fs;
use std::path::Path;

const RASTER_SIZE: f32 = 512.0;

pub struct Tip {
    width: usize,
    height: usize,
    alpha: Vec<f32>,
}

impl Tip {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Box<dyn Error>> {
        let tree = usvg::Tree::from_data(&fs::read(path)?, &usvg::Options::default())?;
        let size = tree.size();
        let scale = RASTER_SIZE / size.width().max(size.height());
        let (width, height) = ((size.width() * scale).ceil() as u32, (size.height() * scale).ceil() as u32);
        let mut pixmap = tiny_skia::Pixmap::new(width, height).ok_or("empty brush tip")?;
        resvg::render(&tree, tiny_skia::Transform::from_scale(scale, scale), &mut pixmap.as_mut());
        let alpha = pixmap.pixels().iter().map(|pixel| pixel.alpha() as f32 / 255.0).collect();
        Ok(Self { width: width as usize, height: height as usize, alpha })
    }

    pub fn aspect(&self) -> f32 {
        self.width as f32 / self.height as f32
    }

    pub fn coverage(&self, u: f32, v: f32) -> f32 {
        if !(0.0..1.0).contains(&u) || !(0.0..1.0).contains(&v) {
            return 0.0;
        }
        let (x, y) = (u * self.width as f32 - 0.5, v * self.height as f32 - 0.5);
        let (x0, y0) = (x.floor(), y.floor());
        let (fx, fy) = (x - x0, y - y0);
        let top = self.at(x0, y0) * (1.0 - fx) + self.at(x0 + 1.0, y0) * fx;
        let bottom = self.at(x0, y0 + 1.0) * (1.0 - fx) + self.at(x0 + 1.0, y0 + 1.0) * fx;
        top * (1.0 - fy) + bottom * fy
    }

    fn at(&self, x: f32, y: f32) -> f32 {
        if x < 0.0 || y < 0.0 || x >= self.width as f32 || y >= self.height as f32 {
            return 0.0;
        }
        self.alpha[y as usize * self.width + x as usize]
    }
}
