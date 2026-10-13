use crate::brush::mark::Mark;
use crate::layer::tiles::Bounds;
use crate::stroke::Stroke;
use serde::Deserialize;

const STEPS_PER_RADIUS: f32 = 2.0;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(default)]
pub struct Proxy {
    pub opacity: f32,
    pub width: f32,
}

impl Default for Proxy {
    fn default() -> Self {
        Self { opacity: 1.0, width: 1.0 }
    }
}

pub fn ribbon(stroke: &Stroke, proxy: Proxy, width: usize, height: usize) -> Mark {
    let [xs, ys] = stroke.path.extent();
    let mut mark = Mark::new(Bounds::enclosing(xs, ys, stroke.radius * proxy.width + 2.0, width, height));
    let steps = (stroke.path.length() / stroke.radius * STEPS_PER_RADIUS).ceil().max(1.0);
    let rgb = stroke.color.to_rgb();
    for step in 0..=steps as usize {
        let t = step as f32 / steps;
        let centre = stroke.path.at(t);
        let radius = stroke.radius * proxy.width * (0.5 + 0.5 * stroke.pressure.at(t));
        let bounds = Bounds::around(centre.x, centre.y, radius + 1.0, width, height);
        mark.add(bounds, rgb, |x, y| (radius + 0.5 - (x - centre.x).hypot(y - centre.y)).clamp(0.0, 1.0) * proxy.opacity * stroke.opacity);
    }
    mark
}
