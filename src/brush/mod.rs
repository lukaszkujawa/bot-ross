pub mod bristles;
pub mod config;
pub mod field;
pub mod line;
pub mod mark;
pub mod paint;
pub mod stamp;
pub mod tip;
mod walk;

use crate::brush::bristles::Bristles;
use crate::brush::config::BrushConfig;
use crate::brush::line::Line;
use crate::brush::paint::Paint;
use crate::brush::stamp::Stamp;
use crate::brush::tip::Tip;
use crate::layer::Layer;
use crate::layer::tiles::Bounds;
use crate::mypaint::engine::Engine;
use crate::proxy::Proxy;
use crate::stroke::Stroke;
use rand::SeedableRng;
use rand::rngs::StdRng;
use std::error::Error;
use std::path::Path;

pub struct Brush {
    pub name: String,
    tool: Tool,
    paint: Paint,
    proxy: Proxy,
}

enum Tool {
    Bristles { engine: Engine, bristles: Bristles },
    Stamp { tip: Tip, stamp: Stamp },
    Line(Line),
}

impl Brush {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Box<dyn Error>> {
        let path = path.as_ref();
        let base = path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
        Self::from_config(BrushConfig::load(path)?, base)
    }

    pub fn from_config(config: BrushConfig, base: &Path) -> Result<Self, Box<dyn Error>> {
        let tool = match (&config.bristles, &config.stamp, &config.line) {
            (Some(bristles), None, None) => Tool::Bristles { engine: config.engine()?, bristles: bristles.clone() },
            (None, Some(stamp), None) => Tool::Stamp { tip: Tip::load(base.join(&stamp.tip))?, stamp: stamp.clone() },
            (None, None, Some(line)) => Tool::Line(*line),
            _ => return Err(format!("brush '{}' needs exactly one of [bristles], [stamp] or [line]", config.name).into()),
        };
        let proxy = match &tool {
            Tool::Line(line) => line.proxy(),
            _ => config.proxy,
        };
        Ok(Self { name: config.name, tool, paint: config.paint, proxy })
    }

    pub fn proxy(&self) -> Proxy {
        self.proxy
    }

    pub fn paint(&mut self, layer: &mut Layer, stroke: &Stroke) -> Bounds {
        let mut rng = StdRng::seed_from_u64(stroke.seed);
        match &mut self.tool {
            Tool::Bristles { engine, bristles } => bristles
                .hairs(stroke, &self.paint, &mut rng)
                .iter()
                .fold(Bounds::empty(), |painted, hair| painted.union(layer.trace(engine, &hair.samples))),
            Tool::Stamp { tip, stamp } => stamp.paint(tip, layer, stroke, &self.paint, &mut rng),
            Tool::Line(line) => line.paint(layer, stroke),
        }
    }
}
