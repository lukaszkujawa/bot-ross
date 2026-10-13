use crate::image::Image;
use minifb::{Key, ScaleMode, WindowOptions};
use std::error::Error;

const LONGEST_WINDOW_SIDE: usize = 1000;

pub struct Window {
    inner: minifb::Window,
}

impl Window {
    pub fn open(title: &str, width: usize, height: usize) -> Result<Self, Box<dyn Error>> {
        let scale = (LONGEST_WINDOW_SIDE as f32 / width.max(height) as f32).min(1.0);
        let fit = |side: usize| ((side as f32 * scale) as usize).max(1);
        let options = WindowOptions { resize: true, scale_mode: ScaleMode::AspectRatioStretch, ..WindowOptions::default() };
        let mut inner = minifb::Window::new(title, fit(width), fit(height), options)?;
        inner.set_target_fps(30);
        Ok(Self { inner })
    }

    pub fn present(&mut self, image: &Image) -> Result<bool, Box<dyn Error>> {
        self.inner.update_with_buffer(&image.argb, image.width, image.height)?;
        Ok(self.inner.is_open() && !self.inner.is_key_down(Key::Escape))
    }

    pub fn show_until_closed(&mut self, image: &Image) -> Result<(), Box<dyn Error>> {
        while self.present(image)? {}
        Ok(())
    }
}
