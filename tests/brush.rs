use bot_ross::brush::Brush;
use bot_ross::brush::config::BrushConfig;
use bot_ross::color::Hsv;
use bot_ross::layer::Layer;
use bot_ross::stroke::{Curve, Point, Pressure, Stroke};
use std::path::Path;

const MINIMAL: &str = r#"
name = "Minimal"
[settings]
hardness = 0.3
[bristles]
density = 0.2
spread = 0.5
thickness = [0.1, 0.2]
opacity = [0.5, 1.0]
shade = 0.0
wobble = 0.0
reach = [1.0, 1.0]
"#;

#[test]
fn minimal_config_builds_engine_with_overrides() {
    let config = BrushConfig::parse(MINIMAL).unwrap();
    assert_eq!(config.engine().unwrap().get("hardness").unwrap(), 0.3);
    assert_eq!(config.paint.load, 1.0);
}

#[test]
fn unknown_setting_is_rejected_when_building() {
    let config = BrushConfig::parse(&MINIMAL.replace("hardness", "fluffiness")).unwrap();
    assert!(config.engine().is_err());
}

#[test]
fn shipped_presets_load() {
    let presets: Vec<_> = std::fs::read_dir("brushes").unwrap().map(|entry| entry.unwrap().path()).filter(|path| path.extension().is_some_and(|ext| ext == "toml")).collect();
    assert_eq!(presets.len(), 11);
    for preset in presets {
        let brush = Brush::load(&preset).unwrap();
        assert!(!brush.name.is_empty(), "{}", preset.display());
    }
}

#[test]
fn brush_needs_exactly_one_tool() {
    let neither = &MINIMAL[..MINIMAL.find("[bristles]").unwrap()];
    assert!(Brush::from_config(BrushConfig::parse(neither).unwrap(), Path::new(".")).is_err());
    let both = format!("{MINIMAL}\n[stamp]\ntip = \"brush_tips/knife.svg\"\nspacing = 0.1\nripple = 0.0\nwobble = 0.0\nstreaks = 0.0\nsmear = 0.0\n");
    assert!(Brush::from_config(BrushConfig::parse(&both).unwrap(), Path::new(".")).is_err());
}

#[test]
fn sub_pixel_strokes_still_paint() {
    let mut layer = Layer::new(100, 60);
    let stroke = Stroke {
        path: Curve::line(Point { x: 10.0, y: 30.0 }, Point { x: 90.0, y: 30.0 }),
        pressure: Pressure::constant(1.0),
        color: Hsv { hue: 0.0, saturation: 1.0, value: 0.0 },
        radius: 0.2,
        opacity: 1.0,
        seed: 1,
    };
    Brush::load("brushes/pen.toml").unwrap().paint(&mut layer, &stroke);
    let column: u32 = (0..60).map(|y| layer.pixels()[y * 100 + 50].alpha as u32).sum();
    assert!(column > 16384, "{column}");
}

#[test]
fn liner_paints_a_true_one_pixel_line() {
    let mut layer = Layer::new(100, 60);
    let stroke = Stroke {
        path: Curve::line(Point { x: 10.0, y: 30.5 }, Point { x: 90.0, y: 30.5 }),
        pressure: Pressure::constant(1.0),
        color: Hsv { hue: 0.0, saturation: 1.0, value: 0.0 },
        radius: 0.5,
        opacity: 1.0,
        seed: 1,
    };
    Brush::load("brushes/liner.toml").unwrap().paint(&mut layer, &stroke);
    let column: Vec<u16> = (0..60).map(|y| layer.pixels()[y * 100 + 50].alpha).collect();
    assert_eq!(column.iter().filter(|a| **a > 0).count(), 1);
    assert_eq!(column[30], 32768);
}
