use bot_ross::color::Hsv;
use bot_ross::painting::stroke_type::PressureShape;
use bot_ross::sketch::{Settings, fit_curve, stroke_from_drag};
use bot_ross::stroke::Point;

fn settings() -> Settings {
    Settings { color: Hsv { hue: 0.6, saturation: 0.5, value: 0.4 }, radius: 12.0, opacity: 0.7, pressure: PressureShape::Flat }
}

#[test]
fn fitted_curve_passes_through_the_thirds_of_the_drag() {
    let drag = [Point { x: 0.0, y: 0.0 }, Point { x: 60.0, y: 0.0 }, Point { x: 60.0, y: 60.0 }];
    let curve = fit_curve(&drag).unwrap();
    assert_eq!((curve.from, curve.to), (drag[0], drag[2]));
    let near = |a: Point, b: Point| a.distance(b) < 0.01;
    assert!(near(curve.at(1.0 / 3.0), Point { x: 40.0, y: 0.0 }));
    assert!(near(curve.at(2.0 / 3.0), Point { x: 60.0, y: 20.0 }));
}

#[test]
fn a_click_becomes_a_dab_and_an_empty_drag_nothing() {
    let dab = fit_curve(&[Point { x: 5.0, y: 5.0 }]).unwrap();
    assert!(dab.length() > 0.0 && dab.length() <= 1.0);
    assert!(fit_curve(&[]).is_none());
}

#[test]
fn drag_takes_the_current_settings() {
    let stroke = stroke_from_drag(&[Point { x: 0.0, y: 0.0 }, Point { x: 100.0, y: 0.0 }], &settings(), 7).unwrap();
    assert_eq!((stroke.radius, stroke.opacity, stroke.seed), (12.0, 0.7, 7));
    assert_eq!(stroke.color, settings().color);
    assert_eq!(stroke.pressure.at(0.0), 1.0);
}
