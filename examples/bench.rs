use bot_ross::brush::Brush;
use bot_ross::canvas::Canvas;
use bot_ross::color::{Hsv, Premultiplied};
use bot_ross::stroke::{Curve, Point, Pressure, Stroke};
use rand::rngs::StdRng;
use rand::{Rng, RngExt, SeedableRng};
use std::error::Error;
use std::time::Instant;

const SIZE: usize = 1024;
const STROKES: usize = 1000;
const PRESETS: [&str; 5] = ["wet_oil", "dry_brush", "knife", "calligraphy", "pen"];

fn main() -> Result<(), Box<dyn Error>> {
    println!("{:<14}{:>10}{:>14}", "brush", "total s", "per stroke ms");
    for preset in PRESETS {
        let mut brush = Brush::load(format!("brushes/{preset}.toml"))?;
        let mut canvas = Canvas::new(SIZE, SIZE, Premultiplied::WHITE);
        let layer = canvas.add_layer();
        let mut rng = StdRng::seed_from_u64(1);
        let started = Instant::now();
        for seed in 0..STROKES as u64 {
            brush.paint(layer, &random_stroke(&mut rng, seed));
        }
        let elapsed = started.elapsed().as_secs_f64();
        println!("{preset:<14}{elapsed:>10.2}{:>14.2}", elapsed * 1000.0 / STROKES as f64);
    }
    Ok(())
}

fn random_stroke(rng: &mut impl Rng, seed: u64) -> Stroke {
    let mut point = || Point { x: rng.random_range(0.0..SIZE as f32), y: rng.random_range(0.0..SIZE as f32) };
    let (from, to) = (point(), point());
    Stroke {
        path: Curve { from, control: [point(), point()], to },
        pressure: Pressure::new(vec![0.3, 0.9, 1.0, 0.8, 0.3]),
        color: Hsv { hue: rng.random(), saturation: 0.8, value: 0.7 },
        radius: rng.random_range(10.0..40.0),
        opacity: 1.0,
        seed,
    }
}
