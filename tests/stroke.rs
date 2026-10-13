use bot_ross::stroke::{Curve, Point, Pressure};

fn p(x: f32, y: f32) -> Point {
    Point { x, y }
}

#[test]
fn straight_line_curve_is_a_line() {
    let line = Curve::line(p(0.0, 0.0), p(100.0, 0.0));
    assert!(line.at(0.5).distance(p(50.0, 0.0)) < 0.01);
    assert!((line.length() - 100.0).abs() < 0.01);
    assert!(line.beside(0.5, 10.0).distance(p(50.0, 10.0)) < 0.01);
}

#[test]
fn bent_curve_passes_through_control_side() {
    let arc = Curve { from: p(0.0, 0.0), control: [p(0.0, 100.0), p(100.0, 100.0)], to: p(100.0, 0.0) };
    assert!(arc.at(0.5).distance(p(50.0, 75.0)) < 0.01);
    assert!(arc.length() > 100.0);
    let beside = arc.beside(0.5, 10.0);
    assert!((beside.x - 50.0).abs() < 0.01 && (beside.y - 85.0).abs() < 0.01);
}

#[test]
fn pressure_interpolates_smoothly_between_keys() {
    let pressure = Pressure::new(vec![0.2, 1.0, 0.4]);
    assert_eq!(pressure.at(0.0), 0.2);
    assert_eq!(pressure.at(0.5), 1.0);
    assert_eq!(pressure.at(1.0), 0.4);
    assert!((pressure.at(0.25) - 0.6).abs() < 1e-5);
    assert_eq!(pressure.at(2.0), 0.4);
    assert_eq!(Pressure::constant(0.7).at(0.3), 0.7);
}
