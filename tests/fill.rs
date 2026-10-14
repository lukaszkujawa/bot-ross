use bot_ross::canvas::Canvas;
use bot_ross::color::{Blend, Premultiplied};
use bot_ross::evolve::progress::Progress;
use bot_ross::flow::Flow;
use bot_ross::lattice::{Grain, lattice};
use bot_ross::painter::paint_layer;
use bot_ross::painting::length::Length;
use bot_ross::painting::config::{OutputConfig, PlacementMode, Target};
use bot_ross::painting::plan::{LayerPlan, Marks, Plan};
use bot_ross::painting::stroke_type::{PressureShape, StrokeType};
use bot_ross::reference::Reference;
use bot_ross::shape::fill::Span;
use bot_ross::stroke::Point;
use rand::SeedableRng;
use rand::rngs::StdRng;
use std::path::Path;

fn stroke(angle: Option<Span>) -> StrokeType {
    StrokeType { radius: [1.0, 2.0], length: Length::Range([10.0, 10.0]), bend: [0.0, 0.0], pressure: PressureShape::Flat, jitter: 0.0, contrast: 1.0, saturation: 1.0, value: 1.0, palette: vec![], follow: 0.0, opacity: 1.0, aim: None, angle, length_bias: 0.5 }
}

#[test]
fn lattice_rows_run_from_edge_to_edge_and_thin_with_coverage() {
    let mut rng = StdRng::seed_from_u64(1);
    let points = lattice(100, 100, 10.0, Grain { angle: 0.0, along: Some(10.0) }, 0.0, |_| 1.0, &mut rng);
    for row in 0..10 {
        let mut starts: Vec<f32> = points.iter().filter(|p| (p.y / 10.0) as usize == row).map(|p| p.x).collect();
        starts.sort_by(|a, b| a.total_cmp(b));
        assert!(starts[0] <= 0.0 && starts[starts.len() - 1] + 10.0 >= 100.0, "{starts:?}");
        assert!(starts.windows(2).all(|pair| (pair[1] - pair[0] - 10.0).abs() < 1e-3), "{starts:?}");
    }
    let half = lattice(100, 100, 10.0, Grain { angle: 0.0, along: Some(10.0) }, 0.0, |p| if p.x < 50.0 { 1.0 } else { 0.0 }, &mut rng);
    assert!((50..=60).contains(&half.len()) && half.iter().all(|p| p.x < 50.0), "{}", half.len());
    let angled = lattice(100, 60, 8.0, Grain { angle: 45f32.to_radians(), along: Some(20.0) }, 0.5, |_| 1.0, &mut rng);
    assert!(angled.len() > 20, "{}", angled.len());
    let on_canvas = |p: Point| (0.0..100.0).contains(&p.x) && (0.0..60.0).contains(&p.y);
    let (sin, cos) = 45f32.to_radians().sin_cos();
    assert!(angled.iter().all(|p| on_canvas(*p) || on_canvas(Point { x: p.x + 20.0 * cos, y: p.y + 20.0 * sin })));
    let full = lattice(100, 60, 8.0, Grain { angle: 45f32.to_radians(), along: None }, 0.0, |_| 1.0, &mut rng);
    assert!(full.len() > 10 && full.iter().all(|p| on_canvas(*p)), "{full:?}");
}

#[test]
fn strokes_with_an_angle_head_that_way() {
    let reference = Reference::from_rgb8(64, 64, &vec![128; 64 * 64 * 3]);
    let flow = Flow::flat(64, 64);
    let mut rng = StdRng::seed_from_u64(2);
    for _ in 0..10 {
        let down = stroke(Some(Span::Fixed(90.0))).random(Point { x: 32.0, y: 10.0 }, &reference, &flow, &mut rng);
        assert!((down.path.to.x - 32.0).abs() < 0.01 && (down.path.to.y - 20.0).abs() < 0.01, "{:?}", down.path.to);
        let slanted = stroke(Some(Span::Range([40.0, 50.0]))).random(Point { x: 32.0, y: 32.0 }, &reference, &flow, &mut rng);
        let heading = (slanted.path.to.y - 32.0).atan2(slanted.path.to.x - 32.0).to_degrees();
        assert!((40.0..=50.0).contains(&heading), "{heading}");
    }
}

#[test]
fn fill_layers_paint_every_lattice_point_without_searching() {
    let reference = Reference::from_rgb8(96, 64, &(0..96 * 64).flat_map(|_| [20, 40, 200]).collect::<Vec<_>>());
    let mut canvas = Canvas::new(96, 64, Premultiplied::WHITE);
    let mut progress = Progress::new(None, &OutputConfig { report: 1000, snapshots: None });
    let plan = LayerPlan {
        name: "hatch".into(),
        marks: Marks::Strokes { brush: "brushes/pen.toml".into(), kind: stroke(Some(Span::Fixed(0.0))) },
        count: 0,
        spacing: 16.0,
        scatter: 0.0,
        region: None,
        clip: false,
        placement: PlacementMode::Fill,
        blend: Blend::Normal,
        reference: Target::default(),
        accident: 0.0,
        candidates: 1,
        refinements: 0,
        patience: 1,
    };
    let outcome = paint_layer(&plan, &mut canvas, &reference, &mut progress, &mut StdRng::seed_from_u64(1)).unwrap();
    assert_eq!((outcome.placed, outcome.rejected), (outcome.attempts, 0), "{outcome:?}");
    assert!((40..=44).contains(&outcome.placed), "{outcome:?}");
    let capped = paint_layer(&LayerPlan { count: 7, ..plan }, &mut canvas, &reference, &mut progress, &mut StdRng::seed_from_u64(1)).unwrap();
    assert_eq!(capped.placed, 7);
}

#[test]
fn fill_needs_spacing_and_search_needs_count() {
    let base = "[brushes]\npen = { file = \"../brushes/pen.toml\" }\n[strokes]\nline = { radius = [1, 1], length = [5, 5], angle = 45 }\n[[layers]]\nname = \"one\"\nbrush = \"pen\"\nstroke = \"line\"\n";
    let parse = |text: String| Plan::parse(&text, Path::new("examples")).map_err(|e| e.to_string());
    assert!(parse(base.to_string()).unwrap_err().contains("count must be greater than 0"));
    assert!(parse(base.to_string() + "placement = \"fill\"\n").unwrap_err().contains("fill placement needs spacing"));
    assert!(parse(base.to_string() + "placement = \"fill\"\nspacing = 4\nscatter = 2\n").unwrap_err().contains("scatter must be between 0 and 1"));
    let plan = parse(base.to_string() + "placement = \"fill\"\nspacing = 4\n").unwrap();
    assert_eq!((plan.layers[0].spacing, plan.layers[0].scatter, plan.layers[0].count), (4.0, 0.25, 0));
}
