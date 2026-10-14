use crate::base64;
use bot_ross::bounds::Bounds;
use bot_ross::brush::Brush;
use bot_ross::brush::config::BrushConfig;
use bot_ross::canvas::Canvas;
use bot_ross::color::{Blend, Hsv, Premultiplied, parse_hex};
use bot_ross::image::Image;
use bot_ross::output::{png_bytes, save_image};
use bot_ross::painting::stroke_type::PressureShape;
use bot_ross::sketch::{Settings, stroke_from_drag};
use bot_ross::stroke::Point;
use image::imageops::FilterType;
use image::{RgbImage, imageops};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub struct Studio {
    brushes_dir: PathBuf,
    brushes: HashMap<PathBuf, Brush>,
    canvas: Canvas,
    background: Premultiplied,
    layers: usize,
    strokes: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, default)]
struct NewCanvas {
    width: usize,
    height: usize,
    background: String,
}

impl Default for NewCanvas {
    fn default() -> Self {
        Self { width: 1024, height: 1024, background: "#ffffff".into() }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Draw {
    brush: String,
    strokes: Vec<StrokeSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StrokeSpec {
    points: Vec<[f32; 2]>,
    #[serde(default = "default_radius")]
    radius: f32,
    #[serde(default = "default_colour")]
    colour: String,
    #[serde(default = "default_opacity")]
    opacity: f32,
    #[serde(default)]
    pressure: PressureShape,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields, default)]
struct AddLayer {
    blend: Blend,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, default)]
struct View {
    region: Option<[u32; 4]>,
    max_side: u32,
}

impl Default for View {
    fn default() -> Self {
        Self { region: None, max_side: 768 }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Save {
    path: PathBuf,
}

fn default_radius() -> f32 {
    8.0
}

fn default_colour() -> String {
    "#222222".into()
}

fn default_opacity() -> f32 {
    0.8
}

impl Studio {
    pub fn new(brushes_dir: PathBuf, width: usize, height: usize) -> Self {
        let background = Premultiplied::WHITE;
        let mut studio = Self { brushes_dir, brushes: HashMap::new(), canvas: Canvas::new(width, height, background), background, layers: 0, strokes: 0 };
        studio.reset(width, height, background);
        studio
    }

    pub fn image(&self) -> Image {
        self.canvas.render()
    }

    pub fn call(&mut self, name: &str, arguments: Value) -> Result<Vec<Value>, String> {
        match name {
            "new_canvas" => {
                let spec: NewCanvas = parse(name, arguments)?;
                let background = Premultiplied::from_rgb(colour(&spec.background)?, 1.0);
                self.reset(spec.width.max(1), spec.height.max(1), background);
                Ok(text(format!("blank {}x{} canvas with one normal layer", self.canvas.width, self.canvas.height)))
            }
            "list_brushes" => self.list_brushes().map(text),
            "draw" => self.draw(parse(name, arguments)?),
            "add_layer" => {
                let spec: AddLayer = parse(name, arguments)?;
                self.canvas.add_layer(spec.blend);
                self.layers += 1;
                Ok(text(format!("layer {} added with {:?} blend", self.layers, spec.blend).to_lowercase()))
            }
            "view" => self.view(parse(name, arguments)?),
            "save" => {
                let spec: Save = parse(name, arguments)?;
                save_image(&self.image(), &spec.path).map_err(|problem| problem.to_string())?;
                Ok(text(format!("saved {}", spec.path.display())))
            }
            "clear" => {
                self.reset(self.canvas.width, self.canvas.height, self.background);
                Ok(text("canvas cleared".into()))
            }
            _ => Err(format!("unknown tool {name}")),
        }
    }

    fn reset(&mut self, width: usize, height: usize, background: Premultiplied) {
        self.canvas = Canvas::new(width, height, background);
        self.canvas.add_layer(Blend::Normal);
        self.background = background;
        self.layers = 1;
    }

    fn list_brushes(&self) -> Result<String, String> {
        let mut entries = fs::read_dir(&self.brushes_dir).map_err(|problem| format!("cannot read {}: {problem}", self.brushes_dir.display()))?.filter_map(|entry| entry.ok().map(|entry| entry.path())).filter(|path| path.extension().is_some_and(|ext| ext == "toml")).collect::<Vec<_>>();
        entries.sort();
        let lines: Vec<String> = entries
            .iter()
            .filter_map(|path| {
                let stem = path.file_stem()?.to_string_lossy();
                let name = BrushConfig::load(path).map(|config| config.name).unwrap_or_else(|_| "unreadable".into());
                Some(format!("{stem}: {name}"))
            })
            .collect();
        Ok(lines.join("\n"))
    }

    fn draw(&mut self, spec: Draw) -> Result<Vec<Value>, String> {
        let path = self.brush_path(&spec.brush);
        if !self.brushes.contains_key(&path) {
            let brush = Brush::load(&path).map_err(|problem| format!("cannot load brush {}: {problem}", path.display()))?;
            self.brushes.insert(path.clone(), brush);
        }
        let brush = self.brushes.get_mut(&path).expect("brush was just cached");
        let layer = self.canvas.top().expect("canvas keeps a layer");
        let mut painted = Bounds::empty();
        for stroke in &spec.strokes {
            let settings = Settings { color: Hsv::from_rgb(colour(&stroke.colour)?), radius: stroke.radius, opacity: stroke.opacity, pressure: stroke.pressure };
            let drag: Vec<Point> = stroke.points.iter().map(|[x, y]| Point { x: *x, y: *y }).collect();
            let Some(stroke) = stroke_from_drag(&drag, &settings, self.strokes) else { continue };
            self.strokes += 1;
            painted = painted.union(brush.paint(layer, &stroke));
        }
        Ok(text(format!("painted {} strokes with {} on layer {}, touching x {}..{} y {}..{}", spec.strokes.len(), brush.name, self.layers, painted.left, painted.right, painted.top, painted.bottom)))
    }

    fn view(&self, spec: View) -> Result<Vec<Value>, String> {
        let image = self.image();
        let bytes: Vec<u8> = image.argb.iter().flat_map(|px| [(px >> 16) as u8, (px >> 8) as u8, *px as u8]).collect();
        let mut rgb = RgbImage::from_raw(image.width as u32, image.height as u32, bytes).ok_or("canvas size mismatch")?;
        if let Some([x, y, width, height]) = spec.region {
            let (x, y) = (x.min(rgb.width() - 1), y.min(rgb.height() - 1));
            let (width, height) = (width.clamp(1, rgb.width() - x), height.clamp(1, rgb.height() - y));
            rgb = imageops::crop(&mut rgb, x, y, width, height).to_image();
        }
        let longest = rgb.width().max(rgb.height());
        let scale = (spec.max_side.max(1) as f32 / longest as f32).min(1.0);
        if scale < 1.0 {
            rgb = imageops::resize(&rgb, ((rgb.width() as f32 * scale) as u32).max(1), ((rgb.height() as f32 * scale) as u32).max(1), FilterType::Triangle);
        }
        let png = png_bytes(&Image { width: rgb.width() as usize, height: rgb.height() as usize, argb: rgb.pixels().map(|px| 0xff00_0000 | (px[0] as u32) << 16 | (px[1] as u32) << 8 | px[2] as u32).collect() }).map_err(|problem| problem.to_string())?;
        let region = spec.region.map_or_else(|| "the whole canvas".to_string(), |[x, y, w, h]| format!("region x {x} y {y} {w}x{h}"));
        Ok(vec![
            json!({ "type": "image", "data": base64::encode(&png), "mimeType": "image/png" }),
            json!({ "type": "text", "text": format!("canvas {}x{} with {} layers; showing {} at {:.0}% scale", image.width, image.height, self.layers, region, scale * 100.0) }),
        ])
    }

    fn brush_path(&self, brush: &str) -> PathBuf {
        let path = Path::new(brush);
        if path.extension().is_some_and(|ext| ext == "toml") { path.to_path_buf() } else { self.brushes_dir.join(format!("{brush}.toml")) }
    }
}

fn parse<T: for<'a> Deserialize<'a>>(tool: &str, arguments: Value) -> Result<T, String> {
    serde_json::from_value(arguments).map_err(|problem| format!("{tool}: {problem}"))
}

fn colour(spec: &str) -> Result<[f32; 3], String> {
    match spec {
        "white" => Ok([1.0, 1.0, 1.0]),
        hex => parse_hex(hex),
    }
}

fn text(text: String) -> Vec<Value> {
    vec![json!({ "type": "text", "text": text })]
}
