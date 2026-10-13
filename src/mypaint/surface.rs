use crate::bounds::Bounds;
use crate::mypaint::ffi;

const CHANNELS: usize = 4;
const TILE: usize = ffi::TILE_SIZE;
const TILE_LEN: usize = TILE * TILE * CHANNELS;

pub struct Surface {
    raw: *mut ffi::MyPaintFixedTiledSurface,
}

impl Surface {
    pub fn new(width: usize, height: usize) -> Self {
        Self { raw: unsafe { ffi::mypaint_fixed_tiled_surface_new(width as i32, height as i32) } }
    }

    pub(crate) fn interface(&self) -> *mut ffi::MyPaintSurface {
        unsafe { ffi::mypaint_fixed_tiled_surface_interface(self.raw) }
    }

    pub fn visit(&self, bounds: Bounds, mut visit: impl FnMut(usize, usize, &[u16])) {
        self.walk(bounds, true, |x, y, channels| visit(x, y, channels));
    }

    pub fn visit_mut(&self, bounds: Bounds, visit: impl FnMut(usize, usize, &mut [u16])) {
        self.walk(bounds, false, visit);
    }

    fn walk(&self, bounds: Bounds, readonly: bool, mut visit: impl FnMut(usize, usize, &mut [u16])) {
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
        self.raw as *mut ffi::MyPaintTiledSurface
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe { ffi::mypaint_surface_unref(self.interface()) };
    }
}
