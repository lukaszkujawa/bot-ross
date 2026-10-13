use bot_ross::canvas::Canvas;
use bot_ross::color::Premultiplied;
use bot_ross::evolve::progress::Progress;
use bot_ross::evolve::run_layer;
use bot_ross::painting::config::{OutputConfig, PlacementMode, SearchConfig, Target};
use bot_ross::painting::plan::LayerPlan;
use bot_ross::painting::stroke_type::{PressureShape, StrokeType};
use bot_ross::reference::Reference;
use rand::SeedableRng;
use rand::rngs::StdRng;

fn plan(count: usize) -> LayerPlan {
    LayerPlan {
        name: "test".into(),
        brush: "brushes/pen.toml".into(),
        stroke: StrokeType { radius: [4.0, 10.0], length: [10.0, 40.0], bend: [0.0, 0.2], pressure: PressureShape::Flat, jitter: 0.0, follow: 0.0, opacity: 1.0, aim: None },
        count,
        region: None,
        placement: PlacementMode::Error,
        reference: Target::default(),
        accident: 0.0,
        candidates: 6,
        refinements: 1,
    }
}

fn blue() -> Reference {
    Reference::from_rgb8(96, 64, &(0..96 * 64).flat_map(|_| [20, 40, 200]).collect::<Vec<_>>())
}

#[test]
fn strokes_lower_the_error_and_every_kept_stroke_helped() {
    let reference = blue();
    let mut canvas = Canvas::new(96, 64, Premultiplied::WHITE);
    let output = OutputConfig { report: 1000, snapshots: None };
    let mut progress = Progress::new(None, &output);
    let outcome = run_layer(&plan(12), SearchConfig::default(), &mut canvas, &reference, &mut progress, &mut StdRng::seed_from_u64(1)).unwrap();
    assert!(outcome.placed > 0, "{outcome:?}");
    let painted = canvas.render().argb.iter().filter(|px| **px != 0x00ffffff).count();
    assert!(painted > 0);
    let error_now = outcome.error;
    let second = run_layer(&plan(12), SearchConfig::default(), &mut canvas, &reference, &mut progress, &mut StdRng::seed_from_u64(2)).unwrap();
    assert!(second.error < error_now);
}

#[test]
fn nothing_is_placed_when_the_canvas_already_matches() {
    let reference = blue();
    let mut canvas = Canvas::new(96, 64, Premultiplied::from_rgb([20.0 / 255.0, 40.0 / 255.0, 200.0 / 255.0], 1.0));
    let mut progress = Progress::new(None, &OutputConfig::default());
    let outcome = run_layer(&plan(5), SearchConfig { patience: 2, ..SearchConfig::default() }, &mut canvas, &reference, &mut progress, &mut StdRng::seed_from_u64(1)).unwrap();
    assert_eq!(outcome.placed, 0, "{outcome:?}");
    assert_eq!(outcome.attempts, 10);
}

#[test]
fn blurred_target_and_edge_weights_still_converge() {
    let reference = Reference::from_rgb8(96, 64, &(0..96 * 64).flat_map(|i| if i % 96 < 48 { [20, 40, 200] } else { [230, 220, 40] }).collect::<Vec<_>>());
    let mut canvas = Canvas::new(96, 64, Premultiplied::WHITE);
    let mut progress = Progress::new(None, &OutputConfig { report: 1000, snapshots: None });
    let tweaked = LayerPlan { reference: Target { blur: 4.0, edges: 3.0, ..Target::default() }, ..plan(20) };
    let outcome = run_layer(&tweaked, SearchConfig::default(), &mut canvas, &reference, &mut progress, &mut StdRng::seed_from_u64(3)).unwrap();
    assert!(outcome.placed > 5, "{outcome:?}");
}

#[test]
fn accidents_keep_strokes_that_do_not_help() {
    let reference = blue();
    let mut canvas = Canvas::new(96, 64, Premultiplied::from_rgb([20.0 / 255.0, 40.0 / 255.0, 200.0 / 255.0], 1.0));
    let mut progress = Progress::new(None, &OutputConfig::default());
    let careless = LayerPlan { accident: 1.0, ..plan(5) };
    let outcome = run_layer(&careless, SearchConfig::default(), &mut canvas, &reference, &mut progress, &mut StdRng::seed_from_u64(1)).unwrap();
    assert_eq!(outcome.placed, 5, "{outcome:?}");
    assert!(outcome.error > 0);
}
