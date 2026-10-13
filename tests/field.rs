use bot_ross::brush::field::Field;

#[test]
fn field_is_deterministic_bounded_and_smooth() {
    let field = Field::new(42, [10.0, 5.0]);
    let same = Field::new(42, [10.0, 5.0]);
    let other = Field::new(43, [10.0, 5.0]);
    let mut values = Vec::new();
    for i in 0..200 {
        let (x, y) = (i as f32 * 0.7, i as f32 * 0.3);
        let value = field.at(x, y);
        assert!((0.0..=1.0).contains(&value));
        assert_eq!(value, same.at(x, y));
        assert!((value - field.at(x + 0.05, y)).abs() < 0.05);
        values.push(value);
    }
    assert!(values.iter().any(|v| *v < 0.4) && values.iter().any(|v| *v > 0.6));
    assert_ne!(field.at(3.0, 3.0), other.at(3.0, 3.0));
}
