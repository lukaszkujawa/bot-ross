use bot_ross::reference::Reference;
use bot_ross::stroke::{Curve, Point};

fn checker(width: usize, height: usize) -> Vec<u8> {
    (0..width * height).flat_map(|i| if (i % width + i / width).is_multiple_of(2) { [255, 0, 0] } else { [0, 0, 255] }).collect()
}

#[test]
fn samples_pixels_and_clamps_outside() {
    let reference = Reference::from_rgb8(4, 2, &checker(4, 2));
    assert_eq!(reference.pixel(0, 0), [32768, 0, 0]);
    assert_eq!(reference.pixel(1, 0), [0, 0, 32768]);
    assert_eq!(reference.sample(Point { x: -5.0, y: 9.0 }), reference.pixel(0, 1));
    assert_eq!(reference.mean(), [0.5, 0.0, 0.5]);
}

#[test]
fn mean_along_a_curve_over_a_solid_image_is_that_colour() {
    let reference = Reference::from_rgb8(10, 10, &vec![51; 300]);
    let colour = reference.mean_along(&Curve::line(Point { x: 0.0, y: 0.0 }, Point { x: 9.0, y: 9.0 }));
    assert!(colour.iter().all(|c| (c - 0.2).abs() < 0.01), "{colour:?}");
}

#[test]
fn edges_peak_at_colour_boundaries() {
    let mut bytes = vec![0u8; 10 * 10 * 3];
    bytes[5 * 3 * 10..].iter_mut().for_each(|b| *b = 255);
    let edges = Reference::from_rgb8(10, 10, &bytes).edges();
    assert_eq!(edges[5 * 10 + 5], 1.0);
    assert_eq!(edges[15], 0.0);
}

#[test]
fn loads_and_resizes_an_image_keeping_aspect() {
    let path = std::env::temp_dir().join("bot-ross-reference-test.png");
    image::RgbImage::from_raw(40, 30, checker(40, 30)).unwrap().save(&path).unwrap();
    let reference = Reference::load(&path, 20).unwrap();
    assert_eq!((reference.width, reference.height), (20, 15));
}
