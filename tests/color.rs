use bot_ross::color::{Blend, Hsv, OPAQUE, Premultiplied};

#[test]
fn transparent_over_white_stays_white() {
    assert_eq!(Premultiplied::TRANSPARENT.over(Premultiplied::WHITE), Premultiplied::WHITE);
}

#[test]
fn opaque_colour_replaces_background() {
    let red = Premultiplied::new(OPAQUE, 0, 0, OPAQUE);
    assert_eq!(red.over(Premultiplied::WHITE), red);
}

#[test]
fn half_transparent_black_darkens_white_by_half() {
    let half_black = Premultiplied::new(0, 0, 0, OPAQUE / 2);
    assert_eq!(half_black.over(Premultiplied::WHITE).to_argb(), 0x007f7f7f);
}

#[test]
fn white_maps_to_full_argb() {
    assert_eq!(Premultiplied::WHITE.to_argb(), 0x00ffffff);
}

#[test]
fn hsv_converts_to_rgb() {
    assert_eq!(Hsv { hue: 0.0, saturation: 1.0, value: 1.0 }.to_rgb(), [1.0, 0.0, 0.0]);
    assert_eq!(Hsv { hue: 1.0 / 3.0, saturation: 1.0, value: 0.5 }.to_rgb(), [0.0, 0.5, 0.0]);
    assert_eq!(Hsv { hue: 0.7, saturation: 0.0, value: 0.25 }.to_rgb(), [0.25, 0.25, 0.25]);
}

#[test]
fn rgb_round_trips_through_premultiplied() {
    let blue = Premultiplied::from_rgb([0.0, 0.0, 1.0], 0.5);
    assert_eq!(blue, Premultiplied::new(0, 0, 16384, 16384));
    assert_eq!(blue.to_rgb(), [0.0, 0.0, 1.0]);
    assert_eq!(Premultiplied::TRANSPARENT.mix(Premultiplied::WHITE, 0.5), Premultiplied::new(16384, 16384, 16384, 16384));
}

#[test]
fn rgb_to_hsv_round_trips() {
    for rgb in [[1.0, 0.0, 0.0], [0.0, 0.5, 0.0], [0.25, 0.25, 0.25], [0.2, 0.4, 0.9]] {
        let back = Hsv::from_rgb(rgb).to_rgb();
        assert!(rgb.iter().zip(back).all(|(a, b)| (a - b).abs() < 1e-5), "{rgb:?} -> {back:?}");
    }
    assert_eq!(Hsv::from_rgb([0.0, 0.0, 1.0]).hue, 2.0 / 3.0);
}

#[test]
fn multiply_glazes_like_transparent_pigment() {
    let glaze = Premultiplied::from_rgb([1.0, 0.5, 0.0], 0.5);
    let on_white = glaze.multiply(Premultiplied::WHITE);
    assert_eq!(on_white.alpha, OPAQUE);
    assert_eq!(on_white.to_rgb().map(|c| (c * 100.0).round()), [100.0, 75.0, 50.0]);
    let dark = Premultiplied::from_rgb([0.2, 0.2, 0.2], 1.0);
    let lightened = Premultiplied::from_rgb([1.0, 1.0, 1.0], 0.9).multiply(dark);
    assert_eq!(lightened, dark);
    assert_eq!(glaze.multiply(Premultiplied::TRANSPARENT), glaze);
    assert_eq!(Blend::Normal.apply(glaze, dark), glaze.over(dark));
}
