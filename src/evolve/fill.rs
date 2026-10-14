use crate::bounds::Bounds;
use crate::evolve::Outcome;
use crate::evolve::family::Family;
use crate::evolve::progress::Progress;
use crate::fitness::Fitness;
use crate::lattice::lattice;
use crate::layer::Layer;
use crate::painting::plan::LayerPlan;
use crate::reference::Reference;
use crate::stroke::Point;
use rand::Rng;
use std::error::Error;
use std::time::Instant;

pub fn run<F: Family>(
    plan: &LayerPlan,
    family: &F,
    mut paint: impl FnMut(&mut Layer, &F::Candidate) -> Bounds,
    layer: &mut Layer,
    fitness: &mut Fitness,
    palette: &Reference,
    coverage: &[f32],
    progress: &mut Progress,
    rng: &mut impl Rng,
) -> Result<Outcome, Box<dyn Error>> {
    let (width, height) = (fitness.width, fitness.height);
    let covered = |point: Point| coverage[(point.y as usize).min(height - 1) * width + (point.x as usize).min(width - 1)];
    let mut points = lattice(width, height, plan.spacing, family.grain(plan.spacing), plan.scatter, covered, rng);
    if plan.count > 0 {
        points.truncate(plan.count);
    }
    let total = points.len();
    let mut outcome = Outcome::default();
    let started = Instant::now();
    for start in points {
        if progress.stopped {
            break;
        }
        let candidate = family.random(start, palette, rng);
        let touched = paint(layer, &candidate).union(family.reach(&candidate, width, height));
        fitness.refresh(layer, touched);
        outcome.placed += 1;
        outcome.attempts += 1;
        progress.placed(&plan.name, outcome.placed, total, fitness, started)?;
    }
    progress.show(fitness)?;
    outcome.error = fitness.total();
    Ok(outcome)
}
