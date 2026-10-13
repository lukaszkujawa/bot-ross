use bot_ross::color::Premultiplied;
use bot_ross::fitness::Fitness;
use bot_ross::painting::config::PlacementMode;
use bot_ross::placement::Placement;
use bot_ross::reference::Reference;
use rand::SeedableRng;
use rand::rngs::StdRng;

#[test]
fn picks_land_where_the_error_is() {
    let reference = Reference::from_rgb8(128, 96, &vec![255; 128 * 96 * 3]);
    let below: Vec<Premultiplied> = (0..128 * 96).map(|i| if (64..96).contains(&(i % 128)) && (32..64).contains(&(i / 128)) { Premultiplied::new(0, 0, 0, 32768) } else { Premultiplied::WHITE }).collect();
    let fitness = Fitness::new(&reference, below);
    let placement = Placement::new(&fitness, PlacementMode::Error, None, 32);
    let mut rng = StdRng::seed_from_u64(1);
    for _ in 0..100 {
        let point = placement.pick(&mut rng);
        assert!((64.0..96.0).contains(&point.x) && (32.0..64.0).contains(&point.y), "{point:?}");
    }
    let random = Placement::new(&fitness, PlacementMode::Random, None, 32);
    assert!((0..100).any(|_| random.pick(&mut rng).x < 64.0));
}
