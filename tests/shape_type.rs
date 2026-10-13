use bot_ross::flow::Flow;
use bot_ross::painting::shape_type::{Points, ShapeType};
use bot_ross::reference::Reference;
use bot_ross::shape::fill::{Fill, Span};
use bot_ross::stroke::Point;
use std::f32::consts::TAU;
use rand::SeedableRng;
use rand::rngs::StdRng;

fn kind() -> ShapeType {
    ShapeType { points: Points::Range([3, 7]), radius: [4.0, 9.0], irregular: 0.4, follow: 0.0, aspect: Span::Range([0.3, 1.0]), soft: Span::Fixed(0.0), overlap: true, jitter: 0.1, contrast: 1.0, saturation: 1.0, value: 1.0, palette: vec![], opacity: 0.8, fill: Fill::Solid }
}

fn reference(grey: u8) -> Reference {
    Reference::from_rgb8(100, 100, &vec![grey; 100 * 100 * 3])
}

fn flat() -> Flow {
    Flow::flat(100, 100)
}

#[test]
fn random_shapes_stay_within_the_type_ranges() {
    let (kind, reference) = (kind(), reference(128));
    let mut rng = StdRng::seed_from_u64(3);
    let mut counts = std::collections::BTreeSet::new();
    for _ in 0..500 {
        let shape = kind.random(Point { x: 50.0, y: 50.0 }, &reference, &flat(), &mut rng);
        assert!((4.0..=9.0).contains(&shape.radius));
        assert!((0.3..=1.0).contains(&shape.aspect));
        assert!((3..=7).contains(&shape.spokes.len()));
        assert!(shape.spokes.iter().all(|spoke| (0.6..=1.0).contains(&spoke.reach)));
        assert!(shape.spokes.windows(2).all(|pair| pair[0].angle <= pair[1].angle));
        assert!(shape.spokes.iter().all(|spoke| (0.0..TAU).contains(&spoke.angle)));
        assert!((shape.color.value - 0.5).abs() <= 0.11);
        assert_eq!((shape.centre, shape.opacity), (Point { x: 50.0, y: 50.0 }, 0.8));
        counts.insert(shape.spokes.len());
    }
    assert_eq!(counts.len(), 5);
    let fixed = ShapeType { points: Points::Fixed(5), ..kind };
    assert_eq!(fixed.random(Point { x: 1.0, y: 1.0 }, &reference, &flat(), &mut rng).spokes.len(), 5);
}

#[test]
fn mutations_keep_ranges_and_finite_geometry() {
    let (kind, reference) = (kind(), reference(128));
    let mut rng = StdRng::seed_from_u64(4);
    let mut shape = kind.random(Point { x: 50.0, y: 50.0 }, &reference, &flat(), &mut rng);
    let mut counts = std::collections::BTreeSet::new();
    for _ in 0..3000 {
        shape = kind.mutate(&shape, &mut rng);
        assert!((4.0..=9.0).contains(&shape.radius));
        assert!((0.3..=1.0).contains(&shape.aspect));
        assert!((3..=7).contains(&shape.spokes.len()));
        assert!(shape.spokes.iter().all(|spoke| (0.6..=1.0).contains(&spoke.reach)));
        assert!(shape.spokes.windows(2).all(|pair| pair[0].angle <= pair[1].angle));
        assert!(shape.vertices().iter().all(|p| p.x.is_finite() && p.y.is_finite()));
        counts.insert(shape.spokes.len());
    }
    assert_eq!(counts.len(), 5);
}

#[test]
fn irregularity_scales_from_even_spacing_to_random_angles() {
    let reference = reference(128);
    let even = ShapeType { irregular: 0.0, points: Points::Fixed(5), ..kind() };
    let mut rng = StdRng::seed_from_u64(2);
    for _ in 0..20 {
        let shape = even.random(Point { x: 50.0, y: 50.0 }, &reference, &flat(), &mut rng);
        for (index, spoke) in shape.spokes.iter().enumerate() {
            assert!((spoke.angle - index as f32 * TAU / 5.0).abs() < 1e-5 && spoke.reach == 1.0, "{spoke:?}");
        }
    }
    let wild = ShapeType { irregular: 1.0, points: Points::Fixed(5), ..kind() };
    let shape = wild.random(Point { x: 50.0, y: 50.0 }, &reference, &flat(), &mut rng);
    let gaps: Vec<f32> = shape.spokes.windows(2).map(|pair| pair[1].angle - pair[0].angle).collect();
    assert!(gaps.iter().any(|gap| (gap - TAU / 5.0).abs() > 0.1), "{gaps:?}");
    let mut mutated = shape.clone();
    for _ in 0..200 {
        mutated = wild.mutate(&mutated, &mut rng);
        assert!(mutated.spokes.windows(2).all(|pair| pair[0].angle <= pair[1].angle), "{:?}", mutated.spokes);
    }
}

#[test]
fn seeds_reproduce_and_contrast_applies_to_the_sampled_colour() {
    let (kind, grey) = (kind(), reference(191));
    let sample = |seed| kind.random(Point { x: 50.0, y: 50.0 }, &grey, &flat(), &mut StdRng::seed_from_u64(seed));
    assert_eq!(sample(9), sample(9));
    assert_ne!(sample(9), sample(10));
    let grey = ShapeType { jitter: 0.0, saturation: 0.0, ..kind.clone() };
    let red = Reference::from_rgb8(10, 10, &[200, 40, 40].repeat(100));
    assert_eq!(grey.random(Point { x: 5.0, y: 5.0 }, &red, &flat(), &mut StdRng::seed_from_u64(1)).color.saturation, 0.0);
    let punchy = ShapeType { jitter: 0.0, contrast: 2.0, ..kind };
    assert_eq!(punchy.random(Point { x: 50.0, y: 50.0 }, &reference(220), &flat(), &mut StdRng::seed_from_u64(1)).color.value, 1.0);
    assert_eq!(punchy.random(Point { x: 50.0, y: 50.0 }, &reference(40), &flat(), &mut StdRng::seed_from_u64(1)).color.value, 0.0);
}

#[test]
fn fill_ranges_are_picked_per_shape() {
    let reference = reference(128);
    let ranged = ShapeType { fill: Fill::Hatch { spacing: Span::Range([2.0, 6.0]), width: Span::Fixed(1.5), angle: Some(Span::Range([0.0, 90.0])), cross: false }, ..kind() };
    let mut rng = StdRng::seed_from_u64(5);
    let mut spacings = std::collections::BTreeSet::new();
    for _ in 0..50 {
        let shape = ranged.random(Point { x: 50.0, y: 50.0 }, &reference, &flat(), &mut rng);
        let Fill::Hatch { spacing, width, angle, cross } = shape.fill else { panic!("hatch expected") };
        assert!((2.0..=6.0).contains(&spacing) && width == 1.5 && (0.0..=90.0).contains(&angle.unwrap()) && !cross);
        spacings.insert(spacing.to_bits());
        let mutated = ranged.mutate(&shape, &mut rng);
        assert_eq!(mutated.fill, shape.fill);
    }
    assert!(spacings.len() > 10);
}

#[test]
fn validation_reports_each_problem() {
    let bad = ShapeType { points: Points::Range([2, 1]), radius: [5.0, 1.0], irregular: 2.0, follow: -1.0, aspect: Span::Range([0.0, 2.0]), soft: Span::Range([3.0, 1.0]), overlap: true, jitter: 1.5, contrast: -1.0, saturation: -1.0, value: -1.0, palette: vec![], opacity: 0.0, fill: Fill::Dots { spacing: Span::Fixed(0.0), radius: Span::Range([2.0, 1.0]) } };
    let problems = bad.validate("shape");
    assert_eq!(problems.len(), 13, "{problems:?}");
    assert!(kind().validate("shape").is_empty());
}

#[test]
fn follow_stops_spokes_at_contours_and_leaves_open_sides_alone() {
    let bytes: Vec<u8> = (0..100 * 100).flat_map(|i| if i % 100 < 50 { [30, 30, 30] } else { [220, 220, 220] }).collect();
    let split = Reference::from_rgb8(100, 100, &bytes);
    let flow = Flow::of(&split);
    let grabbing = ShapeType { points: Points::Fixed(6), radius: [40.0, 40.0], irregular: 0.0, follow: 1.0, aspect: Span::Fixed(1.0), ..kind() };
    let mut rng = StdRng::seed_from_u64(7);
    for _ in 0..20 {
        let shape = grabbing.random(Point { x: 30.0, y: 50.0 }, &split, &flow, &mut rng);
        let xs: Vec<f32> = shape.vertices().iter().map(|p| p.x).collect();
        assert!(xs.iter().all(|x| *x <= 53.0), "{xs:?}");
        assert!(xs.iter().any(|x| *x < 5.0), "{xs:?}");
    }
    let free = ShapeType { follow: 0.0, ..grabbing };
    let crossing = (0..20).filter(|seed| free.random(Point { x: 30.0, y: 50.0 }, &split, &flow, &mut StdRng::seed_from_u64(*seed)).vertices().iter().any(|p| p.x > 60.0)).count();
    assert!(crossing > 10, "{crossing}");
}
