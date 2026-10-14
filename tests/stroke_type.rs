use bot_ross::color::Swatch;
use bot_ross::flow::Flow;
use bot_ross::painting::stroke_type::{PressureShape, StrokeType};
use bot_ross::reference::Reference;
use bot_ross::shape::fill::Span;
use bot_ross::stroke::Point;
use rand::SeedableRng;
use rand::rngs::StdRng;

fn kind() -> StrokeType {
    StrokeType { radius: [4.0, 9.0], length: [20.0, 80.0], bend: [0.0, 0.3], pressure: PressureShape::Hand, jitter: 0.1, contrast: 1.0, saturation: 1.0, value: 1.0, palette: vec![], follow: 0.0, opacity: 1.0, aim: None, angle: None, length_bias: 0.5 }
}

fn reference() -> Reference {
    Reference::from_rgb8(200, 200, &vec![128; 200 * 200 * 3])
}

#[test]
fn random_strokes_stay_within_the_type_ranges() {
    let (kind, reference) = (kind(), reference());
    let mut rng = StdRng::seed_from_u64(3);
    for _ in 0..1000 {
        let stroke = kind.random(Point { x: 100.0, y: 100.0 }, &reference, &Flow::flat(200, 200), &mut rng);
        assert!((4.0..=9.0).contains(&stroke.radius));
        let chord = stroke.path.from.distance(stroke.path.to);
        assert!((19.9..=80.1).contains(&chord), "{chord}");
        assert!((stroke.color.value - 0.5).abs() <= 0.11);
        assert!(stroke.pressure.at(0.0) < 0.5 && stroke.pressure.at(0.5) == 1.0);
    }
}

#[test]
fn length_bias_skews_lengths_toward_either_end_of_the_range() {
    let reference = reference();
    let median = |bias: f32| {
        let kind = StrokeType { length_bias: bias, bend: [0.0, 0.0], ..kind() };
        let mut rng = StdRng::seed_from_u64(5);
        let mut chords: Vec<f32> = (0..1001).map(|_| kind.random(Point { x: 100.0, y: 100.0 }, &reference, &Flow::flat(200, 200), &mut rng)).map(|s| s.path.from.distance(s.path.to)).collect();
        chords.sort_by(|a, b| a.total_cmp(b));
        chords[500]
    };
    assert!((median(0.5) - 50.0).abs() < 4.0, "{}", median(0.5));
    assert!(median(0.1) < 30.0, "{}", median(0.1));
    assert!(median(0.9) > 70.0, "{}", median(0.9));
    assert!(StrokeType { length_bias: 1.0, ..kind() }.validate("s").iter().any(|p| p.contains("length_bias")));
    assert!(StrokeType { length_bias: 0.5, ..kind() }.validate("s").is_empty());
}

#[test]
fn mutations_keep_radius_and_length_within_range_and_points_finite() {
    let (kind, reference) = (kind(), reference());
    let mut rng = StdRng::seed_from_u64(4);
    let mut stroke = kind.random(Point { x: 100.0, y: 100.0 }, &reference, &Flow::flat(200, 200), &mut rng);
    for _ in 0..2000 {
        stroke = kind.mutate(&stroke, &mut rng);
        assert!((4.0..=9.0).contains(&stroke.radius));
        let chord = stroke.path.from.distance(stroke.path.to);
        assert!((19.0..=81.0).contains(&chord), "{chord}");
        assert!([stroke.path.from, stroke.path.control[0], stroke.path.control[1], stroke.path.to].iter().all(|p| p.x.is_finite() && p.y.is_finite()));
    }
}

#[test]
fn mutations_keep_the_heading_within_the_angle_span() {
    let (kind, reference) = (StrokeType { angle: Some(Span::Range([-10.0, 10.0])), bend: [0.0, 0.0], ..kind() }, reference());
    let mut rng = StdRng::seed_from_u64(6);
    let mut stroke = kind.random(Point { x: 100.0, y: 100.0 }, &reference, &Flow::flat(200, 200), &mut rng);
    for _ in 0..2000 {
        stroke = kind.mutate(&stroke, &mut rng);
        let heading = (stroke.path.to.y - stroke.path.from.y).atan2(stroke.path.to.x - stroke.path.from.x).to_degrees();
        assert!((-10.01..=10.01).contains(&heading), "{heading}");
    }
}

#[test]
fn contrast_spreads_or_flattens_stroke_lightness() {
    let sample = |contrast: f32, grey: u8| {
        let kind = StrokeType { jitter: 0.0, contrast, ..kind() };
        let reference = Reference::from_rgb8(50, 50, &vec![grey; 50 * 50 * 3]);
        kind.random(Point { x: 25.0, y: 25.0 }, &reference, &Flow::flat(50, 50), &mut StdRng::seed_from_u64(1)).color.value
    };
    assert!((sample(1.0, 191) - 0.75).abs() < 0.01);
    assert_eq!((sample(0.0, 191), sample(0.0, 40)), (0.5, 0.5));
    assert_eq!((sample(2.0, 220), sample(2.0, 40)), (1.0, 0.0));
    assert!(sample(1.5, 191) > 0.8 && sample(1.5, 40) < 0.15);
}

#[test]
fn saturation_scales_stroke_colour_toward_grey() {
    let reference = Reference::from_rgb8(50, 50, &[200, 40, 40].repeat(2500));
    let sample = |saturation: f32| {
        let kind = StrokeType { jitter: 0.0, saturation, ..kind() };
        kind.random(Point { x: 25.0, y: 25.0 }, &reference, &Flow::flat(50, 50), &mut StdRng::seed_from_u64(1)).color.saturation
    };
    assert_eq!(sample(0.0), 0.0);
    assert!((sample(0.5) - 0.4).abs() < 0.01);
    assert_eq!(sample(2.0), 1.0);
}

#[test]
fn value_scales_stroke_lightness_so_zero_is_ink() {
    let reference = Reference::from_rgb8(50, 50, &[200, 150, 100].repeat(2500));
    let sample = |value: f32| {
        let kind = StrokeType { jitter: 0.0, value, ..kind() };
        kind.random(Point { x: 25.0, y: 25.0 }, &reference, &Flow::flat(50, 50), &mut StdRng::seed_from_u64(1)).color
    };
    assert_eq!(sample(0.0).to_rgb(), [0.0, 0.0, 0.0]);
    assert!((sample(0.5).value - 0.39).abs() < 0.01);
    assert_eq!(sample(2.0).value, 1.0);
}

#[test]
fn palette_replaces_the_sampled_colour() {
    let swatch = |hex: &str| Swatch::try_from(hex.to_string()).unwrap();
    let kind = StrokeType { jitter: 0.0, palette: vec![swatch("#0000ff"), swatch("#ff0000")], ..kind() };
    let mut rng = StdRng::seed_from_u64(2);
    let hues: std::collections::BTreeSet<u32> = (0..40).map(|_| (kind.random(Point { x: 50.0, y: 50.0 }, &reference(), &Flow::flat(200, 200), &mut rng).color.hue * 1000.0) as u32).collect();
    assert_eq!(hues, [0, 666].into_iter().collect());
    assert!(Swatch::try_from("#12345".to_string()).unwrap_err().contains("#rrggbb"));
    assert_eq!(String::from(swatch("#1a2b3c")), "#1a2b3c");
}

#[test]
fn profiles_match_their_shape_and_seeds_reproduce() {
    assert_eq!(PressureShape::Flat.profile().at(0.3), 1.0);
    assert!(PressureShape::Fading.profile().at(1.0) < 0.2);
    let (kind, reference) = (kind(), reference());
    let sample = |seed| kind.random(Point { x: 50.0, y: 50.0 }, &reference, &Flow::flat(200, 200), &mut StdRng::seed_from_u64(seed));
    assert_eq!(sample(9), sample(9));
    assert_ne!(sample(9), sample(10));
}

#[test]
fn follow_turns_strokes_along_the_contour() {
    let mut bytes = vec![0u8; 200 * 200 * 3];
    for y in 0..200 {
        for x in 100..200 {
            bytes[(y * 200 + x) * 3..(y * 200 + x) * 3 + 3].copy_from_slice(&[255, 255, 255]);
        }
    }
    let reference = Reference::from_rgb8(200, 200, &bytes);
    let flow = Flow::of(&reference);
    let (tangent, strength) = flow.at(Point { x: 100.0, y: 100.0 });
    assert!(strength > 0.9, "{strength}");
    assert!((tangent.cos()).abs() < 0.05, "tangent {tangent}");
    assert_eq!(flow.at(Point { x: 20.0, y: 100.0 }).1, 0.0);
    let locked = StrokeType { follow: 1.0, ..kind() };
    let mut rng = StdRng::seed_from_u64(5);
    for _ in 0..200 {
        let stroke = locked.random(Point { x: 100.0, y: 100.0 }, &reference, &flow, &mut rng);
        let (dx, dy) = (stroke.path.to.x - stroke.path.from.x, stroke.path.to.y - stroke.path.from.y);
        assert!(dx.abs() < dy.abs() * 0.1, "stroke heads ({dx:.1}, {dy:.1})");
    }
    let loose = StrokeType { follow: 1.0, ..kind() };
    let stroke = loose.random(Point { x: 20.0, y: 100.0 }, &reference, &flow, &mut rng);
    assert!(stroke.path.from.distance(stroke.path.to) >= 19.0);
}

#[test]
fn followed_strokes_trace_a_curved_contour() {
    let mut bytes = vec![255u8; 300 * 300 * 3];
    for y in 0..300 {
        for x in 0..300 {
            if ((x as f32 - 150.0).powi(2) + (y as f32 - 150.0).powi(2)).sqrt() < 100.0 {
                bytes[(y * 300 + x) * 3..(y * 300 + x) * 3 + 3].copy_from_slice(&[0, 0, 0]);
            }
        }
    }
    let reference = Reference::from_rgb8(300, 300, &bytes);
    let flow = Flow::of(&reference);
    let locked = StrokeType { follow: 1.0, length: [150.0, 150.0], bend: [0.0, 0.0], ..kind() };
    let mut rng = StdRng::seed_from_u64(8);
    for _ in 0..20 {
        let stroke = locked.random(Point { x: 250.0, y: 150.0 }, &reference, &flow, &mut rng);
        for t in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let p = stroke.path.at(t);
            let off = ((p.x - 150.0).hypot(p.y - 150.0) - 100.0).abs();
            assert!(off < 20.0, "point {p:?} is {off:.1} px off the contour");
        }
        assert!(stroke.path.from.distance(stroke.path.to) < 150.0);
    }
}

#[test]
fn stroke_opacity_is_carried_and_hue_only_drifts_a_little() {
    let (kind, reference) = (StrokeType { opacity: 0.4, jitter: 0.2, ..kind() }, reference());
    let mut rng = StdRng::seed_from_u64(2);
    for _ in 0..200 {
        let stroke = kind.random(Point { x: 100.0, y: 100.0 }, &reference, &Flow::flat(200, 200), &mut rng);
        assert_eq!(stroke.opacity, 0.4);
        assert!(stroke.color.hue < 0.031 || stroke.color.hue > 0.969, "{}", stroke.color.hue);
    }
}

#[test]
fn aimed_strokes_point_at_the_target() {
    let flow = Flow::radial(200, 200, [100.0, 100.0]);
    let (angle, strength) = flow.at(Point { x: 20.0, y: 100.0 });
    assert!((angle.cos() - 1.0).abs() < 1e-5 && strength == 1.0, "{angle} {strength}");
    let aimed = StrokeType { follow: 1.0, aim: Some([100.0, 100.0]), bend: [0.0, 0.0], ..kind() };
    let mut rng = StdRng::seed_from_u64(6);
    for _ in 0..100 {
        let stroke = aimed.random(Point { x: 20.0, y: 100.0 }, &reference(), &flow, &mut rng);
        assert!((stroke.path.to.y - 100.0).abs() < 1e-3, "{:?}", stroke.path.to);
    }
}
