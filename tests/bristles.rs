use bot_ross::brush::bristles::{Bristles, Spacing};
use bot_ross::brush::paint::Paint;
use bot_ross::color::Hsv;
use bot_ross::stroke::{Curve, Point, Pressure, Stroke};
use rand::SeedableRng;
use rand::rngs::StdRng;

fn stroke() -> Stroke {
    Stroke {
        path: Curve::line(Point { x: 0.0, y: 50.0 }, Point { x: 200.0, y: 50.0 }),
        pressure: Pressure::constant(1.0),
        color: Hsv { hue: 0.5, saturation: 0.5, value: 0.5 },
        radius: 10.0,
        seed: 7,
    }
}

fn bristles() -> Bristles {
    Bristles { density: 1.2, max: None, spacing: Spacing::Random, spread: 1.0, splay: 0.0, thickness: [0.1, 0.2], tip: 1.0, taper: [0.0, 0.0], ripple: 0.0, opacity: [0.4, 0.9], shade: 0.1, wobble: 0.0, onset: [0.0, 0.0], reach: [0.5, 0.8] }
}

#[test]
fn hairs_stay_within_spread_and_reach() {
    let hairs = bristles().hairs(&stroke(), &Paint::default(), &mut StdRng::seed_from_u64(1));
    assert_eq!(hairs.len(), 12);
    for hair in &hairs {
        assert!(hair.samples.iter().all(|sample| (1.0..=2.0).contains(&sample.radius)));
        let last = hair.samples.last().unwrap().point;
        assert!((100.0..=160.0).contains(&last.x), "reach {}", last.x);
        assert!(hair.samples.iter().all(|sample| (40.0..=60.0).contains(&sample.point.y)));
        assert!(hair.samples.iter().all(|sample| (0.4..=0.9).contains(&sample.opacity)));
    }
}

#[test]
fn depletion_fades_opacity_along_the_hair() {
    let paint = Paint { load: 1.0, depletion: 1.0, ..Paint::default() };
    let hair = &bristles().hairs(&stroke(), &paint, &mut StdRng::seed_from_u64(1))[0];
    let (first, last) = (hair.samples[0].opacity, hair.samples.last().unwrap().opacity);
    assert!(last < first * 0.6, "{first} -> {last}");
}

#[test]
fn same_seed_gives_same_hairs() {
    let samples = |seed| bristles().hairs(&stroke(), &Paint::default(), &mut StdRng::seed_from_u64(seed))[3].samples.clone();
    assert_eq!(samples(5), samples(5));
    assert_ne!(samples(5), samples(6));
}

#[test]
fn even_spacing_places_hairs_across_the_full_spread() {
    let bristles = Bristles { density: 0.5, spacing: Spacing::Even, ..bristles() };
    let hairs = bristles.hairs(&stroke(), &Paint::default(), &mut StdRng::seed_from_u64(1));
    let rows: Vec<f32> = hairs.iter().map(|hair| hair.samples[0].point.y).collect();
    assert_eq!(rows, vec![40.0, 45.0, 50.0, 55.0, 60.0]);
}

#[test]
fn splay_narrows_hairs_under_light_pressure() {
    let light = Stroke { pressure: Pressure::constant(0.0), ..stroke() };
    let full = Bristles { splay: 0.5, ..bristles() };
    let narrowed = full.hairs(&light, &Paint::default(), &mut StdRng::seed_from_u64(1));
    let wide = full.hairs(&stroke(), &Paint::default(), &mut StdRng::seed_from_u64(1));
    for (narrow, wide) in narrowed.iter().zip(&wide) {
        let (n, w) = (narrow.samples[0].point.y - 50.0, wide.samples[0].point.y - 50.0);
        assert!((n - w * 0.5).abs() < 1e-3, "{n} vs {w}");
    }
}

#[test]
fn tapered_hair_starts_and_ends_at_tip_width() {
    let bristles = Bristles { tip: 0.5, taper: [0.25, 0.25], thickness: [0.5, 0.5], ..bristles() };
    let hair = &bristles.hairs(&stroke(), &Paint::default(), &mut StdRng::seed_from_u64(1))[0];
    let radii: Vec<f32> = hair.samples.iter().map(|sample| sample.radius).collect();
    assert_eq!((radii[0], *radii.last().unwrap()), (0.5, 0.5));
    assert_eq!(radii[radii.len() / 2], 5.0);
    assert!(radii[1] > radii[0] && radii[radii.len() - 2] > *radii.last().unwrap());
}

#[test]
fn ripple_makes_hair_width_uneven_but_bounded() {
    let bristles = Bristles { ripple: 0.3, thickness: [0.5, 0.5], ..bristles() };
    let hair = &bristles.hairs(&stroke(), &Paint::default(), &mut StdRng::seed_from_u64(1))[0];
    let radii: Vec<f32> = hair.samples.iter().map(|sample| sample.radius).collect();
    assert!(radii.windows(2).any(|pair| pair[0] != pair[1]));
    assert!(radii.iter().all(|radius| (3.0..=7.0).contains(radius)));
}

#[test]
fn hair_count_follows_radius_and_respects_the_cap() {
    let bristles = Bristles { density: 2.0, ..bristles() };
    assert_eq!(bristles.count(50.0), 100);
    assert_eq!(bristles.count(5.0), 10);
    assert_eq!(bristles.count(0.1), 1);
    assert_eq!(Bristles { max: Some(30), ..bristles }.count(50.0), 30);
}

#[test]
fn fade_takes_opacity_to_zero_at_the_end_only() {
    let paint = Paint { fade: 0.5, ..Paint::default() };
    assert_eq!(paint.remaining(0.0), 1.0);
    assert_eq!(paint.remaining(0.5), 1.0);
    assert!((paint.remaining(0.75) - 0.5).abs() < 1e-5);
    assert_eq!(paint.remaining(1.0), 0.0);
    assert!(Paint { fade: 0.1, ..Paint::default() }.remaining(0.85) > paint.remaining(0.85));
}
