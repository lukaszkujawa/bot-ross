pub mod family;
pub mod progress;
pub mod search;
pub mod shapes;
pub mod strokes;

use crate::bounds::Bounds;
use crate::canvas::Canvas;
use crate::evolve::family::Family;
use crate::evolve::progress::Progress;
use crate::evolve::search::Search;
use crate::fitness::Fitness;
use crate::layer::Layer;
use crate::painting::config::PlacementMode;
use crate::painting::plan::LayerPlan;
use crate::placement::Placement;
use crate::reference::Reference;
use rand::{Rng, RngExt};
use std::error::Error;
use std::time::Instant;

const CELL: usize = 32;

#[derive(Debug, Default, PartialEq)]
pub struct Outcome {
    pub placed: usize,
    pub rejected: usize,
    pub attempts: usize,
    pub error: u64,
}

pub fn run_layer<F: Family>(
    plan: &LayerPlan,
    family: &F,
    mut paint: impl FnMut(&mut Layer, &F::Candidate) -> Bounds,
    canvas: &mut Canvas,
    reference: &Reference,
    progress: &mut Progress,
    rng: &mut impl Rng,
) -> Result<Outcome, Box<dyn Error>> {
    let (width, height) = (canvas.width, canvas.height);
    let target = plan.reference.prepare(reference);
    let mut fitness = Fitness::new(&target, canvas.composite());
    let edges = (plan.placement == PlacementMode::Edges || plan.reference.edges > 0.0).then(|| reference.edges());
    if plan.region.is_some() || plan.reference.edges > 0.0 {
        fitness.weigh(|x, y| {
            let region = plan.region.map_or(1.0, |region| region.weight_at(x as f32, y as f32));
            let contour = edges.as_ref().map_or(0.0, |edges| edges[y * width + x]);
            region * (1.0 + plan.reference.edges * contour)
        });
    }
    let mut placement = Placement::new(&fitness, plan.placement, edges, CELL);
    let searcher = Search { family, candidates: plan.candidates.max(1), refinements: plan.refinements };
    let layer = canvas.add_layer();
    let mut outcome = Outcome::default();
    let started = Instant::now();
    while outcome.placed < plan.count && outcome.attempts < plan.count * plan.patience.max(1) && !progress.stopped {
        outcome.attempts += 1;
        let (candidate, delta) = searcher.run(placement.pick(rng), &fitness, &target, rng);
        let forgiven = plan.accident > 0.0 && rng.random_range(0.0..1.0) < plan.accident;
        if delta >= 0 && !forgiven {
            continue;
        }
        let margin = family.reach(&candidate, width, height);
        let snapshot = layer.read_region(margin);
        let before = fitness.total();
        let touched = paint(layer, &candidate).union(margin);
        fitness.refresh(layer, touched);
        if fitness.total() >= before && !forgiven {
            layer.write_region(margin, &snapshot);
            fitness.refresh(layer, touched);
            outcome.rejected += 1;
            continue;
        }
        outcome.placed += 1;
        placement.refresh(&fitness, touched);
        progress.placed(&plan.name, outcome.placed, plan.count, &fitness, started)?;
    }
    progress.show(&fitness)?;
    outcome.error = fitness.total();
    Ok(outcome)
}
