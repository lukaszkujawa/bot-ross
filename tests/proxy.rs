use bot_ross::color::Hsv;
use bot_ross::proxy::{Proxy, ribbon};
use bot_ross::stroke::{Curve, Point, Pressure, Stroke};

fn stroke() -> Stroke {
    Stroke {
        path: Curve::line(Point { x: 20.0, y: 50.0 }, Point { x: 80.0, y: 50.0 }),
        pressure: Pressure::constant(1.0),
        color: Hsv { hue: 0.0, saturation: 1.0, value: 1.0 },
        radius: 10.0,
        seed: 0,
    }
}

#[test]
fn ribbon_covers_the_centre_line_and_nothing_far_from_it() {
    let mark = ribbon(&stroke(), Proxy::default(), 100, 100);
    assert_eq!(mark.coverage_at(50, 50), 1.0);
    assert!(mark.coverage_at(50, 59) > 0.9);
    assert_eq!(mark.coverage_at(50, 60), 0.0);
    assert_eq!(mark.coverage_at(5, 50), 0.0);
    let cap = mark.coverage_at(86, 57);
    assert!(cap > 0.0 && cap < 1.0, "{cap}");
}

#[test]
fn proxy_hints_scale_opacity_and_width() {
    let thin = ribbon(&stroke(), Proxy { opacity: 0.5, width: 0.5 }, 100, 100);
    assert_eq!(thin.coverage_at(50, 50), 0.5);
    assert_eq!(thin.coverage_at(50, 58), 0.0);
}

#[test]
fn light_pressure_narrows_the_ribbon_and_bounds_clamp_to_the_canvas() {
    let light = Stroke { pressure: Pressure::constant(0.0), ..stroke() };
    let mark = ribbon(&light, Proxy::default(), 100, 100);
    assert!(mark.coverage_at(50, 54) > 0.9);
    assert_eq!(mark.coverage_at(50, 55), 0.0);
    let edge = Stroke { path: Curve::line(Point { x: -5.0, y: 2.0 }, Point { x: 30.0, y: 2.0 }), ..stroke() };
    let bounds = ribbon(&edge, Proxy::default(), 100, 100).bounds();
    assert_eq!((bounds.left, bounds.top), (0, 0));
}
