pub mod progress;
pub mod search;

use crate::brush::Brush;
use crate::canvas::Canvas;
use crate::evolve::progress::Progress;
use crate::evolve::search::Search;
use crate::fitness::Fitness;
use crate::layer::tiles::Bounds;
use crate::painting::config::{PlacementMode, SearchConfig};
use crate::painting::plan::LayerPlan;
use crate::placement::Placement;
use crate::reference::Reference;
use rand::Rng;
use std::error::Error;
use std::time::Instant;

const CELL: usize = 32;
const MARGIN_RADII: f32 = 2.0;

#[derive(Debug, Default, PartialEq)]
pub struct Outcome {
    pub placed: usize,
    pub rejected: usize,
    pub attempts: usize,
    pub error: u64,
}

pub fn run_layer(plan: &LayerPlan, search: SearchConfig, canvas: &mut Canvas, reference: &Reference, progress: &mut Progress, rng: &mut impl Rng) -> Result<Outcome, Box<dyn Error>> {
    let mut brush = Brush::load(&plan.brush)?;
    let (width, height) = (canvas.width, canvas.height);
    let mut fitness = Fitness::new(reference, canvas.composite());
    if let Some(region) = plan.region {
        fitness.weigh(|x, y| region.weight_at(x as f32, y as f32));
    }
    let edges = (plan.placement == PlacementMode::Edges).then(|| reference.edges());
    let mut placement = Placement::new(&fitness, plan.placement, edges, CELL);
    let searcher = Search { kind: &plan.stroke, proxy: brush.proxy(), candidates: plan.candidates.max(1), refinements: plan.refinements };
    let layer = canvas.add_layer();
    let mut outcome = Outcome::default();
    let started = Instant::now();
    while outcome.placed < plan.count && outcome.attempts < plan.count * search.patience.max(1) && !progress.stopped {
        outcome.attempts += 1;
        let (stroke, delta) = searcher.run(placement.pick(rng), &fitness, reference, rng);
        if delta >= 0 {
            continue;
        }
        let [xs, ys] = stroke.path.extent();
        let margin = Bounds::enclosing(xs, ys, stroke.radius * MARGIN_RADII + 4.0, width, height);
        let snapshot = layer.read_region(margin);
        let before = fitness.total();
        let touched = brush.paint(layer, &stroke).union(margin);
        fitness.refresh(layer, touched);
        if fitness.total() >= before {
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
