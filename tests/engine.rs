use bot_ross::mypaint::engine::Engine;

#[test]
fn loads_installed_template_by_name() {
    assert!(Engine::named("classic/brush").is_ok());
}

#[test]
fn rejects_unknown_template_and_non_brush_files() {
    assert!(Engine::named("classic/does-not-exist").is_err());
    assert!(Engine::from_file("Cargo.toml").is_err());
}

#[test]
fn settings_round_trip_by_name() {
    let mut engine = Engine::with_defaults();
    engine.set("hardness", 0.42).unwrap();
    assert_eq!(engine.get("hardness").unwrap(), 0.42);
}

#[test]
fn rejects_unknown_setting_and_input_names() {
    let mut engine = Engine::with_defaults();
    assert!(engine.set("softness", 1.0).is_err());
    assert!(engine.set_curve("opaque", "moon_phase", &[[0.0, 0.0]]).is_err());
    assert!(engine.set_curve("opaque", "pressure", &[[0.0, 0.0], [1.0, 1.0]]).is_ok());
}
