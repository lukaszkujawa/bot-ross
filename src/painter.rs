use crate::canvas::Canvas;
use crate::color::Premultiplied;
use crate::evolve::progress::Progress;
use crate::evolve::{Outcome, run_layer};
use crate::painting::plan::Plan;
use crate::reference::Reference;
use rand::SeedableRng;
use rand::rngs::StdRng;
use std::error::Error;

pub fn paint(plan: &Plan, reference: &Reference, canvas: &mut Canvas, progress: &mut Progress, seed: u64) -> Result<Vec<Outcome>, Box<dyn Error>> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut outcomes = Vec::new();
    for layer in &plan.layers {
        let outcome = run_layer(layer, plan.search, canvas, reference, progress, &mut rng)?;
        println!("{:<16} done: {} placed, {} rejected after real render, {} attempts, error {:.3e}", layer.name, outcome.placed, outcome.rejected, outcome.attempts, outcome.error as f64);
        outcomes.push(outcome);
        if progress.stopped {
            break;
        }
    }
    Ok(outcomes)
}

pub fn background(spec: &str, reference: &Reference) -> Result<Premultiplied, Box<dyn Error>> {
    let rgb = match spec {
        "mean" => reference.mean(),
        "white" => [1.0, 1.0, 1.0],
        hex => {
            let digits = hex.strip_prefix('#').ok_or_else(|| format!("background must be \"mean\", \"white\" or #rrggbb, got {spec}"))?;
            let channel = |index: usize| u8::from_str_radix(digits.get(index..index + 2).unwrap_or("zz"), 16).map(|v| v as f32 / 255.0);
            [channel(0)?, channel(2)?, channel(4)?]
        }
    };
    Ok(Premultiplied::from_rgb(rgb, 1.0))
}
