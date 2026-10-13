pub mod progress;
pub mod search;

use crate::brush::Brush;
use crate::canvas::Canvas;
use crate::evolve::progress::Progress;
use crate::evolve::search::Search;
use crate::fitness::Fitness;
use crate::flow::Flow;
use crate::bounds::Bounds;
use crate::painting::config::{PlacementMode, SearchConfig};
use crate::painting::plan::LayerPlan;
use crate::placement::Placement;
use crate::reference::Reference;
use rand::{Rng, RngExt};
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
    let flow = match plan.stroke.aim {
        Some(centre) => Flow::radial(width, height, centre),
        None if plan.stroke.follow > 0.0 => Flow::of(reference),
        None => Flow::flat(width, height),
    };
    let searcher = Search { kind: &plan.stroke, flow: &flow, proxy: brush.proxy(), candidates: plan.candidates.max(1), refinements: plan.refinements };
    let layer = canvas.add_layer();
    let mut outcome = Outcome::default();
    let started = Instant::now();
    while outcome.placed < plan.count && outcome.attempts < plan.count * search.patience.max(1) && !progress.stopped {
        outcome.attempts += 1;
        let (stroke, delta) = searcher.run(placement.pick(rng), &fitness, &target, rng);
        let forgiven = plan.accident > 0.0 && rng.random_range(0.0..1.0) < plan.accident;
        if delta >= 0 && !forgiven {
            continue;
        }
        let [xs, ys] = stroke.path.extent();
        let margin = Bounds::enclosing(xs, ys, stroke.radius * MARGIN_RADII + 4.0, width, height);
        let snapshot = layer.read_region(margin);
        let before = fitness.total();
        let touched = brush.paint(layer, &stroke).union(margin);
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
