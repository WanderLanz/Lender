mod common;
use common::*;
use ::lender::prelude::*;

// ============================================================================
// DoubleEndedLender trait method tests
// ============================================================================

#[test]
fn double_ended_lender_next_back() {
    let mut lender = VecLender::new(vec![1, 2, 3]);
    assert_eq!(lender.next_back(), Some(&3));
    assert_eq!(lender.next_back(), Some(&2));
    assert_eq!(lender.next_back(), Some(&1));
    assert_eq!(lender.next_back(), None);
}

#[test]
fn double_ended_lender_advance_back_by() {
    use core::num::NonZeroUsize;

    let mut lender = VecLender::new(vec![1, 2, 3, 4, 5]);

    // advance_back_by(2) skips 2 elements from back
    assert_eq!(lender.advance_back_by(2), Ok(()));
    assert_eq!(lender.next_back(), Some(&3));

    // advance_back_by past remaining
    assert_eq!(
        lender.advance_back_by(5),
        Err(NonZeroUsize::new(3).unwrap())
    );
}

#[test]
fn double_ended_lender_nth_back() {
    let mut lender = VecLender::new(vec![1, 2, 3, 4, 5]);

    // nth_back(1) skips 5 and returns 4
    assert_eq!(lender.nth_back(1), Some(&4));
    // nth_back(0) returns 3
    assert_eq!(lender.nth_back(0), Some(&3));
}

#[test]
fn double_ended_lender_rfind() {
    let mut lender = VecLender::new(vec![1, 2, 3, 4, 5]);
    // rfind searches from back
    assert_eq!(lender.rfind(|&&x| x < 4), Some(&3));
}

#[test]
fn double_ended_lender_rposition() {
    let mut lender = VecLender::new(vec![1, 2, 3, 2, 1]);
    // rposition searches from the end but returns a front-based index
    // Element 2 appears at indices 1 and 3; searching from the end finds index 3 first
    assert_eq!(lender.rposition(|&x| x == 2), Some(3));
}

// ============================================================================
// DoubleEndedLender comprehensive tests
// ============================================================================

#[test]
fn double_ended_advance_back_by() {
    use lender::DoubleEndedLender;

    let mut lender = VecLender::new(vec![1, 2, 3, 4, 5]);
    assert_eq!(lender.advance_back_by(2), Ok(()));
    assert_eq!(lender.next_back(), Some(&3));
}

#[test]
fn double_ended_nth_back() {
    use lender::DoubleEndedLender;

    let mut lender = VecLender::new(vec![1, 2, 3, 4, 5]);
    assert_eq!(lender.nth_back(2), Some(&3)); // 5, 4, [3]
}

#[test]
fn double_ended_try_rfold() {
    use lender::DoubleEndedLender;

    let result: Option<i32> = VecLender::new(vec![1, 2, 3]).try_rfold(0, |acc, x| Some(acc + *x));
    assert_eq!(result, Some(6));
}

#[test]
fn double_ended_rfold() {
    use lender::DoubleEndedLender;

    let values: Vec<i32> = VecLender::new(vec![1, 2, 3]).rfold(Vec::new(), |mut acc, &x| {
        acc.push(x);
        acc
    });
    assert_eq!(values, vec![3, 2, 1]);
}
