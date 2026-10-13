use bot_ross::painting::stroke_type::{PressureShape, StrokeType};
use bot_ross::reference::Reference;
use bot_ross::stroke::Point;
use rand::SeedableRng;
use rand::rngs::StdRng;

fn kind() -> StrokeType {
    StrokeType { radius: [4.0, 9.0], length: [20.0, 80.0], bend: [0.0, 0.3], pressure: PressureShape::Hand, jitter: 0.1 }
}

fn reference() -> Reference {
    Reference::from_rgb8(200, 200, &vec![128; 200 * 200 * 3])
}

#[test]
fn random_strokes_stay_within_the_type_ranges() {
    let (kind, reference) = (kind(), reference());
    let mut rng = StdRng::seed_from_u64(3);
    for _ in 0..1000 {
        let stroke = kind.random(Point { x: 100.0, y: 100.0 }, &reference, &mut rng);
        assert!((4.0..=9.0).contains(&stroke.radius));
        let chord = stroke.path.from.distance(stroke.path.to);
        assert!((19.9..=80.1).contains(&chord), "{chord}");
        assert!((stroke.color.value - 0.5).abs() <= 0.11);
        assert!(stroke.pressure.at(0.0) < 0.5 && stroke.pressure.at(0.5) == 1.0);
    }
}

#[test]
fn mutations_keep_radius_and_length_within_range_and_points_finite() {
    let (kind, reference) = (kind(), reference());
    let mut rng = StdRng::seed_from_u64(4);
    let mut stroke = kind.random(Point { x: 100.0, y: 100.0 }, &reference, &mut rng);
    for _ in 0..2000 {
        stroke = kind.mutate(&stroke, &mut rng);
        assert!((4.0..=9.0).contains(&stroke.radius));
        let chord = stroke.path.from.distance(stroke.path.to);
        assert!((19.0..=81.0).contains(&chord), "{chord}");
        assert!([stroke.path.from, stroke.path.control[0], stroke.path.control[1], stroke.path.to].iter().all(|p| p.x.is_finite() && p.y.is_finite()));
    }
}

#[test]
fn profiles_match_their_shape_and_seeds_reproduce() {
    assert_eq!(PressureShape::Flat.profile().at(0.3), 1.0);
    assert!(PressureShape::Fading.profile().at(1.0) < 0.2);
    let (kind, reference) = (kind(), reference());
    let sample = |seed| kind.random(Point { x: 50.0, y: 50.0 }, &reference, &mut StdRng::seed_from_u64(seed));
    assert_eq!(sample(9), sample(9));
    assert_ne!(sample(9), sample(10));
}
