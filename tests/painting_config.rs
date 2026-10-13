use bot_ross::painting::config::{PlacementMode, Target};
use bot_ross::painting::plan::Plan;
use bot_ross::painting::stroke_type::PressureShape;
use std::path::Path;

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

#[test]
fn example_configs_load_and_resolve_names() {
    let plan = Plan::load("examples/oil_portrait.toml").unwrap();
    assert_eq!(plan.layers.len(), 4);
    assert_eq!(plan.layers[0].brush, Path::new("examples/../brushes/wet_oil.toml"));
    assert_eq!(plan.layers[1].stroke.radius, [15.0, 30.0]);
    assert_eq!(plan.layers[1].stroke.pressure, PressureShape::Flat);
    let region = plan.layers[2].region.unwrap();
    assert_eq!((region.weight, region.outside, region.feather), (3.0, 1.0, 0.5));
    assert_eq!(region.weight_at(512.0, 380.0), 3.0);
    assert_eq!(region.weight_at(512.0, 380.0 + 260.0 * 1.5), 1.0);
    let mid = region.weight_at(512.0, 380.0 + 260.0 * 1.25);
    assert!(mid > 1.0 && mid < 3.0, "{mid}");
    assert_eq!(plan.layers[3].placement, PlacementMode::Edges);
    assert_eq!(plan.layers[3].reference, Target::default());
    assert_eq!((plan.search.candidates, plan.layers[0].candidates), (16, 16));
    let quick = Plan::load("examples/quick.toml").unwrap();
    assert_eq!(quick.layers[0].stroke.bend, [0.0, 0.3]);
    assert_eq!(quick.layers[0].stroke.jitter, 0.05);
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
    assert!(parse(&BASE.replace("count = 10", "count = 0")).unwrap_err().contains("count must be greater than 0"));
    assert!(parse(&BASE.replace("radius = [10, 30]", "radius = [30, 10]")).unwrap_err().contains("radius must be [min, max]"));
    assert!(parse(&BASE.replace("length = [60, 300]", "length = [60, 300], jitter = 2")).unwrap_err().contains("jitter must be between 0 and 1"));
    assert!(parse(&BASE.replace("count = 10", "count = 10\nradious = 3")).is_err());
    assert!(parse(&BASE.replace("length = [60, 300]", "length = [60, 300], contrast = -0.5")).unwrap_err().contains("contrast must not be negative"));
    assert_eq!(parse(&BASE.replace("count = 10", "count = 10\nstroke_contrast = 1.5")).unwrap().layers[0].stroke.contrast, 1.5);
    assert!(parse(&BASE.replace("count = 10", "count = 10\nreference = { blur = -1 }")).unwrap_err().contains("must not be negative"));
    assert!(parse(&BASE.replace("count = 10", "count = 10\naccident = 1.5")).unwrap_err().contains("accident must be between 0 and 1"));
    assert_eq!(parse(&BASE.replace("count = 10", "count = 10\naccident = 0.2")).unwrap().layers[0].accident, 0.2);
    assert_eq!(parse(&BASE.replace("count = 10", "count = 10\nreference = { blur = 12, edges = 2 }")).unwrap().layers[0].reference, Target { blur: 12.0, edges: 2.0, ..Target::default() });
    assert!(parse(&BASE.replace("count = 10", "count = 10\nreference = { levels = 1 }")).unwrap_err().contains("levels must be 0 or at least 2"));
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
    assert!(plan.layers[0].brush.ends_with("pen.toml"));
    let aimed = parse(&BASE.replace("stroke = \"bold\"", "stroke = { radius = [1, 2], length = [5, 9], aim = [50, 60], follow = 0.8 }")).unwrap();
    assert_eq!(aimed.layers[0].stroke.aim, Some([50.0, 60.0]));
    assert_eq!(plan.layers[0].stroke.length, [5.0, 9.0]);
    assert_eq!((plan.layers[0].candidates, plan.layers[0].refinements), (4, 3));
}

#[test]
fn prefixed_keys_override_single_stroke_and_region_fields() {
    let plan = parse(&BASE.replace("count = 10", "count = 10\nregion = \"face\"\nstroke_radius = [1, 2]\nstroke_pressure = \"flat\"\nregion_weight = 9")).unwrap();
    let stroke = &plan.layers[0].stroke;
    assert_eq!((stroke.radius, stroke.length, stroke.pressure), ([1.0, 2.0], [60.0, 300.0], PressureShape::Flat));
    let region = plan.layers[0].region.unwrap();
    assert_eq!((region.weight, region.radius), (9.0, 50.0));
    let inline = parse(&BASE.replace("stroke = \"bold\"", "stroke = { radius = [1, 2], length = [5, 9] }\nstroke_length = [7, 8]")).unwrap();
    assert_eq!(inline.layers[0].stroke.length, [7.0, 8.0]);
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
