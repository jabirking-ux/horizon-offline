use super::*;

#[test]
fn identifier_adds_default_namespace() {
    assert_eq!(identifier("stone").unwrap(), "minecraft:stone");
    assert_eq!(identifier("a:b").unwrap(), "a:b");
}

#[test]
fn positive_modulo_wraps_negatives() {
    assert!((positive_modulo(-0.25, 1.0) - 0.75).abs() < 1e-6);
    assert!((positive_modulo(1.5, 1.0) - 0.5).abs() < 1e-6);
}

#[test]
fn display_context_names_round_trip() {
    for name in ["gui", "ground", "fixed", "head"] {
        assert_eq!(DisplayContext::by_name(name).unwrap().name(), name);
    }
}

#[test]
fn last_index_less_or_equal_linear_and_binary() {
    use super::last_index_less_or_equal;
    let make = |thresholds: &[f32]| -> Vec<(f32, ItemModelNode)> {
        thresholds
            .iter()
            .map(|&t| (t, ItemModelNode::Empty))
            .collect()
    };

    // Empty
    assert_eq!(last_index_less_or_equal(&[], 0.0), None);

    // Short (< 16): linear search
    let short = make(&[-1.0, 0.0, 1.0, 2.5, 5.0]);
    assert_eq!(last_index_less_or_equal(&short, -2.0), None);
    assert_eq!(last_index_less_or_equal(&short, -1.0), Some(0));
    assert_eq!(last_index_less_or_equal(&short, -0.5), Some(0));
    assert_eq!(last_index_less_or_equal(&short, 0.0), Some(1));
    assert_eq!(last_index_less_or_equal(&short, 2.5), Some(3));
    assert_eq!(last_index_less_or_equal(&short, 3.0), Some(3));
    assert_eq!(last_index_less_or_equal(&short, 5.0), Some(4));
    assert_eq!(last_index_less_or_equal(&short, 10.0), Some(4));

    // Long (>= 16): binary search
    let thresholds: Vec<f32> = (0..32).map(|i| i as f32 * 10.0).collect();
    let long = make(&thresholds);
    assert_eq!(last_index_less_or_equal(&long, -5.0), None);
    assert_eq!(last_index_less_or_equal(&long, 0.0), Some(0));
    assert_eq!(last_index_less_or_equal(&long, 5.0), Some(0));
    assert_eq!(last_index_less_or_equal(&long, 10.0), Some(1));
    assert_eq!(last_index_less_or_equal(&long, 150.0), Some(15));
    assert_eq!(last_index_less_or_equal(&long, 155.0), Some(15));
    assert_eq!(last_index_less_or_equal(&long, 310.0), Some(31));
    assert_eq!(last_index_less_or_equal(&long, 1000.0), Some(31));

    // Short lists use upper-bound tie behavior, including signed zero.
    let duplicates = make(&[1.0, 1.0, 2.0]);
    assert_eq!(last_index_less_or_equal(&duplicates, 1.0), Some(1));

    let zeroes = make(&[-0.0, 0.0]);
    assert_eq!(last_index_less_or_equal(&zeroes, -0.0), Some(1));
    assert_eq!(last_index_less_or_equal(&zeroes, 0.0), Some(1));

    // At 16 entries, vanilla returns Arrays.binarySearch's hit, not the last tie.
    let duplicates = make(&[
        0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 7.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0,
    ]);
    assert_eq!(last_index_less_or_equal(&duplicates, 7.0), Some(7));

    let zeroes = make(&[
        -8.0, -7.0, -6.0, -5.0, -4.0, -3.0, -2.0, -0.0, -0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0,
    ]);
    assert_eq!(last_index_less_or_equal(&zeroes, -0.0), Some(7));
    assert_eq!(last_index_less_or_equal(&zeroes, 0.0), Some(9));
}
