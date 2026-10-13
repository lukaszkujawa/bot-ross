pub mod tiles;

use crate::brush::bristles::Sample;
use crate::color::{OPAQUE, Premultiplied};
use crate::layer::tiles::{Bounds, Tiles};
use crate::mypaint::engine::Engine;
use crate::mypaint::ffi;
use crate::stroke::Point;

const SEGMENT_SECONDS: f64 = 0.02;

pub struct Layer {
    tiles: Tiles,
    pub width: usize,
    pub height: usize,
}

impl Layer {
    pub fn new(width: usize, height: usize) -> Self {
        let layer = Self { tiles: Tiles::new(width, height), width, height };
        layer.tiles.visit(layer.whole(), false, |_, _, channels| channels.fill(0));
        layer
    }

    pub fn trace(&mut self, engine: &mut Engine, samples: &[Sample]) -> Bounds {
        unsafe {
            ffi::mypaint_brush_reset(engine.raw());
            ffi::mypaint_brush_new_stroke(engine.raw());
            ffi::mypaint_surface_begin_atomic(self.tiles.interface());
        }
        for sample in samples {
            engine.set_color(sample.color);
            engine.set_radius(sample.radius);
            engine.set_opacity(sample.opacity);
            unsafe { self.stroke_to(engine, sample.point, sample.pressure) };
        }
        let mut touched = ffi::MyPaintRectangle::default();
        unsafe { ffi::mypaint_surface_end_atomic(self.tiles.interface(), &mut touched) };
        self.bounds_of(touched)
    }

    pub fn blend(&mut self, bounds: Bounds, paint: impl Fn(usize, usize) -> Premultiplied) -> Bounds {
        self.tiles.visit(bounds, false, |x, y, channels| {
            let pixel = paint(x, y);
            if pixel.alpha > 0 {
                channels.copy_from_slice(&pixel.over(clamped(channels)).to_channels());
            }
        });
        bounds
    }

    pub fn read_region(&self, bounds: Bounds) -> Vec<Premultiplied> {
        let mut pixels = Vec::with_capacity(bounds.area());
        self.tiles.visit(bounds, true, |_, _, channels| pixels.push(clamped(channels)));
        pixels
    }

    pub fn write_region(&mut self, bounds: Bounds, pixels: &[Premultiplied]) {
        let mut next = pixels.iter();
        self.tiles.visit(bounds, false, |_, _, channels| {
            channels.copy_from_slice(&next.next().expect("pixel for every position in bounds").to_channels());
        });
    }

    pub fn average(&self, centre: Point, extent: f32) -> Premultiplied {
        let (mut sum, mut count) = ([0u64; 4], 0u64);
        self.tiles.visit(Bounds::around(centre.x, centre.y, extent, self.width, self.height), true, |_, _, channels| {
            for (total, channel) in sum.iter_mut().zip(clamped(channels).to_channels()) {
                *total += channel as u64;
            }
            count += 1;
        });
        let mean = |total: u64| (total / count.max(1)) as u16;
        Premultiplied::new(mean(sum[0]), mean(sum[1]), mean(sum[2]), mean(sum[3]))
    }

    pub fn read(&self, bounds: Bounds, mut visit: impl FnMut(usize, usize, Premultiplied)) {
        self.tiles.visit(bounds, true, |x, y, channels| visit(x, y, clamped(channels)));
    }

    pub fn pixels(&self) -> Vec<Premultiplied> {
        let mut pixels = vec![Premultiplied::TRANSPARENT; self.width * self.height];
        self.tiles.visit(self.whole(), true, |x, y, channels| pixels[y * self.width + x] = clamped(channels));
        pixels
    }

    fn whole(&self) -> Bounds {
        Bounds::whole(self.width, self.height)
    }

    fn bounds_of(&self, touched: ffi::MyPaintRectangle) -> Bounds {
        if touched.width <= 0 || touched.height <= 0 {
            return Bounds::empty();
        }
        let (x, y) = (touched.x as f32, touched.y as f32);
        Bounds::enclosing([x, x + touched.width as f32], [y, y + touched.height as f32], 0.0, self.width, self.height)
    }

    unsafe fn stroke_to(&mut self, engine: &mut Engine, point: Point, pressure: f32) {
        unsafe {
            ffi::mypaint_brush_stroke_to(engine.raw(), self.tiles.interface(), point.x, point.y, pressure, 0.0, 0.0, SEGMENT_SECONDS)
        };
    }
}

fn clamped(channels: &[u16]) -> Premultiplied {
    let clamp = |index: usize| channels[index].min(OPAQUE);
    Premultiplied::new(clamp(0), clamp(1), clamp(2), clamp(3))
}
