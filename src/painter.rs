use crate::brush::Brush;
use crate::canvas::Canvas;
use crate::color::{Premultiplied, parse_hex};
use crate::evolve::progress::Progress;
use crate::evolve::shapes::Shapes;
use crate::evolve::strokes::Strokes;
use crate::evolve::{Outcome, run_layer};
use crate::layer::Layer;
use crate::painting::plan::{LayerPlan, Marks, Plan};
use crate::reference::Reference;
use crate::shape::Shape;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::error::Error;

pub fn paint(plan: &Plan, reference: &Reference, canvas: &mut Canvas, progress: &mut Progress, seed: u64) -> Result<Vec<Outcome>, Box<dyn Error>> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut outcomes = Vec::new();
    for layer in &plan.layers {
        let outcome = paint_layer(layer, canvas, reference, progress, &mut rng)?;
        println!("{:<16} done: {} placed, {} rejected after real render, {} attempts, error {:.3e}", layer.name, outcome.placed, outcome.rejected, outcome.attempts, outcome.error as f64);
        outcomes.push(outcome);
        if progress.stopped {
            break;
        }
    }
    Ok(outcomes)
}

pub fn paint_layer(plan: &LayerPlan, canvas: &mut Canvas, reference: &Reference, progress: &mut Progress, rng: &mut impl Rng) -> Result<Outcome, Box<dyn Error>> {
    match &plan.marks {
        Marks::Strokes { brush, kind } => {
            let mut brush = Brush::load(brush)?;
            let family = Strokes::new(kind, reference, brush.proxy());
            run_layer(plan, &family, |layer, stroke| brush.paint(layer, stroke), canvas, reference, progress, rng)
        }
        Marks::Shapes(kind) => {
            let paint = |layer: &mut Layer, shape: &Shape| {
                let mut mark = shape.mark(layer.width, layer.height);
                if !kind.overlap {
                    mark.clip_to(layer);
                }
                mark.composite(layer)
            };
            run_layer(plan, &Shapes::new(kind, reference), paint, canvas, reference, progress, rng)
        }
    }
}

pub fn background(spec: &str, reference: &Reference) -> Result<Premultiplied, Box<dyn Error>> {
    let rgb = match spec {
        "mean" => reference.mean(),
        "white" => [1.0, 1.0, 1.0],
        hex => parse_hex(hex).map_err(|_| format!("background must be \"mean\", \"white\" or #rrggbb, got {spec}"))?,
    };
    Ok(Premultiplied::from_rgb(rgb, 1.0))
}
