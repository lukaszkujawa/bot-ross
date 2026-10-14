use crate::image::Image;
use crate::stroke::Point;
use minifb::{Key, KeyRepeat, MouseButton, MouseMode, ScaleMode, WindowOptions};
use std::error::Error;

const LONGEST_WINDOW_SIDE: usize = 1000;

pub struct Window {
    inner: minifb::Window,
    width: usize,
    height: usize,
}

impl Window {
    pub fn open(title: &str, width: usize, height: usize) -> Result<Self, Box<dyn Error>> {
        let scale = (LONGEST_WINDOW_SIDE as f32 / width.max(height) as f32).min(1.0);
        let fit = |side: usize| ((side as f32 * scale) as usize).max(1);
        let options = WindowOptions { resize: true, scale_mode: ScaleMode::AspectRatioStretch, ..WindowOptions::default() };
        let mut inner = minifb::Window::new(title, fit(width), fit(height), options)?;
        inner.set_target_fps(30);
        Ok(Self { inner, width, height })
    }

    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn present(&mut self, image: &Image) -> Result<bool, Box<dyn Error>> {
        self.inner.update_with_buffer(&image.argb, image.width, image.height)?;
        Ok(self.inner.is_open() && !self.inner.is_key_down(Key::Escape))
    }

    pub fn show_until_closed(&mut self, image: &Image) -> Result<(), Box<dyn Error>> {
        while self.present(image)? {}
        Ok(())
    }

    pub fn mouse(&self) -> Option<Point> {
        let (window_width, window_height) = self.inner.get_size();
        self.inner.get_mouse_pos(MouseMode::Clamp).map(|(x, y)| Point { x: x * self.width as f32 / window_width as f32, y: y * self.height as f32 / window_height as f32 })
    }

    pub fn mouse_down(&self) -> bool {
        self.inner.get_mouse_down(MouseButton::Left)
    }

    pub fn pressed(&self, key: Key) -> bool {
        self.inner.is_key_pressed(key, KeyRepeat::No)
    }
}
