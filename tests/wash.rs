use bot_ross::brush::Brush;
use bot_ross::color::{Hsv, Premultiplied, luminance};
use bot_ross::layer::Layer;
use bot_ross::stroke::{Curve, Point, Pressure, Stroke};

fn wash(seed: u64) -> Vec<Premultiplied> {
    washes(seed, 1)
}

fn washes(seed: u64, passes: usize) -> Vec<Premultiplied> {
    let mut layer = Layer::new(200, 120);
    let stroke = Stroke {
        path: Curve::line(Point { x: 30.0, y: 60.0 }, Point { x: 170.0, y: 60.0 }),
        pressure: Pressure::constant(1.0),
        color: Hsv { hue: 0.6, saturation: 0.5, value: 0.6 },
        radius: 20.0,
        opacity: 0.5,
        seed,
    };
    let mut brush = Brush::load("brushes/watercolour.toml").unwrap();
    for _ in 0..passes {
        brush.paint(&mut layer, &stroke);
    }
    layer.pixels()
}

fn at(pixels: &[Premultiplied], x: usize, y: usize) -> Premultiplied {
    pixels[y * 200 + x]
}

#[test]
fn pigment_settles_densest_and_darkest_along_the_rim() {
    let pixels = wash(1);
    let centre = at(&pixels, 60, 60);
    let (y, rim) = (30..=60).map(|y| (y, at(&pixels, 60, y))).max_by_key(|(_, pixel)| pixel.alpha).unwrap();
    assert!(y < 52, "densest pixel at {y}");
    assert!(rim.alpha as f32 > centre.alpha as f32 * 1.1, "rim {} centre {}", rim.alpha, centre.alpha);
    assert!(luminance(rim.to_rgb()) < luminance(centre.to_rgb()));
}

#[test]
fn water_blooms_past_the_radius_but_not_far_beyond() {
    let pixels = wash(2);
    let beyond = (20..=160).filter(|x| at(&pixels, *x, 32).alpha > 0).count();
    assert!(beyond > 0, "no bloom past the radius");
    assert!((0..200).all(|x| at(&pixels, x, 20).alpha == 0));
    assert!(at(&pixels, 100, 60).alpha > 0);
}

#[test]
fn seed_fixes_the_blooms() {
    assert_eq!(wash(3), wash(3));
    assert_ne!(wash(3), wash(4));
}

#[test]
fn wash_runs_pale_as_the_brush_empties() {
    let pixels = wash(5);
    let start = at(&pixels, 50, 60).alpha;
    let end = at(&pixels, 160, 60).alpha;
    assert!(end < start, "wash should run pale toward the end: {start} -> {end}");
}

#[test]
fn a_wash_over_wet_paint_bleeds_in_without_a_rim() {
    let (first, both) = (wash(1), washes(1, 2));
    let added = |x: usize, y: usize| {
        let (before, after) = (at(&first, x, y).alpha as f32 / 32768.0, at(&both, x, y).alpha as f32 / 32768.0);
        (after - before) / (1.0 - before)
    };
    let rim = (30..=60).max_by_key(|y| at(&first, 60, *y).alpha).unwrap();
    assert!(added(60, rim) < added(60, 60) * 1.05, "rim {} centre {}", added(60, rim), added(60, 60));
    assert!(added(60, 60) > 0.3);
}
