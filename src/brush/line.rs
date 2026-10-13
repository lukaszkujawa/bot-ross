use crate::layer::Layer;
use crate::bounds::Bounds;
use crate::proxy::{Proxy, ribbon};
use crate::stroke::Stroke;
use serde::Deserialize;

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Line {
    pub opacity: f32,
}

impl Default for Line {
    fn default() -> Self {
        Self { opacity: 1.0 }
    }
}

impl Line {
    pub fn proxy(&self) -> Proxy {
        Proxy { opacity: self.opacity, width: 1.0 }
    }

    pub fn paint(&self, layer: &mut Layer, stroke: &Stroke) -> Bounds {
        ribbon(stroke, self.proxy(), layer.width, layer.height).composite(layer)
    }
}
