use crate::painting::config::{LayerConfig, Named, OutputConfig, Painting, PlacementMode, Region, SearchConfig};
use crate::painting::stroke_type::StrokeType;
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
    pub brush: PathBuf,
    pub stroke: StrokeType,
    pub count: usize,
    pub region: Option<Region>,
    pub placement: PlacementMode,
    pub candidates: usize,
    pub refinements: usize,
}

impl Plan {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Box<dyn Error>> {
        let path = path.as_ref();
        let base = path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
        Self::parse(&fs::read_to_string(path)?, base)
    }

    pub fn parse(text: &str, base: &Path) -> Result<Self, Box<dyn Error>> {
        let painting: Painting = toml::from_str(text)?;
        let mut problems = Vec::new();
        for (name, stroke) in &painting.strokes {
            problems.extend(stroke.validate(&format!("stroke \"{name}\"")));
        }
        let layers = painting.layers.iter().enumerate().filter_map(|(index, layer)| resolve(index, layer, &painting, base, &mut problems)).collect();
        if painting.layers.is_empty() {
            problems.push("no layers defined".into());
        }
        if !problems.is_empty() {
            return Err(problems.join("\n").into());
        }
        Ok(Self { background: painting.canvas.background.clone(), search: painting.search, output: painting.output.clone(), layers })
    }
}

fn resolve(index: usize, layer: &LayerConfig, painting: &Painting, base: &Path, problems: &mut Vec<String>) -> Option<LayerPlan> {
    let label = format!("layer {index} \"{}\"", layer.name);
    let mut report = |problem: String| problems.push(format!("{label}: {problem}"));
    let brush = lookup(&layer.brush, &painting.brushes, "brush", "brushes").map_err(&mut report).ok();
    let stroke = lookup(&layer.stroke, &painting.strokes, "stroke", "strokes").map_err(&mut report).ok().cloned();
    let region = layer.region.as_ref().and_then(|named| lookup(named, &painting.regions, "region", "regions").map_err(&mut report).ok().copied());
    if layer.count == 0 {
        report("count must be greater than 0".into());
    }
    let brush = brush.map(|brush| base.join(&brush.file))?;
    if !brush.exists() {
        report(format!("brush file {} not found", brush.display()));
        return None;
    }
    let stroke = stroke?;
    for problem in stroke.validate("stroke") {
        report(problem);
    }
    Some(LayerPlan {
        name: layer.name.clone(),
        brush,
        stroke,
        count: layer.count,
        region,
        placement: layer.placement,
        candidates: layer.candidates.unwrap_or(painting.search.candidates),
        refinements: layer.refinements.unwrap_or(painting.search.refinements),
    })
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
