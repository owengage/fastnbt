use crate::{min_bits_for_n_states, StatesIter};

#[test]
fn iter_zeroes() {
    let actual: Vec<_> = StatesIter::new(4, 16, &[0]).collect();
    let expected = [0; 16];
    assert_eq!(expected[..], actual);
}

#[test]
fn iter_one() {
    let actual: Vec<_> = StatesIter::new(4, 16, &[1]).collect();
    let expected = [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    assert_eq!(expected[..], actual);
}

#[test]
fn iter_capped() {
    let actual: Vec<_> = StatesIter::new(4, 2, &[1]).collect();
    let expected = [1, 0];
    assert_eq!(expected[..], actual);
}

#[test]
fn iter_skips_padding() {
    let actual: Vec<_> = StatesIter::new(60, 3, &[1, 2, 3]).collect();
    let expected = [1, 2, 3];
    assert_eq!(expected[..], actual);
}

#[test]
fn min_bits() {
    let ideal = |n: usize| (n as f64).log2().ceil() as usize;

    assert_eq!(0, min_bits_for_n_states(1));
    assert_eq!(1, min_bits_for_n_states(2));
    assert_eq!(2, min_bits_for_n_states(3));
    assert_eq!(2, min_bits_for_n_states(4));
    assert_eq!(3, min_bits_for_n_states(5));
    assert_eq!(3, min_bits_for_n_states(5));

    for i in 1..16 * 16 * 16 {
        assert_eq!(ideal(i), min_bits_for_n_states(i));
    }
}
