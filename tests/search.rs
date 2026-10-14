use bot_ross::canvas::Canvas;
use bot_ross::color::{Blend, Premultiplied};
use bot_ross::evolve::progress::Progress;
use bot_ross::painter::paint_layer;
use bot_ross::painting::length::Length;
use bot_ross::painting::config::{OutputConfig, PlacementMode, Target};
use bot_ross::painting::plan::{LayerPlan, Marks};
use bot_ross::painting::shape_type::{Points, ShapeType};
use bot_ross::painting::stroke_type::{PressureShape, StrokeType};
use bot_ross::shape::fill::{Fill, Span};
use bot_ross::reference::Reference;
use rand::SeedableRng;
use rand::rngs::StdRng;

fn plan(count: usize) -> LayerPlan {
    LayerPlan {
        name: "test".into(),
        marks: Marks::Strokes { brush: "brushes/pen.toml".into(), kind: StrokeType { radius: [4.0, 10.0], length: Length::Range([10.0, 40.0]), bend: [0.0, 0.2], pressure: PressureShape::Flat, jitter: 0.0, contrast: 1.0, saturation: 1.0, value: 1.0, palette: vec![], follow: 0.0, opacity: 1.0, aim: None, angle: None, length_bias: 0.5 } },
        count,
        spacing: 0.0,
        scatter: 0.0,
        region: None,
        clip: false,
        placement: PlacementMode::Error,
        blend: Blend::Normal,
        reference: Target::default(),
        accident: 0.0,
        candidates: 6,
        refinements: 1,
        patience: 4,
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
    let outcome = paint_layer(&plan(12), &mut canvas, &reference, &mut progress, &mut StdRng::seed_from_u64(1)).unwrap();
    assert!(outcome.placed > 0, "{outcome:?}");
    let painted = canvas.render().argb.iter().filter(|px| **px != 0x00ffffff).count();
    assert!(painted > 0);
    let error_now = outcome.error;
    let second = paint_layer(&plan(12), &mut canvas, &reference, &mut progress, &mut StdRng::seed_from_u64(2)).unwrap();
    assert!(second.error < error_now);
}

#[test]
fn nothing_is_placed_when_the_canvas_already_matches() {
    let reference = blue();
    let mut canvas = Canvas::new(96, 64, Premultiplied::from_rgb([20.0 / 255.0, 40.0 / 255.0, 200.0 / 255.0], 1.0));
    let mut progress = Progress::new(None, &OutputConfig::default());
    let outcome = paint_layer(&LayerPlan { patience: 2, ..plan(5) }, &mut canvas, &reference, &mut progress, &mut StdRng::seed_from_u64(1)).unwrap();
    assert_eq!(outcome.placed, 0, "{outcome:?}");
    assert_eq!(outcome.attempts, 10);
}

#[test]
fn blurred_target_and_edge_weights_still_converge() {
    let reference = Reference::from_rgb8(96, 64, &(0..96 * 64).flat_map(|i| if i % 96 < 48 { [20, 40, 200] } else { [230, 220, 40] }).collect::<Vec<_>>());
    let mut canvas = Canvas::new(96, 64, Premultiplied::WHITE);
    let mut progress = Progress::new(None, &OutputConfig { report: 1000, snapshots: None });
    let tweaked = LayerPlan { reference: Target { blur: 4.0, edges: 3.0, ..Target::default() }, ..plan(20) };
    let outcome = paint_layer(&tweaked, &mut canvas, &reference, &mut progress, &mut StdRng::seed_from_u64(3)).unwrap();
    assert!(outcome.placed > 5, "{outcome:?}");
}

#[test]
fn accidents_keep_strokes_that_do_not_help() {
    let reference = blue();
    let mut canvas = Canvas::new(96, 64, Premultiplied::from_rgb([20.0 / 255.0, 40.0 / 255.0, 200.0 / 255.0], 1.0));
    let mut progress = Progress::new(None, &OutputConfig::default());
    let careless = LayerPlan { accident: 1.0, ..plan(5) };
    let outcome = paint_layer(&careless, &mut canvas, &reference, &mut progress, &mut StdRng::seed_from_u64(1)).unwrap();
    assert_eq!(outcome.placed, 5, "{outcome:?}");
    assert!(outcome.error > 0);
}

#[test]
fn shape_layers_place_exact_polygons_without_rejections() {
    let reference = blue();
    let mut canvas = Canvas::new(96, 64, Premultiplied::WHITE);
    let mut progress = Progress::new(None, &OutputConfig { report: 1000, snapshots: None });
    let kind = ShapeType { points: Points::Range([3, 6]), radius: [4.0, 12.0], irregular: 0.3, follow: 0.5, aspect: Span::Fixed(1.0), soft: Span::Fixed(0.0), overlap: false, jitter: 0.0, contrast: 1.0, saturation: 1.0, value: 1.0, palette: vec![], opacity: 1.0, fill: Fill::Hatch { spacing: Span::Fixed(3.0), width: Span::Range([0.5, 1.5]), angle: None, cross: true } };
    let shapes = LayerPlan { marks: Marks::Shapes(kind), ..plan(15) };
    let outcome = paint_layer(&shapes, &mut canvas, &reference, &mut progress, &mut StdRng::seed_from_u64(1)).unwrap();
    assert_eq!((outcome.placed, outcome.rejected), (15, 0), "{outcome:?}");
    let painted = canvas.render().argb.iter().filter(|px| **px != 0x00ffffff).count();
    assert!(painted > 0);
}

#[test]
fn clipped_layers_paint_only_inside_their_region() {
    let reference = blue();
    let mut canvas = Canvas::new(96, 64, Premultiplied::WHITE);
    let mut progress = Progress::new(None, &OutputConfig { report: 1000, snapshots: None });
    let left = toml::from_str("select = [{ rectangle = { from = [0, 0], to = [48, 64] } }]\nfeather = 0\noutside = 0").unwrap();
    let clipped = LayerPlan { region: Some(left), clip: true, ..plan(20) };
    let outcome = paint_layer(&clipped, &mut canvas, &reference, &mut progress, &mut StdRng::seed_from_u64(1)).unwrap();
    assert!(outcome.placed > 0, "{outcome:?}");
    let image = canvas.render();
    let painted = |columns: std::ops::Range<usize>| image.argb.iter().enumerate().filter(|(i, px)| columns.contains(&(i % 96)) && **px != 0x00ffffff).count();
    assert!(painted(0..48) > 0);
    assert_eq!(painted(48..96), 0);
}
