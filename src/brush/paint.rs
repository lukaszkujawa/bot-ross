use rand::{Rng, RngExt};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct Paint {
    pub load: f32,
    pub depletion: f32,
    pub unevenness: f32,
    pub skip: f32,
    pub grain: f32,
    pub fade: f32,
}

impl Paint {
    pub fn remaining(&self, progress: f32) -> f32 {
        ((self.load - self.depletion * progress) * self.fading(progress)).clamp(0.0, 1.0)
    }

    fn fading(&self, progress: f32) -> f32 {
        if self.fade <= 0.0 {
            return 1.0;
        }
        let left = ((1.0 - progress) / self.fade).clamp(0.0, 1.0);
        left * left * (3.0 - 2.0 * left)
    }

    pub fn contact(&self) -> Contact {
        Contact { touching: true }
    }
}

impl Default for Paint {
    fn default() -> Self {
        Self { load: 1.0, depletion: 0.0, unevenness: 0.0, skip: 0.0, grain: 0.0, fade: 0.0 }
    }
}

pub struct Contact {
    touching: bool,
}

impl Contact {
    pub fn next(&mut self, paint: &Paint, remaining: f32, rng: &mut impl Rng) -> f32 {
        let skip = paint.skip + (1.0 - remaining) * (1.0 - paint.skip);
        let flip = if self.touching { skip } else { 1.0 - skip } * paint.grain;
        if rng.random_bool(flip.clamp(0.0, 1.0) as f64) {
            self.touching = !self.touching;
        }
        if self.touching { 1.0 } else { 0.0 }
    }
}
