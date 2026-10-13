use std::error::Error;

const WIDTH: u32 = 512;
const HEIGHT: u32 = 384;

fn main() -> Result<(), Box<dyn Error>> {
    let image = image::RgbImage::from_fn(WIDTH, HEIGHT, |x, y| {
        let (fx, fy) = (x as f32 / WIDTH as f32, y as f32 / HEIGHT as f32);
        let hill = |phase: f32, height: f32, base: f32| base + height * (fx * 3.0 + phase).sin() * 0.5;
        let mix = |a: [f32; 3], b: [f32; 3], t: f32| [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t];
        let sky = mix([0.98, 0.62, 0.35], [0.25, 0.42, 0.78], fy * 1.6);
        let sun = ((fx - 0.68).hypot((fy - 0.32) * HEIGHT as f32 / WIDTH as f32) < 0.07) as u8 as f32;
        let colour = if fy > hill(1.2, 0.08, 0.78) {
            [0.13, 0.32, 0.15]
        } else if fy > hill(0.3, 0.12, 0.66) {
            [0.35, 0.55, 0.22]
        } else {
            mix(sky, [1.0, 0.95, 0.75], sun)
        };
        image::Rgb(colour.map(|c| (c.clamp(0.0, 1.0) * 255.0) as u8))
    });
    image.save("examples/reference.png")?;
    Ok(())
}
