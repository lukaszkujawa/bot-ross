use crate::color::Blend;
use crate::painting::region::Region;
use crate::painting::shape_type::ShapeType;
use crate::painting::stroke_type::StrokeType;
use crate::reference::Reference;
use serde::Deserialize;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct PaintingConfig {
    pub canvas: CanvasConfig,
    pub search: SearchConfig,
    pub output: OutputConfig,
    pub brushes: BTreeMap<String, BrushRef>,
    pub strokes: BTreeMap<String, StrokeType>,
    pub shapes: BTreeMap<String, ShapeType>,
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

#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PlacementMode {
    #[default]
    Error,
    Edges,
    Random,
    Fill,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
#[serde(deny_unknown_fields, default)]
pub struct Target {
    pub blur: f32,
    pub smooth: f32,
    pub edges: f32,
    pub levels: u8,
    pub contrast: f32,
    pub saturation: f32,
    pub gamma: f32,
}

impl Default for Target {
    fn default() -> Self {
        Self { blur: 0.0, smooth: 0.0, edges: 0.0, levels: 0, contrast: 1.0, saturation: 1.0, gamma: 1.0 }
    }
}

impl Target {
    pub fn grades(&self) -> bool {
        self.levels >= 2 || self.contrast != 1.0 || self.saturation != 1.0 || self.gamma != 1.0
    }

    pub fn prepare<'a>(&self, reference: &'a Reference) -> Cow<'a, Reference> {
        let mut target = Cow::Borrowed(reference);
        if self.blur > 0.0 {
            target = Cow::Owned(target.blurred(self.blur));
        }
        if self.smooth > 0.0 {
            target = Cow::Owned(target.smoothed(self.smooth));
        }
        if self.grades() {
            target = Cow::Owned(target.graded(self.contrast, self.saturation, self.gamma, self.levels));
        }
        target
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct LayerConfig {
    pub name: String,
    pub brush: Option<Named<BrushRef>>,
    pub stroke: Option<Named<StrokeType>>,
    pub shape: Option<Named<ShapeType>>,
    #[serde(default)]
    pub count: usize,
    #[serde(default)]
    pub spacing: f32,
    #[serde(default = "quarter")]
    pub scatter: f32,
    pub region: Option<Named<Region>>,
    #[serde(default)]
    pub placement: PlacementMode,
    #[serde(default)]
    pub blend: Blend,
    #[serde(default)]
    pub reference: Target,
    #[serde(default)]
    pub accident: f32,
    #[serde(default)]
    pub skip: bool,
    pub candidates: Option<usize>,
    pub refinements: Option<usize>,
    #[serde(flatten)]
    pub overrides: BTreeMap<String, toml::Value>,
}

fn quarter() -> f32 {
    0.25
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Named<T> {
    Name(String),
    Inline(T),
}
