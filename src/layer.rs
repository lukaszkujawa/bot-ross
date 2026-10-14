use crate::bounds::Bounds;
use crate::color::{Blend, OPAQUE, Premultiplied};
use crate::mypaint::engine::{Engine, Sample};
use crate::mypaint::ffi::MyPaintRectangle;
use crate::mypaint::surface::Surface;
use crate::stroke::Point;

pub struct Layer {
    surface: Surface,
    blend: Blend,
    pub width: usize,
    pub height: usize,
}

impl Layer {
    pub fn new(width: usize, height: usize) -> Self {
        Self::blended(width, height, Blend::Normal)
    }

    pub fn blended(width: usize, height: usize, blend: Blend) -> Self {
        let layer = Self { surface: Surface::new(width, height), blend, width, height };
        layer.surface.visit_mut(layer.whole(), |_, _, channels| channels.fill(0));
        layer
    }

    pub fn mode(&self) -> Blend {
        self.blend
    }

    pub fn trace(&mut self, engine: &mut Engine, samples: &[Sample]) -> Bounds {
        self.bounds_of(engine.stroke(&self.surface, samples))
    }

    pub fn blend(&mut self, bounds: Bounds, paint: impl Fn(usize, usize) -> Premultiplied) -> Bounds {
        self.surface.visit_mut(bounds, |x, y, channels| {
            let pixel = paint(x, y);
            if pixel.alpha > 0 {
                channels.copy_from_slice(&self.blend.apply(pixel, clamped(channels)).to_channels());
            }
        });
        bounds
    }

    pub fn read_region(&self, bounds: Bounds) -> Vec<Premultiplied> {
        let mut pixels = Vec::with_capacity(bounds.area());
        self.surface.visit(bounds, |_, _, channels| pixels.push(clamped(channels)));
        pixels
    }

    pub fn write_region(&mut self, bounds: Bounds, pixels: &[Premultiplied]) {
        let mut next = pixels.iter();
        self.surface.visit_mut(bounds, |_, _, channels| {
            channels.copy_from_slice(&next.next().expect("pixel for every position in bounds").to_channels());
        });
    }

    pub fn average(&self, centre: Point, extent: f32) -> Premultiplied {
        let (mut sum, mut count) = ([0u64; 4], 0u64);
        self.surface.visit(Bounds::around(centre.x, centre.y, extent, self.width, self.height), |_, _, channels| {
            for (total, channel) in sum.iter_mut().zip(clamped(channels).to_channels()) {
                *total += channel as u64;
            }
            count += 1;
        });
        let mean = |total: u64| (total / count.max(1)) as u16;
        Premultiplied::new(mean(sum[0]), mean(sum[1]), mean(sum[2]), mean(sum[3]))
    }

    pub fn read(&self, bounds: Bounds, mut visit: impl FnMut(usize, usize, Premultiplied)) {
        self.surface.visit(bounds, |x, y, channels| visit(x, y, clamped(channels)));
    }

    pub fn pixels(&self) -> Vec<Premultiplied> {
        let mut pixels = vec![Premultiplied::TRANSPARENT; self.width * self.height];
        self.surface.visit(self.whole(), |x, y, channels| pixels[y * self.width + x] = clamped(channels));
        pixels
    }

    fn whole(&self) -> Bounds {
        Bounds::whole(self.width, self.height)
    }

    fn bounds_of(&self, touched: MyPaintRectangle) -> Bounds {
        if touched.width <= 0 || touched.height <= 0 {
            return Bounds::empty();
        }
        let (x, y) = (touched.x as f32, touched.y as f32);
        Bounds::enclosing([x, x + touched.width as f32], [y, y + touched.height as f32], 0.0, self.width, self.height)
    }
}

fn clamped(channels: &[u16]) -> Premultiplied {
    let clamp = |index: usize| channels[index].min(OPAQUE);
    Premultiplied::new(clamp(0), clamp(1), clamp(2), clamp(3))
}
