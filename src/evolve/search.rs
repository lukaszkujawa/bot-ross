use crate::evolve::family::Family;
use crate::fitness::Fitness;
use crate::reference::Reference;
use crate::stroke::Point;
use rand::rngs::StdRng;
use rand::{Rng, RngExt, SeedableRng};
use rayon::prelude::*;

const PARALLEL_AREA: f32 = 2000.0;

pub struct Search<'a, F: Family> {
    pub family: &'a F,
    pub candidates: usize,
    pub refinements: usize,
    pub mask: Option<&'a [f32]>,
}

impl<F: Family> Search<'_, F> {
    pub fn run(&self, start: Point, fitness: &Fitness, target: &Reference, rng: &mut impl Rng) -> (F::Candidate, i64) {
        let mut best = self.round(fitness, rng, |seed| self.family.random(start, target, &mut StdRng::seed_from_u64(seed)));
        if best.1 >= 0 {
            return best;
        }
        for _ in 0..self.refinements {
            let parent = best.0.clone();
            let challenger = self.round(fitness, rng, |seed| self.family.mutate(&parent, &mut StdRng::seed_from_u64(seed)));
            if challenger.1 < best.1 {
                best = challenger;
            }
        }
        best
    }

    fn round(&self, fitness: &Fitness, rng: &mut impl Rng, propose: impl Fn(u64) -> F::Candidate + Sync) -> (F::Candidate, i64) {
        let seeds: Vec<u64> = (0..self.candidates).map(|_| rng.random()).collect();
        let score = |seed: u64| {
            let candidate = propose(seed);
            let mut mark = self.family.proxy(&candidate, fitness);
            if let Some(mask) = self.mask {
                mark.clip(|x, y| 1.0 - mask[y * fitness.width + x]);
            }
            let delta = fitness.delta(&mark);
            (candidate, delta)
        };
        let worth_threads = self.family.largest_area() > PARALLEL_AREA;
        let best = if worth_threads {
            seeds.into_par_iter().map(score).min_by_key(|(_, delta)| *delta)
        } else {
            seeds.into_iter().map(score).min_by_key(|(_, delta)| *delta)
        };
        best.expect("at least one candidate")
    }
}
