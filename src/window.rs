use crate::image::Image;
use minifb::{Key, WindowOptions};
use std::error::Error;

pub struct Window {
    inner: minifb::Window,
}

impl Window {
    pub fn open(title: &str, width: usize, height: usize) -> Result<Self, Box<dyn Error>> {
        let mut inner = minifb::Window::new(title, width, height, WindowOptions::default())?;
        inner.set_target_fps(30);
        Ok(Self { inner })
    }

    pub fn present(&mut self, image: &Image) -> Result<bool, Box<dyn Error>> {
        self.inner.update_with_buffer(&image.argb, image.width, image.height)?;
        Ok(self.inner.is_open() && !self.inner.is_key_down(Key::Escape))
    }

    pub fn show_until_closed(&mut self, image: &Image) -> Result<(), Box<dyn Error>> {
        while self.inner.is_open() && !self.inner.is_key_down(Key::Escape) {
            self.inner.update_with_buffer(&image.argb, image.width, image.height)?;
        }
        Ok(())
    }
}
