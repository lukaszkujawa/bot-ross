use crate::bounds::Bounds;
use crate::fitness::Fitness;
use crate::mark::Mark;
use crate::reference::Reference;
use crate::stroke::Point;
use rand::Rng;

pub trait Family: Sync {
    type Candidate: Clone + Send + Sync;

    fn random(&self, start: Point, target: &Reference, rng: &mut impl Rng) -> Self::Candidate;
    fn mutate(&self, parent: &Self::Candidate, rng: &mut impl Rng) -> Self::Candidate;
    fn proxy(&self, candidate: &Self::Candidate, fitness: &Fitness) -> Mark;
    fn reach(&self, candidate: &Self::Candidate, width: usize, height: usize) -> Bounds;
    fn largest_area(&self) -> f32;
}
