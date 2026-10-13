use crate::fitness::Fitness;
use crate::painting::stroke_type::StrokeType;
use crate::proxy::{Proxy, ribbon};
use crate::reference::Reference;
use crate::stroke::{Point, Stroke};
use rand::rngs::StdRng;
use rand::{Rng, RngExt, SeedableRng};
use rayon::prelude::*;

const PARALLEL_AREA: f32 = 2000.0;

pub struct Search<'a> {
    pub kind: &'a StrokeType,
    pub proxy: Proxy,
    pub candidates: usize,
    pub refinements: usize,
}

impl Search<'_> {
    pub fn run(&self, start: Point, fitness: &Fitness, reference: &Reference, rng: &mut impl Rng) -> (Stroke, i64) {
        let mut best = self.round(fitness, rng, |seed| self.kind.random(start, reference, &mut StdRng::seed_from_u64(seed)));
        if best.1 >= 0 {
            return best;
        }
        for _ in 0..self.refinements {
            let parent = best.0.clone();
            let challenger = self.round(fitness, rng, |seed| self.kind.mutate(&parent, &mut StdRng::seed_from_u64(seed)));
            if challenger.1 < best.1 {
                best = challenger;
            }
        }
        best
    }

    fn round(&self, fitness: &Fitness, rng: &mut impl Rng, propose: impl Fn(u64) -> Stroke + Sync) -> (Stroke, i64) {
        let seeds: Vec<u64> = (0..self.candidates).map(|_| rng.random()).collect();
        let score = |seed: u64| {
            let stroke = propose(seed);
            let delta = fitness.delta(&ribbon(&stroke, self.proxy, fitness.width, fitness.height), stroke.color.to_rgb());
            (stroke, delta)
        };
        let worth_threads = self.kind.radius[1] * self.kind.length[1] > PARALLEL_AREA;
        let best = if worth_threads {
            seeds.into_par_iter().map(score).min_by_key(|(_, delta)| *delta)
        } else {
            seeds.into_iter().map(score).min_by_key(|(_, delta)| *delta)
        };
        best.expect("at least one candidate")
    }
}
