use bot_ross::color::Blend;
use bot_ross::painting::config::{PlacementMode, Target};
use bot_ross::painting::plan::{LayerPlan, Marks, Plan};
use bot_ross::painting::shape_type::{Points, ShapeType};
use bot_ross::painting::stroke_type::{PressureShape, StrokeType};
use bot_ross::reference::Reference;
use bot_ross::shape::fill::{Fill, Span};
use std::path::{Path, PathBuf};

const BASE: &str = r#"
[brushes]
oil = { file = "../brushes/wet_oil.toml" }
[strokes]
bold = { radius = [10, 30], length = [60, 300] }
[regions]
face = { centre = [100, 100], radius = 50 }
[[layers]]
name = "one"
brush = "oil"
stroke = "bold"
count = 10
"#;

fn parse(text: &str) -> Result<Plan, String> {
    Plan::parse(text, Path::new("examples")).map_err(|e| e.to_string())
}

fn stroke(layer: &LayerPlan) -> (&PathBuf, &StrokeType) {
    match &layer.marks {
        Marks::Strokes { brush, kind } => (brush, kind),
        Marks::Shapes(_) => panic!("expected a stroke layer"),
    }
}

fn shape(layer: &LayerPlan) -> &ShapeType {
    match &layer.marks {
        Marks::Shapes(kind) => kind,
        Marks::Strokes { .. } => panic!("expected a shape layer"),
    }
}

#[test]
fn example_configs_load_and_resolve_names() {
    let plan = Plan::load("examples/oil_portrait.toml").unwrap();
    assert_eq!(plan.layers.len(), 4);
    assert_eq!(stroke(&plan.layers[0]).0, Path::new("examples/../brushes/wet_oil.toml"));
    assert_eq!(stroke(&plan.layers[1]).1.radius, [15.0, 30.0]);
    assert_eq!(stroke(&plan.layers[1]).1.pressure, PressureShape::Flat);
    let region = plan.layers[2].region.as_ref().unwrap();
    assert_eq!((region.weight, region.outside, region.feather), (3.0, 1.0, 0.5));
    assert_eq!((region.centre, region.radius), (Some([512.0, 380.0]), Some(260.0)));
    assert_eq!(plan.layers[3].placement, PlacementMode::Edges);
    assert_eq!(plan.layers[3].reference, Target::default());
    assert_eq!((plan.search.candidates, plan.layers[0].candidates, plan.layers[0].patience), (16, 16, 4));
    let quick = Plan::load("examples/quick.toml").unwrap();
    assert_eq!(stroke(&quick.layers[0]).1.bend, [0.0, 0.3]);
    assert_eq!(stroke(&quick.layers[0]).1.jitter, 0.05);
    assert_eq!(quick.background, "mean");
}

#[test]
fn unknown_names_are_reported_together_with_the_layer() {
    let text = BASE.replace("brush = \"oil\"", "brush = \"ink\"").replace("stroke = \"bold\"", "stroke = \"fine\"") + "region = \"hand\"\n";
    let error = parse(&text).unwrap_err();
    assert!(error.contains("layer 0 \"one\": unknown brush \"ink\"; defined brushes: oil"), "{error}");
    assert!(error.contains("unknown stroke \"fine\"; defined strokes: bold"), "{error}");
    assert!(error.contains("unknown region \"hand\"; defined regions: face"), "{error}");
}

#[test]
fn invalid_values_are_rejected_with_messages() {
    assert!(parse(&BASE.replace("count = 10", "count = 10\nclip = true")).unwrap_err().contains("clip needs a region"));
    assert!(parse(&BASE.replace("count = 10", "count = 0")).unwrap_err().contains("count must be greater than 0"));
    assert!(parse(&BASE.replace("radius = [10, 30]", "radius = [30, 10]")).unwrap_err().contains("radius must be [min, max]"));
    assert!(parse(&BASE.replace("length = [60, 300]", "length = [60, 300], jitter = 2")).unwrap_err().contains("jitter must be between 0 and 1"));
    assert!(parse(&BASE.replace("count = 10", "count = 10\nradious = 3")).is_err());
    assert!(parse(&BASE.replace("length = [60, 300]", "length = [60, 300], contrast = -0.5")).unwrap_err().contains("contrast must not be negative"));
    assert!(parse(&BASE.replace("length = [60, 300]", "length = [60, 300], saturation = -0.5")).unwrap_err().contains("saturation must not be negative"));
    assert!(parse(&BASE.replace("length = [60, 300]", "length = [60, 300], palette = [\"#12345g\"]")).unwrap_err().contains("colour must be #rrggbb"));
    assert_eq!(stroke(&parse(&BASE.replace("count = 10", "count = 10\nstroke_palette = [\"#ff0000\"]")).unwrap().layers[0]).1.palette.len(), 1);
    assert!(parse(&SHAPED.replace("irregular = 0.3", "irregular = 0.3, opacity = 0")).unwrap_err().contains("opacity must be greater than 0"));
    assert_eq!(stroke(&parse(&BASE.replace("count = 10", "count = 10\nstroke_contrast = 1.5")).unwrap().layers[0]).1.contrast, 1.5);
    assert!(parse(&BASE.replace("count = 10", "count = 10\nreference = { blur = -1 }")).unwrap_err().contains("must not be negative"));
    assert!(parse(&BASE.replace("count = 10", "count = 10\naccident = 1.5")).unwrap_err().contains("accident must be between 0 and 1"));
    assert_eq!(parse(&BASE.replace("count = 10", "count = 10\naccident = 0.2")).unwrap().layers[0].accident, 0.2);
    assert_eq!(parse(&BASE.replace("count = 10", "count = 10\nreference = { blur = 12, edges = 2 }")).unwrap().layers[0].reference, Target { blur: 12.0, edges: 2.0, ..Target::default() });
    assert!(parse(&BASE.replace("count = 10", "count = 10\nreference = { levels = 1 }")).unwrap_err().contains("levels must be 0 or at least 2"));
    assert!(parse(&BASE.replace("count = 10", "count = 10\nreference = { gamma = 0 }")).unwrap_err().contains("gamma must be greater than 0"));
    assert!(parse(&BASE.replace("count = 10", "count = 10\nreference = { gamma = 1.8 }")).unwrap().layers[0].reference.grades());
    let graded = parse(&BASE.replace("count = 10", "count = 10\nreference = { levels = 5, contrast = 1.2, saturation = 1.3 }")).unwrap().layers[0].reference;
    assert!(graded.grades() && graded.levels == 5);
    assert!(!Target::default().grades());
    assert!(parse("[brushes]\noil = { file = \"../brushes/pen.toml\" }\n").unwrap_err().contains("no layers to paint"));
    assert!(parse(&BASE.replace("count = 10", "count = 10\nskip = true")).unwrap_err().contains("no layers to paint"));
    assert!(parse(&BASE.replace("wet_oil", "missing")).unwrap_err().contains("brush file"));
}

#[test]
fn inline_definitions_and_per_layer_overrides_work() {
    let text = BASE.replace("brush = \"oil\"", "brush = { file = \"../brushes/pen.toml\" }").replace("stroke = \"bold\"", "stroke = { radius = [1, 2], length = [5, 9], pressure = \"fading\" }") + "candidates = 4\n";
    let plan = parse(&text).unwrap();
    assert!(stroke(&plan.layers[0]).0.ends_with("pen.toml"));
    let aimed = parse(&BASE.replace("stroke = \"bold\"", "stroke = { radius = [1, 2], length = [5, 9], aim = [50, 60], follow = 0.8 }")).unwrap();
    assert_eq!(stroke(&aimed.layers[0]).1.aim, Some([50.0, 60.0]));
    assert_eq!(stroke(&plan.layers[0]).1.length, [5.0, 9.0]);
    assert_eq!((plan.layers[0].candidates, plan.layers[0].refinements), (4, 3));
}

#[test]
fn prefixed_keys_override_single_stroke_and_region_fields() {
    let plan = parse(&BASE.replace("count = 10", "count = 10\nregion = \"face\"\nstroke_radius = [1, 2]\nstroke_pressure = \"flat\"\nregion_weight = 9")).unwrap();
    let kind = stroke(&plan.layers[0]).1;
    assert_eq!((kind.radius, kind.length, kind.pressure), ([1.0, 2.0], [60.0, 300.0], PressureShape::Flat));
    let region = plan.layers[0].region.as_ref().unwrap();
    assert_eq!((region.weight, region.radius), (9.0, Some(50.0)));
    let inline = parse(&BASE.replace("stroke = \"bold\"", "stroke = { radius = [1, 2], length = [5, 9] }\nstroke_length = [7, 8]")).unwrap();
    assert_eq!(stroke(&inline.layers[0]).1.length, [7.0, 8.0]);
    assert!(parse(&BASE.replace("count = 10", "count = 10\nstroke_radius = [5, 1]")).unwrap_err().contains("radius must be [min, max]"));
    assert!(parse(&BASE.replace("count = 10", "count = 10\nstroke_radious = [1, 2]")).unwrap_err().contains("unknown field `radious`"));
    assert!(parse(&BASE.replace("count = 10", "count = 10\nregion_weight = 2")).unwrap_err().contains("region_weight needs a region"));
    assert!(parse(&BASE.replace("count = 10", "count = 10\nradious = 3")).unwrap_err().contains("unknown key \"radious\""));
}

#[test]
fn skipped_layers_are_left_out_without_being_validated() {
    let text = format!("{BASE}\n[[layers]]\nname = \"later\"\nbrush = \"missing\"\nstroke = \"bold\"\ncount = 0\nskip = true\n");
    let plan = parse(&text).unwrap();
    assert_eq!(plan.layers.len(), 1);
    assert_eq!(plan.layers[0].name, "one");
}

#[test]
fn layers_can_be_selected_by_name() {
    let mut plan = Plan::load("examples/oil_portrait.toml").unwrap();
    plan.only(&["accents".into(), "blocking".into()]).unwrap();
    let names: Vec<&str> = plan.layers.iter().map(|layer| layer.name.as_str()).collect();
    assert_eq!(names, ["blocking", "accents"]);
    let error = plan.only(&["nope".into()]).unwrap_err().to_string();
    assert!(error.contains("unknown layers: nope; defined layers: blocking, accents"), "{error}");
    plan.only(&[]).unwrap();
    assert_eq!(plan.layers.len(), 2);
}

const SHAPED: &str = r#"
[shapes]
blob = { points = [3, 6], radius = [10, 40], irregular = 0.3, fill = { style = "hatch", spacing = 6, angle = 45 } }
[[layers]]
name = "blocks"
shape = "blob"
count = 10
"#;

#[test]
fn shape_layers_resolve_names_inline_definitions_and_overrides() {
    let plan = parse(SHAPED).unwrap();
    let kind = shape(&plan.layers[0]);
    assert_eq!((kind.points.range(), kind.radius, kind.irregular, kind.jitter), ([3, 6], [10.0, 40.0], 0.3, 0.05));
    assert_eq!(kind.fill, Fill::Hatch { spacing: Span::Fixed(6.0), width: Span::Fixed(1.0), angle: Some(Span::Fixed(45.0)), cross: false });
    let ranged = parse(&SHAPED.replace("spacing = 6, angle = 45", "spacing = [3, 8], width = [0.5, 2], angle = [30, 60]")).unwrap();
    assert_eq!(shape(&ranged.layers[0]).fill, Fill::Hatch { spacing: Span::Range([3.0, 8.0]), width: Span::Range([0.5, 2.0]), angle: Some(Span::Range([30.0, 60.0])), cross: false });
    let inline = parse("[[layers]]\nname = \"dots\"\nshape = { points = 4, radius = [2, 5], fill = { style = \"dots\", spacing = 3, radius = 0.8 } }\ncount = 1\n").unwrap();
    assert_eq!(shape(&inline.layers[0]).points, Points::Fixed(4));
    assert!(shape(&inline.layers[0]).overlap);
    assert!(!shape(&parse(&SHAPED.replace("irregular = 0.3", "irregular = 0.3, overlap = false")).unwrap().layers[0]).overlap);
    assert_eq!(shape(&inline.layers[0]).fill, Fill::Dots { spacing: Span::Fixed(3.0), radius: Span::Fixed(0.8) });
    let overridden = parse(&SHAPED.replace("count = 10", "count = 10\nshape_radius = [1, 2]\nshape_fill = { style = \"outline\", width = 2 }")).unwrap();
    assert_eq!((shape(&overridden.layers[0]).radius, &shape(&overridden.layers[0]).fill), ([1.0, 2.0], &Fill::Outline { width: Span::Fixed(2.0) }));
    assert_eq!(shape(&parse("[[layers]]\nname = \"s\"\nshape = { points = 3, radius = [1, 2] }\ncount = 1\n").unwrap().layers[0]).fill, Fill::Solid);
}

#[test]
fn shape_layer_problems_are_reported() {
    let error = |text: &str| parse(text).unwrap_err();
    assert!(error(&SHAPED.replace("shape = \"blob\"", "shape = \"blob\"\nbrush = \"oil\"")).contains("shape layers do not use a brush"));
    assert!(error(&SHAPED.replace("shape = \"blob\"", "shape = \"blob\"\nstroke = { radius = [1, 2], length = [1, 2] }")).contains("either a stroke or a shape, not both"));
    assert!(error(&SHAPED.replace("shape = \"blob\"", "stroke = { radius = [1, 2], length = [1, 2] }")).contains("a stroke layer needs a brush"));
    assert!(error(&SHAPED.replace("shape = \"blob\"\n", "")).contains("a layer needs a stroke or a shape"));
    assert!(error(&SHAPED.replace("shape = \"blob\"", "shape = \"blub\"")).contains("unknown shape \"blub\"; defined shapes: blob"));
    assert!(error(&SHAPED.replace("points = [3, 6]", "points = 2")).contains("points must be at least 3"));
    assert!(error(&SHAPED.replace("points = [3, 6]", "points = [5, 4]")).contains("points must be at least 3"));
    assert!(error(&SHAPED.replace("spacing = 6", "spacing = 0")).contains("fill spacing must be greater than 0"));
    assert!(error(&SHAPED.replace("spacing = 6", "spacing = [6, 2]")).contains("fill spacing must be a value or [min, max]"));
    assert!(error(&SHAPED.replace("angle = 45", "angle = [90, 10]")).contains("fill angle must be a value or [min, max]"));
    assert!(error(&SHAPED.replace("style = \"hatch\"", "style = \"stripes\"")).contains("stripes"));
    assert!(error(&SHAPED.replace("angle = 45", "angel = 45")).contains("angel"));
    assert!(error(&SHAPED.replace("count = 10", "count = 10\nstroke_radius = [1, 2]")).contains("stroke_radius needs a stroke"));
    assert!(error(&BASE.replace("count = 10", "count = 10\nshape_radius = [1, 2]")).contains("shape_radius needs a shape"));
}

#[test]
fn relative_regions_resolve_against_the_canvas() {
    let plan = parse(&BASE.replace("face = { centre = [100, 100], radius = 50 }", "face = { centre = [0.5, 0.25], radius = 0.1, relative = true }").replace("count = 10", "count = 10\nregion = \"face\"")).unwrap();
    let reference = Reference::from_rgb8(200, 400, &vec![255; 200 * 400 * 3]);
    let mask = plan.layers[0].region.as_ref().unwrap().mask(&reference);
    assert_eq!(mask[100 * 200 + 100], 3.0);
    assert_eq!(mask[100 * 200 + 150], 1.0);
    assert_eq!(mask[100 * 200 + 115], 3.0);
}

#[test]
fn layers_default_to_normal_blend_and_accept_multiply() {
    assert_eq!(parse(BASE).unwrap().layers[0].blend, Blend::Normal);
    assert_eq!(parse(&BASE.replace("count = 10", "count = 10\nblend = \"multiply\"")).unwrap().layers[0].blend, Blend::Multiply);
}

#[test]
fn oval_regions_stretch_the_circle_vertically() {
    let plan = parse(&BASE.replace("radius = 50 }", "radius = 50, aspect = 2.0 }").replace("count = 10", "count = 10\nregion = \"face\"")).unwrap();
    let reference = Reference::from_rgb8(200, 200, &vec![255; 200 * 200 * 3]);
    let mask = plan.layers[0].region.as_ref().unwrap().mask(&reference);
    assert_eq!(mask[190 * 200 + 100], 3.0);
    assert_eq!(mask[100 * 200 + 190], 1.0);
    let flat = parse(&BASE.replace("radius = 50 }", "radius = 50, aspect = 0 }").replace("count = 10", "count = 10\nregion = \"face\"")).unwrap_err();
    assert!(flat.contains("region aspect must be greater than 0"), "{flat}");
}
