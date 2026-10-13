use bot_ross::brush::tip::Tip;

#[test]
fn knife_tip_is_solid_inside_and_empty_outside() {
    let tip = Tip::load("brush_tips/knife.svg").unwrap();
    assert!((tip.aspect() - 0.3).abs() < 0.01);
    assert_eq!(tip.coverage(0.5, 0.5), 1.0);
    assert_eq!(tip.coverage(0.02, 0.02), 0.0);
    assert_eq!(tip.coverage(1.5, 0.5), 0.0);
    assert_eq!(tip.coverage(0.5, -0.1), 0.0);
}

#[test]
fn rejects_non_svg_file() {
    assert!(Tip::load("Cargo.toml").is_err());
}
