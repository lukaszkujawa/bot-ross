use bot_ross::mark::Mark;
use bot_ross::color::{Blend, Hsv, Premultiplied};
use bot_ross::fitness::Fitness;
use bot_ross::layer::Layer;
use bot_ross::bounds::Bounds;
use bot_ross::proxy::{Proxy, ribbon};
use bot_ross::reference::Reference;
use bot_ross::stroke::{Curve, Point, Pressure, Stroke};

const SIZE: usize = 64;

fn reference() -> Reference {
    Reference::from_rgb8(SIZE, SIZE, &(0..SIZE * SIZE * 3).map(|i| (i * 37 % 256) as u8).collect::<Vec<_>>())
}

fn layer_with_paint() -> Layer {
    let mut layer = Layer::new(SIZE, SIZE);
    let green = Premultiplied::from_rgb([0.0, 1.0, 0.0], 0.6);
    layer.blend(Bounds::around(20.0, 20.0, 12.0, SIZE, SIZE), |x, y| if x + y < 40 { green } else { Premultiplied::TRANSPARENT });
    layer
}

fn stroke() -> Stroke {
    Stroke {
        path: Curve::line(Point { x: 10.0, y: 30.0 }, Point { x: 50.0, y: 40.0 }),
        pressure: Pressure::constant(1.0),
        color: Hsv { hue: 0.6, saturation: 0.8, value: 0.9 },
        radius: 6.0,
        opacity: 1.0,
        seed: 1,
    }
}

fn whole() -> Bounds {
    Bounds::whole(SIZE, SIZE)
}

#[test]
fn delta_matches_a_full_recompute_after_painting_the_mark() {
    let mut layer = layer_with_paint();
    let mut fitness = Fitness::new(&reference(), vec![Premultiplied::WHITE; SIZE * SIZE], Blend::Normal);
    fitness.refresh(&layer, whole());
    let before = fitness.total();
    let mark = ribbon(&stroke(), Proxy::default(), SIZE, SIZE);
    let rgb = stroke().color.to_rgb();
    let delta = fitness.delta(&mark);
    assert!(delta != 0);
    layer.blend(mark.bounds(), |x, y| Premultiplied::from_rgb(rgb, mark.coverage_at(x, y)));
    fitness.refresh(&layer, mark.bounds());
    assert_eq!(fitness.total() as i64 - before as i64, delta);
    let fresh = {
        let mut fresh = Fitness::new(&reference(), vec![Premultiplied::WHITE; SIZE * SIZE], Blend::Normal);
        fresh.refresh(&layer, whole());
        fresh.total()
    };
    assert_eq!(fitness.total(), fresh);
}

#[test]
fn matching_composite_has_zero_error_and_weights_scale_it() {
    let reference = reference();
    let below: Vec<Premultiplied> = reference.pixels().iter().map(|[r, g, b]| Premultiplied::new(*r, *g, *b, 32768)).collect();
    let mut fitness = Fitness::new(&reference, below, Blend::Normal);
    assert_eq!(fitness.total(), 0);
    let mut white = Fitness::new(&reference, vec![Premultiplied::WHITE; SIZE * SIZE], Blend::Normal);
    let plain = white.total();
    white.weigh(|x, _| if x < SIZE / 2 { 2.0 } else { 0.0 });
    assert!(white.total() > plain / 2 && white.total() < plain * 2);
    fitness.weigh(|_, _| 5.0);
    assert_eq!(fitness.total(), 0);
}

#[test]
fn empty_mark_has_zero_delta() {
    let fitness = Fitness::new(&reference(), vec![Premultiplied::WHITE; SIZE * SIZE], Blend::Normal);
    let mark = Mark::new(Bounds::around(10.0, 10.0, 5.0, SIZE, SIZE));
    assert_eq!(fitness.delta(&mark), 0);
}
