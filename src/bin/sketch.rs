use bot_ross::brush::Brush;
use bot_ross::color::{Blend, Hsv, Premultiplied, hex, parse_hex};
use bot_ross::output::save_png;
use bot_ross::painting::stroke_type::PressureShape;
use bot_ross::sketch::{Settings, Sketch, overlay};
use bot_ross::stroke::Point;
use bot_ross::window::Window;
use clap::Parser;
use minifb::Key;
use std::error::Error;
use std::path::PathBuf;

const PALETTE: [&str; 6] = ["#1f3a5f", "#7f8c6a", "#c8a85a", "#8b3a3a", "#5c3d7a", "#222222"];
const RADIUS_STEP: f32 = 1.25;
const OPACITY_STEP: f32 = 0.1;

#[derive(Parser, Debug)]
#[command(name = "sketch", about = "Draws strokes by hand with a brush preset, to try brushes without a reference")]
struct Cli {
    #[arg(long)]
    brush: PathBuf,
    #[arg(long, default_value_t = 1000)]
    size: usize,
    #[arg(long, default_value = "sketch.png")]
    out: PathBuf,
    #[arg(long, default_value = "normal")]
    blend: String,
    #[arg(long, default_value_t = 20.0)]
    radius: f32,
    #[arg(long, default_value_t = 0.6)]
    opacity: f32,
    #[arg(long, default_value = PALETTE[0])]
    colour: String,
    #[arg(long, default_value = "#ffffff")]
    background: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let blend = match cli.blend.as_str() {
        "normal" => Blend::Normal,
        "multiply" => Blend::Multiply,
        other => return Err(format!("blend must be normal or multiply, got {other}").into()),
    };
    let settings = Settings { color: Hsv::from_rgb(parse_hex(&cli.colour)?), radius: cli.radius, opacity: cli.opacity, pressure: PressureShape::Hand };
    let brush = Brush::load(&cli.brush)?;
    let mut sketch = Sketch::new(brush, cli.size, cli.size, Premultiplied::from_rgb(parse_hex(&cli.background)?, 1.0), blend, settings);
    let mut window = Window::open(&format!("Bot Ross sketch: {}", cli.brush.display()), cli.size, cli.size)?;
    println!("drag to draw; [ ] radius, - = opacity, C colour, P pressure, X clear, S save, Esc save and quit");
    describe(&sketch.settings);
    let mut drag: Vec<Point> = Vec::new();
    let mut palette = 0;
    loop {
        let mut image = sketch.render();
        overlay(&mut image, &drag, 0x808080);
        if !window.present(&image)? {
            break;
        }
        match (window.mouse_down(), window.mouse()) {
            (true, Some(point)) => drag.push(point),
            _ if !drag.is_empty() => {
                sketch.draw(&drag);
                drag.clear();
            }
            _ => {}
        }
        let settings = &mut sketch.settings;
        let changed = if window.pressed(Key::RightBracket) {
            settings.radius *= RADIUS_STEP;
            true
        } else if window.pressed(Key::LeftBracket) {
            settings.radius = (settings.radius / RADIUS_STEP).max(0.3);
            true
        } else if window.pressed(Key::Equal) {
            settings.opacity = (settings.opacity + OPACITY_STEP).min(1.0);
            true
        } else if window.pressed(Key::Minus) {
            settings.opacity = (settings.opacity - OPACITY_STEP).max(OPACITY_STEP);
            true
        } else if window.pressed(Key::C) {
            palette = (palette + 1) % PALETTE.len();
            settings.color = Hsv::from_rgb(parse_hex(PALETTE[palette])?);
            true
        } else if window.pressed(Key::P) {
            settings.pressure = match settings.pressure {
                PressureShape::Hand => PressureShape::Flat,
                PressureShape::Flat => PressureShape::Fading,
                PressureShape::Fading => PressureShape::Hand,
            };
            true
        } else {
            false
        };
        if changed {
            describe(&sketch.settings);
        }
        if window.pressed(Key::X) {
            sketch.clear();
        }
        if window.pressed(Key::S) {
            save(&sketch, &cli.out)?;
        }
    }
    save(&sketch, &cli.out)
}

fn save(sketch: &Sketch, out: &PathBuf) -> Result<(), Box<dyn Error>> {
    save_png(&sketch.render(), out)?;
    println!("saved {}", out.display());
    Ok(())
}

fn describe(settings: &Settings) {
    println!("radius {:.1}  opacity {:.1}  colour {}  pressure {:?}", settings.radius, settings.opacity, hex(settings.color.to_rgb()), settings.pressure);
}
