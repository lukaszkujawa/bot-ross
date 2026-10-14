use crate::brush::Brush;
use crate::canvas::Canvas;
use crate::color::{Blend, Hsv, Premultiplied};
use crate::image::Image;
use crate::painting::stroke_type::PressureShape;
use crate::stroke::{Curve, Point, Pressure, Stroke};

const DOT_LENGTH: f32 = 1.0;

#[derive(Clone, Copy, Debug)]
pub struct Settings {
    pub color: Hsv,
    pub radius: f32,
    pub opacity: f32,
    pub pressure: PressureShape,
}

pub struct Sketch {
    brush: Brush,
    canvas: Canvas,
    background: Premultiplied,
    blend: Blend,
    strokes: u64,
    pub settings: Settings,
}

impl Sketch {
    pub fn new(brush: Brush, width: usize, height: usize, background: Premultiplied, blend: Blend, settings: Settings) -> Self {
        let mut sketch = Self { brush, canvas: Canvas::new(width, height, background), background, blend, strokes: 0, settings };
        sketch.clear();
        sketch
    }

    pub fn clear(&mut self) {
        self.canvas = Canvas::new(self.canvas.width, self.canvas.height, self.background);
        self.canvas.add_layer(self.blend);
    }

    pub fn draw(&mut self, drag: &[Point]) -> bool {
        let Some(stroke) = stroke_from_drag(drag, &self.settings, self.strokes) else { return false };
        self.strokes += 1;
        let layer = self.canvas.top().expect("sketch keeps one layer");
        self.brush.paint(layer, &stroke);
        true
    }

    pub fn render(&self) -> Image {
        self.canvas.render()
    }
}

pub fn stroke_from_drag(drag: &[Point], settings: &Settings, seed: u64) -> Option<Stroke> {
    let path = fit_curve(drag)?;
    let pressure = match settings.pressure {
        PressureShape::Flat => Pressure::constant(1.0),
        shape => shape.profile(),
    };
    Some(Stroke { path, pressure, color: settings.color, radius: settings.radius, opacity: settings.opacity, seed })
}

pub fn fit_curve(drag: &[Point]) -> Option<Curve> {
    let first = *drag.first()?;
    let lengths: Vec<f32> = drag.iter().scan((0.0, first), |(total, previous), point| {
        *total += previous.distance(*point);
        *previous = *point;
        Some(*total)
    }).collect();
    let total = *lengths.last()?;
    if total < DOT_LENGTH {
        return Some(Curve::line(first, Point { x: first.x + DOT_LENGTH, y: first.y }));
    }
    let along = |fraction: f32| {
        let wanted = total * fraction;
        let index = lengths.iter().position(|length| *length >= wanted).unwrap_or(drag.len() - 1).max(1);
        let span = (lengths[index] - lengths[index - 1]).max(f32::EPSILON);
        drag[index - 1].lerp(drag[index], ((wanted - lengths[index - 1]) / span).clamp(0.0, 1.0))
    };
    let (p0, p1, p2, p3) = (first, along(1.0 / 3.0), along(2.0 / 3.0), *drag.last()?);
    let blend = |a: f32, b: f32, c: f32, d: f32| Point { x: (a * p0.x + b * p1.x + c * p2.x + d * p3.x) / 6.0, y: (a * p0.y + b * p1.y + c * p2.y + d * p3.y) / 6.0 };
    Some(Curve { from: p0, control: [blend(-5.0, 18.0, -9.0, 2.0), blend(2.0, -9.0, 18.0, -5.0)], to: p3 })
}

pub fn overlay(image: &mut Image, drag: &[Point], argb: u32) {
    for pair in drag.windows(2) {
        let steps = pair[0].distance(pair[1]).ceil().max(1.0) as usize;
        for step in 0..=steps {
            let point = pair[0].lerp(pair[1], step as f32 / steps as f32);
            let (x, y) = (point.x as usize, point.y as usize);
            if x < image.width && y < image.height {
                image.argb[y * image.width + x] = argb;
            }
        }
    }
}
