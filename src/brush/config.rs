use crate::brush::bristles::Bristles;
use crate::brush::line::Line;
use crate::brush::paint::Paint;
use crate::brush::stamp::Stamp;
use crate::brush::wash::Wash;
use crate::mypaint::engine::Engine;
use crate::proxy::Proxy;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct BrushConfig {
    pub name: String,
    pub template: Option<String>,
    #[serde(default)]
    pub settings: BTreeMap<String, f32>,
    #[serde(default)]
    pub dynamics: BTreeMap<String, BTreeMap<String, Vec<[f32; 2]>>>,
    pub bristles: Option<Bristles>,
    pub stamp: Option<Stamp>,
    pub line: Option<Line>,
    pub wash: Option<Wash>,
    #[serde(default)]
    pub paint: Paint,
    #[serde(default)]
    pub proxy: Proxy,
}

impl BrushConfig {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Box<dyn Error>> {
        Self::parse(&fs::read_to_string(path)?)
    }

    pub fn parse(text: &str) -> Result<Self, Box<dyn Error>> {
        Ok(toml::from_str(text)?)
    }

    pub fn engine(&self) -> Result<Engine, Box<dyn Error>> {
        let mut engine = match &self.template {
            Some(name) => Engine::named(name)?,
            None => Engine::with_defaults(),
        };
        for (setting, value) in &self.settings {
            engine.set(setting, *value)?;
        }
        for (setting, curves) in &self.dynamics {
            for (input, points) in curves {
                engine.set_curve(setting, input, points)?;
            }
        }
        Ok(engine)
    }
}
