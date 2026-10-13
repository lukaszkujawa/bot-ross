#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bounds {
    pub left: usize,
    pub top: usize,
    pub right: usize,
    pub bottom: usize,
}

impl Bounds {
    pub fn whole(width: usize, height: usize) -> Self {
        Self { left: 0, top: 0, right: width, bottom: height }
    }

    pub fn empty() -> Self {
        Self { left: 0, top: 0, right: 0, bottom: 0 }
    }

    pub fn is_empty(&self) -> bool {
        self.right <= self.left || self.bottom <= self.top
    }

    pub fn union(self, other: Self) -> Self {
        match (self.is_empty(), other.is_empty()) {
            (true, _) => other,
            (_, true) => self,
            _ => Self { left: self.left.min(other.left), top: self.top.min(other.top), right: self.right.max(other.right), bottom: self.bottom.max(other.bottom) },
        }
    }

    pub fn area(&self) -> usize {
        if self.is_empty() { 0 } else { (self.right - self.left) * (self.bottom - self.top) }
    }

    pub fn around(x: f32, y: f32, extent: f32, width: usize, height: usize) -> Self {
        Self::enclosing([x, x], [y, y], extent, width, height)
    }

    pub fn enclosing(xs: [f32; 2], ys: [f32; 2], extent: f32, width: usize, height: usize) -> Self {
        let clamp = |value: f32, limit: usize| (value.max(0.0) as usize).min(limit);
        Self {
            left: clamp((xs[0] - extent).floor(), width),
            top: clamp((ys[0] - extent).floor(), height),
            right: clamp((xs[1] + extent).ceil() + 1.0, width),
            bottom: clamp((ys[1] + extent).ceil() + 1.0, height),
        }
    }
}
