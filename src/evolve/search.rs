use crate::fitness::Fitness;
use crate::painting::stroke_type::StrokeType;
use crate::proxy::{Proxy, ribbon};
use crate::reference::Reference;
use crate::stroke::{Point, Stroke};
use rand::Rng;

pub struct Search<'a> {
    pub kind: &'a StrokeType,
    pub proxy: Proxy,
    pub candidates: usize,
    pub refinements: usize,
}

impl Search<'_> {
    pub fn run(&self, start: Point, fitness: &Fitness, reference: &Reference, rng: &mut impl Rng) -> (Stroke, i64) {
        let score = |stroke: Stroke| {
            let delta = fitness.delta(&ribbon(&stroke, self.proxy, fitness.width, fitness.height), stroke.color.to_rgb());
            (stroke, delta)
        };
        let mut best = (0..self.candidates).map(|_| score(self.kind.random(start, reference, rng))).min_by_key(|(_, delta)| *delta).expect("at least one candidate");
        for _ in 0..self.refinements {
            let challenger = (0..self.candidates).map(|_| score(self.kind.mutate(&best.0, rng))).min_by_key(|(_, delta)| *delta).expect("at least one mutant");
            if challenger.1 < best.1 {
                best = challenger;
            }
        }
        best
    }
}
