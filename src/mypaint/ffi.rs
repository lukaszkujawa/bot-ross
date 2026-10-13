use std::ffi::{c_char, c_double, c_float, c_int, c_ushort, c_void};

pub const TILE_SIZE: usize = 64;

#[repr(C)]
pub struct MyPaintBrush {
    _private: [u8; 0],
}

#[repr(C)]
pub struct MyPaintSurface {
    _private: [u8; 0],
}

#[repr(C)]
pub struct MyPaintTiledSurface {
    _private: [u8; 0],
}

#[repr(C)]
pub struct MyPaintFixedTiledSurface {
    _private: [u8; 0],
}

#[repr(C)]
#[derive(Default)]
pub struct MyPaintRectangle {
    pub x: c_int,
    pub y: c_int,
    pub width: c_int,
    pub height: c_int,
}

#[repr(C)]
pub struct MyPaintTileRequest {
    pub tx: c_int,
    pub ty: c_int,
    pub readonly: c_int,
    pub buffer: *mut c_ushort,
    pub context: *mut c_void,
    pub thread_id: c_int,
    pub mipmap_level: c_int,
}

unsafe extern "C" {
    pub fn mypaint_brush_new() -> *mut MyPaintBrush;
    pub fn mypaint_brush_unref(brush: *mut MyPaintBrush);
    pub fn mypaint_brush_from_defaults(brush: *mut MyPaintBrush);
    pub fn mypaint_brush_from_string(brush: *mut MyPaintBrush, json: *const c_char) -> c_int;
    pub fn mypaint_brush_set_base_value(brush: *mut MyPaintBrush, setting: c_int, value: c_float);
    pub fn mypaint_brush_get_base_value(brush: *mut MyPaintBrush, setting: c_int) -> c_float;
    pub fn mypaint_brush_set_mapping_n(brush: *mut MyPaintBrush, setting: c_int, input: c_int, n: c_int);
    pub fn mypaint_brush_set_mapping_point(brush: *mut MyPaintBrush, setting: c_int, input: c_int, index: c_int, x: c_float, y: c_float);
    pub fn mypaint_brush_setting_from_cname(cname: *const c_char) -> c_int;
    pub fn mypaint_brush_input_from_cname(cname: *const c_char) -> c_int;
    pub fn mypaint_brush_reset(brush: *mut MyPaintBrush);
    pub fn mypaint_brush_new_stroke(brush: *mut MyPaintBrush);
    pub fn mypaint_brush_stroke_to(
        brush: *mut MyPaintBrush,
        surface: *mut MyPaintSurface,
        x: c_float,
        y: c_float,
        pressure: c_float,
        xtilt: c_float,
        ytilt: c_float,
        dtime: c_double,
    ) -> c_int;

    pub fn mypaint_surface_begin_atomic(surface: *mut MyPaintSurface);
    pub fn mypaint_surface_end_atomic(surface: *mut MyPaintSurface, roi: *mut MyPaintRectangle);
    pub fn mypaint_surface_unref(surface: *mut MyPaintSurface);

    pub fn mypaint_fixed_tiled_surface_new(width: c_int, height: c_int) -> *mut MyPaintFixedTiledSurface;
    pub fn mypaint_fixed_tiled_surface_interface(surface: *mut MyPaintFixedTiledSurface) -> *mut MyPaintSurface;

    pub fn mypaint_tile_request_init(request: *mut MyPaintTileRequest, level: c_int, tx: c_int, ty: c_int, readonly: c_int);
    pub fn mypaint_tiled_surface_tile_request_start(surface: *mut MyPaintTiledSurface, request: *mut MyPaintTileRequest);
    pub fn mypaint_tiled_surface_tile_request_end(surface: *mut MyPaintTiledSurface, request: *mut MyPaintTileRequest);
}
