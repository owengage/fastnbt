use crate::StatesIter;

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
