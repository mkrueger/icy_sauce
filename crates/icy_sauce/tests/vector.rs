use icy_sauce::VectorFormat;

#[test]
fn test_vector_format_round_trip() {
    for i in 0..=3 {
        let format = VectorFormat::from_sauce(i);
        assert_eq!(format.to_sauce(), i);
    }
}
