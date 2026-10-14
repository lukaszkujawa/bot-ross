use crate::bounds::Bounds;
use crate::evolve::family::Family;
use crate::fitness::Fitness;
use crate::flow::Flow;
use crate::lattice::Grain;
use crate::mark::Mark;
use crate::painting::shape_type::ShapeType;
use crate::reference::Reference;
use crate::shape::Shape;
use crate::stroke::Point;
use rand::Rng;

pub struct Shapes<'a> {
    kind: &'a ShapeType,
    flow: Flow,
}

impl<'a> Shapes<'a> {
    pub fn new(kind: &'a ShapeType, reference: &Reference) -> Self {
        let flow = if kind.follow > 0.0 { Flow::of(reference) } else { Flow::flat(reference.width, reference.height) };
        Self { kind, flow }
    }
}

impl Family for Shapes<'_> {
    type Candidate = Shape;

    fn random(&self, start: Point, target: &Reference, rng: &mut impl Rng) -> Shape {
        self.kind.random(start, target, &self.flow, rng)
    }

    fn mutate(&self, parent: &Shape, rng: &mut impl Rng) -> Shape {
        self.kind.mutate(parent, rng)
    }

    fn proxy(&self, shape: &Shape, fitness: &Fitness) -> Mark {
        let mut mark = shape.mark(fitness.width, fitness.height);
        if !self.kind.overlap {
            mark.clip(|x, y| fitness.occupied(x, y));
        }
        mark
    }

    fn reach(&self, shape: &Shape, width: usize, height: usize) -> Bounds {
        shape.bounds(width, height)
    }

    fn largest_area(&self) -> f32 {
        self.kind.radius[1] * self.kind.radius[1]
    }

    fn grain(&self, spacing: f32) -> Grain {
        Grain { angle: 0.0, along: Some(spacing) }
    }

    fn passing(&self, point: Point, target: &Reference, rng: &mut impl Rng) -> Shape {
        self.random(point, target, rng)
    }
}
