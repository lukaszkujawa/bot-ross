use crate::color::Hsv;
use crate::mypaint::ffi;
use std::error::Error;
use std::ffi::CString;
use std::fs;
use std::path::{Path, PathBuf};

const BRUSHES_DIR: &str = env!("MYPAINT_BRUSHES_DIR");

pub struct Engine {
    raw: *mut ffi::MyPaintBrush,
}

impl Engine {
    pub fn with_defaults() -> Self {
        let engine = Self::allocate();
        unsafe { ffi::mypaint_brush_from_defaults(engine.raw) };
        engine
    }

    pub fn named(name: &str) -> Result<Self, Box<dyn Error>> {
        Self::from_file(Path::new(BRUSHES_DIR).join(format!("{name}.myb")))
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, Box<dyn Error>> {
        let definition = CString::new(fs::read_to_string(&path)?)?;
        let engine = Self::allocate();
        if unsafe { ffi::mypaint_brush_from_string(engine.raw, definition.as_ptr()) } == 0 {
            return Err(format!("invalid brush file {}", path.as_ref().display()).into());
        }
        Ok(engine)
    }

    pub fn template_path(name: &str) -> PathBuf {
        Path::new(BRUSHES_DIR).join(format!("{name}.myb"))
    }

    pub fn set(&mut self, setting: &str, value: f32) -> Result<(), Box<dyn Error>> {
        unsafe { ffi::mypaint_brush_set_base_value(self.raw, setting_id(setting)?, value) };
        Ok(())
    }

    pub fn get(&self, setting: &str) -> Result<f32, Box<dyn Error>> {
        Ok(unsafe { ffi::mypaint_brush_get_base_value(self.raw, setting_id(setting)?) })
    }

    pub fn set_curve(&mut self, setting: &str, input: &str, points: &[[f32; 2]]) -> Result<(), Box<dyn Error>> {
        let (setting, input) = (setting_id(setting)?, input_id(input)?);
        unsafe { ffi::mypaint_brush_set_mapping_n(self.raw, setting, input, points.len() as i32) };
        for (index, [x, y]) in points.iter().enumerate() {
            unsafe { ffi::mypaint_brush_set_mapping_point(self.raw, setting, input, index as i32, *x, *y) };
        }
        Ok(())
    }

    pub fn set_color(&mut self, color: Hsv) {
        self.set_known("color_h", color.hue);
        self.set_known("color_s", color.saturation);
        self.set_known("color_v", color.value);
    }

    pub fn set_radius(&mut self, pixels: f32) {
        self.set_known("radius_logarithmic", pixels.ln());
    }

    pub fn set_opacity(&mut self, opacity: f32) {
        self.set_known("opaque", opacity);
    }

    pub(crate) fn raw(&mut self) -> *mut ffi::MyPaintBrush {
        self.raw
    }

    fn allocate() -> Self {
        Self { raw: unsafe { ffi::mypaint_brush_new() } }
    }

    fn set_known(&mut self, setting: &str, value: f32) {
        self.set(setting, value).expect("built-in setting name");
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        unsafe { ffi::mypaint_brush_unref(self.raw) };
    }
}

fn setting_id(name: &str) -> Result<i32, Box<dyn Error>> {
    let id = unsafe { ffi::mypaint_brush_setting_from_cname(CString::new(name)?.as_ptr()) };
    (id >= 0).then_some(id).ok_or_else(|| format!("unknown brush setting '{name}'").into())
}

fn input_id(name: &str) -> Result<i32, Box<dyn Error>> {
    let id = unsafe { ffi::mypaint_brush_input_from_cname(CString::new(name)?.as_ptr()) };
    (id >= 0).then_some(id).ok_or_else(|| format!("unknown brush input '{name}'").into())
}
