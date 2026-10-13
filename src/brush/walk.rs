use rand::{Rng, RngExt};

pub struct Walk {
    position: f32,
    step: f32,
}

impl Walk {
    pub fn new(extent: f32, steps: f32) -> Self {
        Self { position: 0.0, step: extent / steps.sqrt() }
    }

    pub fn next(&mut self, rng: &mut impl Rng) -> f32 {
        self.position += rng.random_range(-1.0..=1.0) * self.step;
        self.position
    }
}
