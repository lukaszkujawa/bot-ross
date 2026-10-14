use bot_ross::painting::region::Region;
use bot_ross::reference::Reference;

fn region(text: &str) -> Region {
    toml::from_str(text).unwrap()
}

fn flat(width: usize, height: usize, rgb: [u8; 3]) -> Reference {
    Reference::from_rgb8(width, height, &rgb.repeat(width * height))
}

fn at(mask: &[f32], width: usize, x: usize, y: usize) -> f32 {
    mask[y * width + x]
}

#[test]
fn rectangles_and_polygons_select_with_a_soft_edge_outside_them() {
    let reference = flat(100, 100, [128; 3]);
    let mask = region("select = [{ rectangle = { from = [0, 60], to = [100, 100] } }]\nfeather = 1.0\nweight = 2\noutside = 0").mask(&reference);
    assert_eq!(at(&mask, 100, 50, 80), 2.0);
    assert_eq!(at(&mask, 100, 50, 10), 0.0);
    let ramp = at(&mask, 100, 50, 50);
    assert!(ramp > 0.0 && ramp < 2.0, "{ramp}");
    let triangle = region("select = [{ polygon = [[0, 0], [100, 0], [50, 100]] }]\nfeather = 0\noutside = 0").mask(&reference);
    assert_eq!(at(&triangle, 100, 50, 10), 3.0);
    assert_eq!(at(&triangle, 100, 5, 90), 0.0);
}

#[test]
fn ranges_select_by_luminance_hue_and_saturation_of_the_reference() {
    let mut bytes = [20u8; 3].repeat(50);
    bytes.extend([0, 0, 255].repeat(50));
    let reference = Reference::from_rgb8(100, 1, &bytes);
    let dark = region("select = [{ luminance = [0, 0.15] }]\nfeather = 0\noutside = 0").mask(&reference);
    assert_eq!((at(&dark, 100, 10, 0), at(&dark, 100, 90, 0)), (3.0, 0.0));
    let blue = region("select = [{ hue = [0.6, 0.7], saturation = [0.5, 1] }]\nfeather = 0\noutside = 0").mask(&reference);
    assert_eq!((at(&blue, 100, 10, 0), at(&blue, 100, 90, 0)), (0.0, 3.0));
    let red = region("select = [{ hue = [0.9, 0.1] }]\nfeather = 0\noutside = 0").mask(&Reference::from_rgb8(1, 1, &[255, 0, 0]));
    assert_eq!(red[0], 3.0);
}

#[test]
fn selections_combine_by_adding_subtracting_and_intersecting() {
    let reference = flat(100, 100, [128; 3]);
    let left = "{ rectangle = { from = [0, 0], to = [60, 100] } }";
    let combined = |mode: &str| region(&format!("select = [{left}, {{ mode = \"{mode}\", rectangle = {{ from = [40, 0], to = [100, 100] }} }}]\nfeather = 0\noutside = 0")).mask(&reference);
    let union = combined("add");
    assert_eq!((at(&union, 100, 10, 50), at(&union, 100, 50, 50), at(&union, 100, 90, 50)), (3.0, 3.0, 3.0));
    let cut = combined("subtract");
    assert_eq!((at(&cut, 100, 10, 50), at(&cut, 100, 50, 50), at(&cut, 100, 90, 50)), (3.0, 0.0, 0.0));
    let overlap = combined("intersect");
    assert_eq!((at(&overlap, 100, 10, 50), at(&overlap, 100, 50, 50), at(&overlap, 100, 90, 50)), (0.0, 3.0, 0.0));
}

#[test]
fn shape_and_range_within_one_selection_intersect_and_relative_shapes_scale() {
    let mut bytes = [255u8; 3].repeat(5000);
    bytes.extend([0u8; 3].repeat(5000));
    let reference = Reference::from_rgb8(100, 100, &bytes);
    let mask = region("select = [{ ellipse = { centre = [0.5, 0.5], radius = 0.4 }, luminance = [0, 0.2] }]\nrelative = true\nfeather = 0\noutside = 0").mask(&reference);
    assert_eq!(at(&mask, 100, 50, 70), 3.0);
    assert_eq!(at(&mask, 100, 50, 30), 0.0);
    assert_eq!(at(&mask, 100, 2, 70), 0.0);
}

#[test]
fn invalid_regions_report_every_problem() {
    let problems = region("select = [{ luminance = [0.5, 0.2] }, { mode = \"add\" }]\naspect = 0").validate();
    assert!(problems.iter().any(|p| p.contains("luminance")), "{problems:?}");
    assert!(problems.iter().any(|p| p.contains("needs a shape or a range")), "{problems:?}");
    assert!(problems.iter().any(|p| p.contains("aspect")), "{problems:?}");
    assert!(region("centre = [1, 1]").validate().iter().any(|p| p.contains("go together")));
    assert!(toml::from_str::<Region>("select = [{ mode = \"union\", luminance = [0, 1] }]").is_err());
}
