use bot_ross::brush::Brush;
use bot_ross::brush::config::BrushConfig;
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
    for preset in ["wet_oil", "dry_brush", "knife", "calligraphy", "pen", "airbrush", "soft_round"] {
        let brush = Brush::load(format!("brushes/{preset}.toml")).unwrap();
        assert!(!brush.name.is_empty(), "{preset}");
    }
}

#[test]
fn brush_needs_exactly_one_tool() {
    let neither = &MINIMAL[..MINIMAL.find("[bristles]").unwrap()];
    assert!(Brush::from_config(BrushConfig::parse(neither).unwrap(), Path::new(".")).is_err());
    let both = format!("{MINIMAL}\n[stamp]\ntip = \"brush_tips/knife.svg\"\nspacing = 0.1\nripple = 0.0\nwobble = 0.0\nstreaks = 0.0\nsmear = 0.0\n");
    assert!(Brush::from_config(BrushConfig::parse(&both).unwrap(), Path::new(".")).is_err());
}
