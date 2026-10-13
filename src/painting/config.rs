use crate::painting::stroke_type::StrokeType;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Painting {
    pub canvas: CanvasConfig,
    pub search: SearchConfig,
    pub output: OutputConfig,
    pub brushes: BTreeMap<String, BrushRef>,
    pub strokes: BTreeMap<String, StrokeType>,
    pub regions: BTreeMap<String, Region>,
    pub layers: Vec<LayerConfig>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct CanvasConfig {
    pub background: String,
}

impl Default for CanvasConfig {
    fn default() -> Self {
        Self { background: "mean".into() }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct SearchConfig {
    pub candidates: usize,
    pub refinements: usize,
    pub patience: usize,
    pub seed: u64,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self { candidates: 16, refinements: 3, patience: 4, seed: 0 }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct OutputConfig {
    pub report: usize,
    pub snapshots: Option<PathBuf>,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self { report: 100, snapshots: None }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BrushRef {
    pub file: PathBuf,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Region {
    pub centre: [f32; 2],
    pub radius: f32,
    #[serde(default = "default_weight")]
    pub weight: f32,
    #[serde(default = "one")]
    pub outside: f32,
    #[serde(default = "half")]
    pub feather: f32,
}

impl Region {
    pub fn weight_at(&self, x: f32, y: f32) -> f32 {
        let beyond = (x - self.centre[0]).hypot(y - self.centre[1]) - self.radius;
        let t = (beyond / (self.radius * self.feather).max(f32::EPSILON)).clamp(0.0, 1.0);
        self.weight + (self.outside - self.weight) * t * t * (3.0 - 2.0 * t)
    }
}

fn default_weight() -> f32 {
    3.0
}

fn one() -> f32 {
    1.0
}

fn half() -> f32 {
    0.5
}

#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PlacementMode {
    #[default]
    Error,
    Edges,
    Random,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayerConfig {
    pub name: String,
    pub brush: Named<BrushRef>,
    pub stroke: Named<StrokeType>,
    pub count: usize,
    pub region: Option<Named<Region>>,
    #[serde(default)]
    pub placement: PlacementMode,
    pub candidates: Option<usize>,
    pub refinements: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Named<T> {
    Name(String),
    Inline(T),
}
