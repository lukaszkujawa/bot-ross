use crate::image::Image;
use std::error::Error;
use std::path::Path;

pub fn save_png(image: &Image, path: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
    let bytes: Vec<u8> = image.argb.iter().flat_map(|px| [(px >> 16) as u8, (px >> 8) as u8, *px as u8]).collect();
    let rgb = image::RgbImage::from_raw(image.width as u32, image.height as u32, bytes).ok_or("image size mismatch")?;
    rgb.save(path)?;
    Ok(())
}
