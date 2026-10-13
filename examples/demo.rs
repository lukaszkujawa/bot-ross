use bot_ross::brush::Brush;
use bot_ross::canvas::Canvas;
use bot_ross::color::{Hsv, Premultiplied};
use bot_ross::stroke::{Curve, Point, Pressure, Stroke};
use bot_ross::window::Window;
use std::error::Error;

const SIZE: usize = 1024;
const PRESETS: [&str; 5] = ["wet_oil", "dry_brush", "knife", "calligraphy", "pen"];
const MARGIN: f32 = 0.1;
const RADIUS_PER_LANE: f32 = 0.16;
const SWAY_PER_LANE: f32 = 0.3;
const HAND_PRESSURE: [f32; 5] = [0.3, 0.85, 1.0, 0.8, 0.35];

fn main() -> Result<(), Box<dyn Error>> {
    let mut canvas = Canvas::new(SIZE, SIZE, Premultiplied::WHITE);
    let layer = canvas.add_layer();
    for (index, preset) in PRESETS.iter().enumerate() {
        Brush::load(format!("brushes/{preset}.toml"))?.paint(layer, &lane_stroke(index));
    }
    Window::open("Bot Ross brushes", SIZE, SIZE)?.show_until_closed(&canvas.render())
}

fn lane_stroke(index: usize) -> Stroke {
    let lane = SIZE as f32 / (PRESETS.len() + 1) as f32;
    let y = lane * (index + 1) as f32;
    let (left, right) = (SIZE as f32 * MARGIN, SIZE as f32 * (1.0 - MARGIN));
    let at = |t: f32| left + (right - left) * t;
    let sway = lane * SWAY_PER_LANE;
    Stroke {
        path: Curve { from: Point { x: left, y }, control: [Point { x: at(0.35), y: y - sway }, Point { x: at(0.65), y: y + sway }], to: Point { x: right, y } },
        pressure: Pressure::new(HAND_PRESSURE.to_vec()),
        color: Hsv { hue: index as f32 / PRESETS.len() as f32, saturation: 0.75, value: 0.65 },
        radius: lane * RADIUS_PER_LANE,
        opacity: 1.0,
        seed: index as u64,
    }
}
