use bot_ross::color::{Hsv, Premultiplied};
use bot_ross::layer::Layer;
use bot_ross::shape::{Shape, Spoke};
use bot_ross::shape::fill::Fill;
use bot_ross::stroke::Point;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};

fn spokes(reaches: &[f32]) -> Vec<Spoke> {
    reaches.iter().enumerate().map(|(index, reach)| Spoke { angle: index as f32 * FRAC_PI_2, reach: *reach }).collect()
}

fn filled(fill: Fill<f32>) -> Shape {
    Shape { fill, ..square() }
}

fn square() -> Shape {
    Shape { centre: Point { x: 50.5, y: 50.5 }, radius: 20.0 * std::f32::consts::SQRT_2, rotation: FRAC_PI_4, aspect: 1.0, soft: 0.0, spokes: spokes(&[1.0; 4]), fill: Fill::Solid, color: Hsv { hue: 0.0, saturation: 1.0, value: 1.0 }, opacity: 1.0 }
}

#[test]
fn vertices_sit_on_the_circumcircle_at_even_angles() {
    let vertices = square().vertices();
    assert_eq!(vertices.len(), 4);
    for vertex in &vertices {
        assert!((vertex.distance(square().centre) - square().radius).abs() < 1e-3);
        assert!(((vertex.x - 50.5).abs() - 20.0).abs() < 1e-3 && ((vertex.y - 50.5).abs() - 20.0).abs() < 1e-3, "{vertex:?}");
    }
}

#[test]
fn spoke_angles_place_vertices_anywhere_around_the_centre() {
    let lopsided = Shape { spokes: vec![Spoke { angle: 0.0, reach: 1.0 }, Spoke { angle: 0.3, reach: 0.4 }, Spoke { angle: 3.0, reach: 1.0 }], rotation: 0.0, ..square() };
    let vertices = lopsided.vertices();
    assert!((vertices[0].x - 50.5 - lopsided.radius).abs() < 1e-3);
    assert!(vertices[1].distance(lopsided.centre) < vertices[0].distance(lopsided.centre) / 2.0);
    assert!(vertices[2].x < 50.5 && vertices[2].y > 50.5);
}

#[test]
fn aspect_flattens_the_shape_across_its_rotation() {
    let flake = Shape { aspect: 0.2, rotation: 0.3, ..square() };
    let (sin, cos) = flake.rotation.sin_cos();
    let (mut along, mut across) = (0.0f32, 0.0f32);
    for vertex in flake.vertices() {
        let (dx, dy) = (vertex.x - flake.centre.x, vertex.y - flake.centre.y);
        along = along.max((dx * cos + dy * sin).abs());
        across = across.max((dy * cos - dx * sin).abs());
    }
    assert!((along - flake.radius).abs() < 1e-3 && (across - flake.radius * 0.2).abs() < 1e-3, "{along} {across}");
}

#[test]
fn soft_edges_fade_coverage_across_the_boundary() {
    let soft = Shape { soft: 20.0, ..square() };
    let mark = soft.mark(100, 100);
    assert_eq!(mark.coverage_at(50, 50), 1.0);
    assert!((mark.coverage_at(70, 50) - 0.5).abs() < 0.05, "{}", mark.coverage_at(70, 50));
    assert!(mark.coverage_at(65, 50) > 0.6 && mark.coverage_at(65, 50) < 0.9);
    assert!(mark.coverage_at(75, 50) > 0.1 && mark.coverage_at(75, 50) < 0.4);
    assert_eq!(mark.coverage_at(85, 50), 0.0);
    assert!(mark.bounds().right >= 80);
}

#[test]
fn solid_fill_covers_the_inside_and_nothing_beyond_the_bounds() {
    let mark = square().mark(100, 100);
    assert_eq!(mark.coverage_at(50, 50), 1.0);
    assert_eq!(mark.coverage_at(31, 31), 1.0);
    assert_eq!(mark.coverage_at(25, 50), 0.0);
    assert_eq!(mark.coverage_at(50, 80), 0.0);
    assert_eq!(mark.coverage_at(69, 50), 1.0);
    assert_eq!(mark.coverage_at(70, 50), 0.5);
    assert_eq!(mark.rgb_at(50, 50), [1.0, 0.0, 0.0]);
    let bounds = mark.bounds();
    assert!(bounds.left >= 15 && bounds.right <= 85 && bounds.top >= 15 && bounds.bottom <= 85, "{bounds:?}");
}

#[test]
fn outline_covers_the_edge_only() {
    let mark = filled(Fill::Outline { width: 2.0 }).mark(100, 100);
    assert_eq!(mark.coverage_at(50, 50), 0.0);
    assert_eq!(mark.coverage_at(70, 50), 1.0);
    assert_eq!(mark.coverage_at(30, 50), 1.0);
    assert_eq!(mark.coverage_at(50, 75), 0.0);
}

#[test]
fn hatch_leaves_gaps_between_lines_at_the_given_spacing() {
    let mark = filled(Fill::Hatch { spacing: 8.0, width: 1.0, angle: Some(0.0), cross: false }).mark(100, 100);
    let row = |y: usize| mark.coverage_at(50, y);
    assert_eq!(row(50), 1.0);
    assert_eq!(row(58), 1.0);
    assert_eq!(row(54), 0.0);
    let crossed = filled(Fill::Hatch { spacing: 8.0, width: 1.0, angle: Some(0.0), cross: true }).mark(100, 100);
    assert_eq!(crossed.coverage_at(50, 54), 1.0);
    assert_eq!(crossed.coverage_at(54, 54), 0.0);
}

#[test]
fn dots_cover_grid_points_and_not_the_space_between() {
    let mark = filled(Fill::Dots { spacing: 10.0, radius: 1.0 }).mark(100, 100);
    assert_eq!(mark.coverage_at(50, 50), 1.0);
    assert_eq!(mark.coverage_at(55, 50), 0.0);
    assert_eq!(mark.coverage_at(55, 55), 0.0);
}

#[test]
fn opacity_scales_coverage_and_irregular_spokes_pull_vertices_in() {
    let faint = Shape { opacity: 0.25, ..square() };
    assert_eq!(faint.mark(100, 100).coverage_at(50, 50), 0.25);
    let dented = Shape { spokes: spokes(&[0.5, 1.0, 1.0, 1.0]), ..square() };
    assert!((dented.vertices()[0].distance(dented.centre) - dented.radius * 0.5).abs() < 1e-3);
    assert_eq!(dented.mark(100, 100).coverage_at(68, 68), 0.0);
}

#[test]
fn clipping_keeps_a_shape_off_pixels_the_layer_already_holds() {
    let mut layer = Layer::new(100, 100);
    square().mark(100, 100).composite(&mut layer);
    let before = layer.pixels();
    let shifted = Shape { centre: Point { x: 70.5, y: 50.5 }, color: Hsv { hue: 0.6, saturation: 1.0, value: 1.0 }, ..square() };
    let mut mark = shifted.mark(100, 100);
    mark.clip_to(&layer);
    assert_eq!(mark.coverage_at(60, 50), 0.0);
    assert_eq!(mark.coverage_at(65, 60), 0.0);
    assert_eq!(mark.coverage_at(75, 60), 1.0);
    assert_eq!(mark.coverage_at(85, 50), 1.0);
    mark.composite(&mut layer);
    let after = layer.pixels();
    assert_eq!(after[50 * 100 + 60], before[50 * 100 + 60]);
    assert_eq!(after[50 * 100 + 85], Premultiplied::from_rgb([0.0, 0.4, 1.0], 1.0));
}
