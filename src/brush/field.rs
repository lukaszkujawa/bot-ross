pub struct Field {
    seed: u64,
    cell: [f32; 2],
}

impl Field {
    pub fn new(seed: u64, cell: [f32; 2]) -> Self {
        Self { seed, cell }
    }

    pub fn at(&self, x: f32, y: f32) -> f32 {
        let (x, y) = (x / self.cell[0], y / self.cell[1]);
        [(1.0, 0.5), (2.0, 0.25), (4.0, 0.15), (8.0, 0.1)].iter().enumerate().map(|(i, (f, w))| self.octave(x * f, y * f, i as u64) * w).sum()
    }

    fn octave(&self, x: f32, y: f32, octave: u64) -> f32 {
        let (x0, y0) = (x.floor(), y.floor());
        let ease = |f: f32| f * f * (3.0 - 2.0 * f);
        let (fx, fy) = (ease(x - x0), ease(y - y0));
        let (i, j) = (x0 as i64, y0 as i64);
        let corner = |di: i64, dj: i64| hash(self.seed ^ octave, i + di, j + dj);
        let top = corner(0, 0) + (corner(1, 0) - corner(0, 0)) * fx;
        let bottom = corner(0, 1) + (corner(1, 1) - corner(0, 1)) * fx;
        top + (bottom - top) * fy
    }
}

fn hash(seed: u64, i: i64, j: i64) -> f32 {
    let mut h = seed ^ (i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (j as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    h ^= h >> 31;
    h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h ^= h >> 29;
    h = h.wrapping_mul(0x94D0_49BB_1331_11EB);
    h ^= h >> 32;
    (h >> 40) as f32 / (1u64 << 24) as f32
}
