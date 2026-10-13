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

#[test]
fn blur_softens_a_step_edge_and_keeps_flat_areas() {
    let mut bytes = vec![0u8; 40 * 10 * 3];
    for y in 0..10 {
        for x in 20..40 {
            bytes[(y * 40 + x) * 3..(y * 40 + x) * 3 + 3].copy_from_slice(&[255, 255, 255]);
        }
    }
    let sharp = Reference::from_rgb8(40, 10, &bytes);
    let soft = sharp.blurred(2.0);
    assert_eq!(soft.pixel(2, 5), [0, 0, 0]);
    assert_eq!(soft.pixel(37, 5), [32768, 32768, 32768]);
    let ramp = soft.pixel(19, 5)[0];
    assert!(ramp > 2000 && ramp < 30000, "{ramp}");
    assert!(soft.pixel(21, 5)[0] > ramp);
    assert_eq!(sharp.blurred(0.0).pixel(19, 5), sharp.pixel(19, 5));
}

#[test]
fn grading_posterises_pushes_contrast_and_drains_colour() {
    let reference = Reference::from_rgb8(2, 1, &[64, 128, 192, 200, 100, 50]);
    let flat = reference.graded(1.0, 1.0, 1.0, 2);
    assert_eq!(flat.pixel(0, 0), [0, 32768, 32768]);
    let grey = reference.graded(1.0, 0.0, 1.0, 0);
    let [r, g, b] = grey.pixel(1, 0);
    assert!(r == g && g == b, "{:?}", grey.pixel(1, 0));
    let punchy = reference.graded(2.0, 1.0, 1.0, 0);
    assert!(punchy.pixel(0, 0)[0] < 100);
    assert!(punchy.pixel(0, 0)[2] > reference.pixel(0, 0)[2]);
    assert_eq!(reference.graded(1.0, 1.0, 1.0, 0).pixel(0, 0), reference.pixel(0, 0));
    let lifted = reference.graded(1.0, 1.0, 2.0, 0).pixel(0, 0);
    assert!(lifted[0] > reference.pixel(0, 0)[0] && lifted[0] == ((64.0f32 / 255.0).sqrt() * 32768.0).round() as u16, "{lifted:?}");
    assert!(reference.graded(1.0, 1.0, 0.5, 0).pixel(0, 0)[0] < reference.pixel(0, 0)[0]);
}

#[test]
fn smoothing_flattens_noise_but_keeps_a_step_edge() {
    let mut bytes = vec![0u8; 60 * 20 * 3];
    for y in 0..20 {
        for x in 0..60 {
            let base = if x < 30 { 60 } else { 200 };
            let noise = ((x * 7 + y * 13) % 11) as u8;
            bytes[(y * 60 + x) * 3..(y * 60 + x) * 3 + 3].copy_from_slice(&[base + noise; 3]);
        }
    }
    let noisy = Reference::from_rgb8(60, 20, &bytes);
    let smooth = noisy.smoothed(3.0);
    let spread = |reference: &Reference, xs: std::ops::Range<usize>| {
        let values: Vec<u16> = xs.map(|x| reference.pixel(x, 10)[0]).collect();
        values.iter().max().unwrap() - values.iter().min().unwrap()
    };
    assert!(spread(&smooth, 5..25) < spread(&noisy, 5..25) / 4, "{} vs {}", spread(&smooth, 5..25), spread(&noisy, 5..25));
    assert!(smooth.pixel(29, 10)[0] < 10000 && smooth.pixel(30, 10)[0] > 24000, "{:?} {:?}", smooth.pixel(29, 10), smooth.pixel(30, 10));
}
