use crate::mypaint::ffi;

pub const CHANNELS: usize = 4;
const TILE: usize = ffi::TILE_SIZE;
const TILE_LEN: usize = TILE * TILE * CHANNELS;

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

pub struct Tiles {
    surface: *mut ffi::MyPaintFixedTiledSurface,
}

impl Tiles {
    pub fn new(width: usize, height: usize) -> Self {
        Self { surface: unsafe { ffi::mypaint_fixed_tiled_surface_new(width as i32, height as i32) } }
    }

    pub fn interface(&self) -> *mut ffi::MyPaintSurface {
        unsafe { ffi::mypaint_fixed_tiled_surface_interface(self.surface) }
    }

    pub fn visit(&self, bounds: Bounds, readonly: bool, mut visit: impl FnMut(usize, usize, &mut [u16])) {
        if bounds.is_empty() {
            return;
        }
        for ty in bounds.top / TILE..bounds.bottom.div_ceil(TILE) {
            for tx in bounds.left / TILE..bounds.right.div_ceil(TILE) {
                let tile = unsafe { std::slice::from_raw_parts_mut(self.request(tx, ty, readonly), TILE_LEN) };
                for y in bounds.top.max(ty * TILE)..bounds.bottom.min((ty + 1) * TILE) {
                    for x in bounds.left.max(tx * TILE)..bounds.right.min((tx + 1) * TILE) {
                        let offset = ((y - ty * TILE) * TILE + (x - tx * TILE)) * CHANNELS;
                        visit(x, y, &mut tile[offset..offset + CHANNELS]);
                    }
                }
                self.release(tx, ty, readonly);
            }
        }
    }

    fn request(&self, tx: usize, ty: usize, readonly: bool) -> *mut u16 {
        let mut request = self.request_for(tx, ty, readonly);
        unsafe { ffi::mypaint_tiled_surface_tile_request_start(self.tiled(), &mut request) };
        request.buffer
    }

    fn release(&self, tx: usize, ty: usize, readonly: bool) {
        let mut request = self.request_for(tx, ty, readonly);
        unsafe { ffi::mypaint_tiled_surface_tile_request_end(self.tiled(), &mut request) };
    }

    fn request_for(&self, tx: usize, ty: usize, readonly: bool) -> ffi::MyPaintTileRequest {
        let mut request = std::mem::MaybeUninit::<ffi::MyPaintTileRequest>::uninit();
        unsafe {
            ffi::mypaint_tile_request_init(request.as_mut_ptr(), 0, tx as i32, ty as i32, readonly as i32);
            request.assume_init()
        }
    }

    fn tiled(&self) -> *mut ffi::MyPaintTiledSurface {
        self.surface as *mut ffi::MyPaintTiledSurface
    }
}

impl Drop for Tiles {
    fn drop(&mut self) {
        unsafe { ffi::mypaint_surface_unref(self.interface()) };
    }
}
