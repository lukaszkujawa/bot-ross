use crate::image::Image;
use image::{ImageFormat, RgbImage};
use std::error::Error;
use std::io::Cursor;
use std::path::Path;

pub fn save_image(image: &Image, path: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
    rgb_image(image)?.save(path)?;
    Ok(())
}

pub fn png_bytes(image: &Image) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut encoded = Cursor::new(Vec::new());
    rgb_image(image)?.write_to(&mut encoded, ImageFormat::Png)?;
    Ok(encoded.into_inner())
}

pub fn rgb_bytes(image: &Image) -> Vec<u8> {
    image.argb.iter().flat_map(|px| [(px >> 16) as u8, (px >> 8) as u8, *px as u8]).collect()
}

fn rgb_image(image: &Image) -> Result<RgbImage, Box<dyn Error>> {
    Ok(RgbImage::from_raw(image.width as u32, image.height as u32, rgb_bytes(image)).ok_or("image size mismatch")?)
}
