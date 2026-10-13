use bot_ross::brush::bristles::Sample;
use bot_ross::canvas::Canvas;
use bot_ross::color::{Hsv, Premultiplied};
use bot_ross::layer::Layer;
use bot_ross::layer::tiles::Bounds;
use bot_ross::mypaint::engine::Engine;
use bot_ross::stroke::Point;

fn diagonal() -> Vec<Sample> {
    let red = Hsv { hue: 0.0, saturation: 1.0, value: 1.0 };
    [10.0, 50.0, 90.0].map(|v| Sample { point: Point { x: v, y: v }, radius: 5.0, opacity: 1.0, pressure: 1.0, color: red }).to_vec()
}

fn red_engine() -> Engine {
    Engine::with_defaults()
}

#[test]
fn new_layer_is_transparent() {
    let pixels = Layer::new(100, 70).pixels();
    assert_eq!(pixels.len(), 100 * 70);
    assert!(pixels.iter().all(|pixel| *pixel == Premultiplied::TRANSPARENT));
}

#[test]
fn traced_path_marks_pixels_along_it_only() {
    let mut layer = Layer::new(100, 100);
    layer.trace(&mut red_engine(), &diagonal());
    let pixels = layer.pixels();
    assert!(pixels[50 * 100 + 50].alpha > 0);
    assert_eq!(pixels[5 * 100 + 95], Premultiplied::TRANSPARENT);
}

#[test]
fn empty_canvas_renders_white() {
    assert_eq!(Canvas::new(8, 4, Premultiplied::WHITE).render().argb, vec![0x00ffffff; 32]);
}

#[test]
fn traced_path_on_canvas_turns_pixels_red() {
    let mut canvas = Canvas::new(100, 100, Premultiplied::WHITE);
    canvas.add_layer().trace(&mut red_engine(), &diagonal());
    let centre = canvas.render().argb[50 * 100 + 50];
    assert!(centre >> 16 > centre & 0xff);
}

#[test]
fn blend_colours_covered_pixels_only_and_average_reads_them_back() {
    let mut layer = Layer::new(100, 100);
    let bounds = Bounds::around(50.0, 50.0, 10.0, 100, 100);
    let blue = Premultiplied::new(0, 0, 32768, 32768);
    layer.blend(bounds, |x, y| if x.abs_diff(50) < 5 && y.abs_diff(50) < 5 { blue } else { Premultiplied::TRANSPARENT });
    let pixels = layer.pixels();
    assert_eq!(pixels[50 * 100 + 50], Premultiplied::new(0, 0, 32768, 32768));
    assert_eq!(pixels[50 * 100 + 60], Premultiplied::TRANSPARENT);
    assert_eq!(layer.average(Point { x: 50.0, y: 50.0 }, 2.0), Premultiplied::new(0, 0, 32768, 32768));
    assert!(layer.average(Point { x: 50.0, y: 50.0 }, 20.0).alpha < 16384);
}

#[test]
fn trace_reports_the_painted_bounds_and_regions_round_trip() {
    let mut layer = Layer::new(100, 100);
    let bounds = layer.trace(&mut red_engine(), &diagonal());
    assert!(bounds.left <= 5 && bounds.top <= 5 && bounds.right >= 95 && bounds.bottom >= 95, "{bounds:?}");
    assert!(Layer::new(10, 10).trace(&mut red_engine(), &[]).is_empty());
    let region = Bounds::around(50.0, 50.0, 8.0, 100, 100);
    let snapshot = layer.read_region(region);
    assert_eq!(snapshot.len(), region.area());
    layer.blend(region, |_, _| Premultiplied::WHITE);
    assert_eq!(layer.pixels()[50 * 100 + 50], Premultiplied::WHITE);
    layer.write_region(region, &snapshot);
    assert_eq!(layer.read_region(region), snapshot);
    assert!(layer.pixels()[50 * 100 + 50].red > 0);
}
