use crate::bounds::Bounds;
use crate::evolve::family::Family;
use crate::fitness::Fitness;
use crate::flow::Flow;
use crate::mark::Mark;
use crate::painting::stroke_type::StrokeType;
use crate::proxy::{Proxy, ribbon};
use crate::reference::Reference;
use crate::stroke::{Point, Stroke};
use rand::Rng;

const MARGIN_RADII: f32 = 2.0;

pub struct Strokes<'a> {
    kind: &'a StrokeType,
    flow: Flow,
    proxy: Proxy,
}

impl<'a> Strokes<'a> {
    pub fn new(kind: &'a StrokeType, reference: &Reference, proxy: Proxy) -> Self {
        let (width, height) = (reference.width, reference.height);
        let flow = match kind.aim {
            Some(centre) => Flow::radial(width, height, centre),
            None if kind.follow > 0.0 => Flow::of(reference),
            None => Flow::flat(width, height),
        };
        Self { kind, flow, proxy }
    }
}

impl Family for Strokes<'_> {
    type Candidate = Stroke;

    fn random(&self, start: Point, target: &Reference, rng: &mut impl Rng) -> Stroke {
        self.kind.random(start, target, &self.flow, rng)
    }

    fn mutate(&self, parent: &Stroke, rng: &mut impl Rng) -> Stroke {
        self.kind.mutate(parent, rng)
    }

    fn proxy(&self, stroke: &Stroke, fitness: &Fitness) -> Mark {
        ribbon(stroke, self.proxy, fitness.width, fitness.height)
    }

    fn reach(&self, stroke: &Stroke, width: usize, height: usize) -> Bounds {
        let [xs, ys] = stroke.path.extent();
        Bounds::enclosing(xs, ys, stroke.radius * MARGIN_RADII + 4.0, width, height)
    }

    fn largest_area(&self) -> f32 {
        self.kind.radius[1] * self.kind.length[1]
    }
}
