use crate::fitness::Fitness;
use crate::layer::tiles::Bounds;
use crate::painting::config::PlacementMode;
use crate::stroke::Point;
use rand::{Rng, RngExt};

const EDGE_FLOOR: f64 = 0.05;

pub struct Placement {
    cell: usize,
    cols: usize,
    mode: PlacementMode,
    edges: Option<Vec<f32>>,
    weights: Vec<f64>,
}

impl Placement {
    pub fn new(fitness: &Fitness, mode: PlacementMode, edges: Option<Vec<f32>>, cell: usize) -> Self {
        let (cols, rows) = (fitness.width.div_ceil(cell), fitness.height.div_ceil(cell));
        let mut placement = Self { cell, cols, mode, edges, weights: vec![0.0; cols * rows] };
        placement.refresh(fitness, Bounds::whole(fitness.width, fitness.height));
        placement
    }

    pub fn refresh(&mut self, fitness: &Fitness, bounds: Bounds) {
        if bounds.is_empty() {
            return;
        }
        for row in bounds.top / self.cell..bounds.bottom.div_ceil(self.cell) {
            for col in bounds.left / self.cell..bounds.right.div_ceil(self.cell) {
                self.weights[row * self.cols + col] = self.weigh(fitness, col, row);
            }
        }
    }

    pub fn pick(&self, rng: &mut impl Rng) -> Point {
        let total: f64 = self.weights.iter().map(|w| w * w).sum();
        let mut target = rng.random_range(0.0..total.max(f64::MIN_POSITIVE));
        let mut chosen = self.weights.len() - 1;
        for (index, weight) in self.weights.iter().enumerate() {
            target -= weight * weight;
            if target < 0.0 {
                chosen = index;
                break;
            }
        }
        let (col, row) = (chosen % self.cols, chosen / self.cols);
        Point {
            x: rng.random_range(0.0..self.cell as f32) + (col * self.cell) as f32,
            y: rng.random_range(0.0..self.cell as f32) + (row * self.cell) as f32,
        }
    }

    fn weigh(&self, fitness: &Fitness, col: usize, row: usize) -> f64 {
        if self.mode == PlacementMode::Random {
            return 1.0;
        }
        let (mut error, mut edge, mut pixels) = (0.0f64, 0.0f64, 0.0f64);
        for y in row * self.cell..((row + 1) * self.cell).min(fitness.height) {
            for x in col * self.cell..((col + 1) * self.cell).min(fitness.width) {
                error += fitness.error_at(x, y) as f64;
                edge += self.edges.as_ref().map_or(0.0, |edges| edges[y * fitness.width + x] as f64);
                pixels += 1.0;
            }
        }
        match self.mode {
            PlacementMode::Edges => error * (EDGE_FLOOR + edge / pixels.max(1.0)),
            _ => error,
        }
    }
}
