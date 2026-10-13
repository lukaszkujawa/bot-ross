use crate::painting::config::{LayerConfig, Named, OutputConfig, PaintingConfig, PlacementMode, Region, SearchConfig, Target};
use crate::painting::shape_type::ShapeType;
use crate::painting::stroke_type::StrokeType;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Plan {
    pub background: String,
    pub search: SearchConfig,
    pub output: OutputConfig,
    pub layers: Vec<LayerPlan>,
}

#[derive(Debug)]
pub struct LayerPlan {
    pub name: String,
    pub marks: Marks,
    pub count: usize,
    pub region: Option<Region>,
    pub placement: PlacementMode,
    pub reference: Target,
    pub accident: f32,
    pub candidates: usize,
    pub refinements: usize,
    pub patience: usize,
}

#[derive(Debug)]
pub enum Marks {
    Strokes { brush: PathBuf, kind: StrokeType },
    Shapes(ShapeType),
}

impl Marks {
    fn validate(&self) -> Vec<String> {
        match self {
            Self::Strokes { kind, .. } => kind.validate("stroke"),
            Self::Shapes(kind) => kind.validate("shape"),
        }
    }
}

impl Plan {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Box<dyn Error>> {
        let path = path.as_ref();
        let base = path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
        Self::parse(&fs::read_to_string(path)?, base)
    }

    pub fn parse(text: &str, base: &Path) -> Result<Self, Box<dyn Error>> {
        let painting: PaintingConfig = toml::from_str(text)?;
        let mut problems = Vec::new();
        for (name, stroke) in &painting.strokes {
            problems.extend(stroke.validate(&format!("stroke \"{name}\"")));
        }
        for (name, shape) in &painting.shapes {
            problems.extend(shape.validate(&format!("shape \"{name}\"")));
        }
        let layers: Vec<LayerPlan> = painting.layers.iter().enumerate().filter(|(_, layer)| !layer.skip).filter_map(|(index, layer)| resolve(index, layer, &painting, base, &mut problems)).collect();
        if layers.is_empty() && problems.is_empty() {
            problems.push("no layers to paint".into());
        }
        if !problems.is_empty() {
            return Err(problems.join("\n").into());
        }
        Ok(Self { background: painting.canvas.background.clone(), search: painting.search, output: painting.output.clone(), layers })
    }

    pub fn only(&mut self, names: &[String]) -> Result<(), Box<dyn Error>> {
        if names.is_empty() {
            return Ok(());
        }
        let known: Vec<&str> = self.layers.iter().map(|layer| layer.name.as_str()).collect();
        let unknown: Vec<&str> = names.iter().map(String::as_str).filter(|name| !known.contains(name)).collect();
        if !unknown.is_empty() {
            return Err(format!("unknown layers: {}; defined layers: {}", unknown.join(", "), known.join(", ")).into());
        }
        self.layers.retain(|layer| names.contains(&layer.name));
        Ok(())
    }
}

fn resolve(index: usize, layer: &LayerConfig, painting: &PaintingConfig, base: &Path, problems: &mut Vec<String>) -> Option<LayerPlan> {
    let label = format!("layer {index} \"{}\"", layer.name);
    let mut report = |problem: String| problems.push(format!("{label}: {problem}"));
    let region = layer.region.as_ref().and_then(|named| lookup(named, &painting.regions, "region", "regions").and_then(|region| overridden(region, &layer.overrides, "region_")).map_err(&mut report).ok());
    for key in layer.overrides.keys() {
        let (kind, _) = key.split_once('_').unwrap_or((key, ""));
        let defined = match kind {
            "stroke" => layer.stroke.is_some(),
            "shape" => layer.shape.is_some(),
            "region" => layer.region.is_some(),
            _ => {
                report(format!("unknown key \"{key}\""));
                continue;
            }
        };
        if !defined {
            report(format!("{key} needs a {kind}"));
        }
    }
    if layer.count == 0 {
        report("count must be greater than 0".into());
    }
    let reference = layer.reference;
    if [reference.blur, reference.smooth, reference.edges, reference.contrast, reference.saturation].iter().any(|v| *v < 0.0) {
        report("reference blur, smooth, edges, contrast and saturation must not be negative".into());
    }
    if reference.gamma <= 0.0 {
        report("reference gamma must be greater than 0".into());
    }
    if layer.reference.levels == 1 {
        report("reference levels must be 0 or at least 2".into());
    }
    if !(0.0..=1.0).contains(&layer.accident) {
        report("accident must be between 0 and 1".into());
    }
    let marks = marks(layer, painting, base).map_err(|problems| problems.into_iter().for_each(&mut report)).ok()?;
    for problem in marks.validate() {
        report(problem);
    }
    Some(LayerPlan {
        name: layer.name.clone(),
        marks,
        count: layer.count,
        region,
        placement: layer.placement,
        reference: layer.reference,
        accident: layer.accident,
        candidates: layer.candidates.unwrap_or(painting.search.candidates),
        refinements: layer.refinements.unwrap_or(painting.search.refinements),
        patience: painting.search.patience,
    })
}

fn marks(layer: &LayerConfig, painting: &PaintingConfig, base: &Path) -> Result<Marks, Vec<String>> {
    match (&layer.brush, &layer.stroke, &layer.shape) {
        (Some(brush), Some(stroke), None) => {
            let brush = lookup(brush, &painting.brushes, "brush", "brushes").map(|brush| base.join(&brush.file)).and_then(|file| if file.exists() { Ok(file) } else { Err(format!("brush file {} not found", file.display())) });
            let kind = lookup(stroke, &painting.strokes, "stroke", "strokes").and_then(|kind| overridden(kind, &layer.overrides, "stroke_"));
            match (brush, kind) {
                (Ok(brush), Ok(kind)) => Ok(Marks::Strokes { brush, kind }),
                (brush, kind) => Err([brush.err(), kind.err()].into_iter().flatten().collect()),
            }
        }
        (None, None, Some(shape)) => lookup(shape, &painting.shapes, "shape", "shapes").and_then(|kind| overridden(kind, &layer.overrides, "shape_")).map(Marks::Shapes).map_err(|problem| vec![problem]),
        (_, Some(_), Some(_)) => Err(vec!["a layer paints either a stroke or a shape, not both".into()]),
        (None, Some(_), None) => Err(vec!["a stroke layer needs a brush".into()]),
        (Some(_), None, Some(_)) => Err(vec!["shape layers do not use a brush".into()]),
        (_, None, None) => Err(vec!["a layer needs a stroke or a shape".into()]),
    }
}

fn overridden<T: Serialize + DeserializeOwned>(value: &T, overrides: &BTreeMap<String, toml::Value>, prefix: &str) -> Result<T, String> {
    let mut table = toml::Table::try_from(value).map_err(|e| e.to_string())?;
    for (key, value) in overrides {
        if let Some(field) = key.strip_prefix(prefix) {
            table.insert(field.to_string(), value.clone());
        }
    }
    table.try_into().map_err(|e: toml::de::Error| e.to_string())
}

fn lookup<'a, T>(named: &'a Named<T>, table: &'a BTreeMap<String, T>, kind: &str, plural: &str) -> Result<&'a T, String> {
    match named {
        Named::Inline(value) => Ok(value),
        Named::Name(name) => table.get(name).ok_or_else(|| {
            let known: Vec<&str> = table.keys().map(String::as_str).collect();
            format!("unknown {kind} \"{name}\"; defined {plural}: {}", if known.is_empty() { "none".to_string() } else { known.join(", ") })
        }),
    }
}
