mod common;
use ::lender::prelude::*;
use common::*;

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

// ============================================================================
// DoubleEndedLender edge cases
// ============================================================================

#[test]
fn double_ended_empty() {
    let mut lender = VecLender::new(vec![]);
    assert_eq!(lender.next_back(), None);
    assert_eq!(lender.nth_back(0), None);
}

#[test]
fn double_ended_single_element() {
    let mut lender = VecLender::new(vec![42]);
    assert_eq!(lender.next_back(), Some(&42));
    assert_eq!(lender.next_back(), None);
}

#[test]
fn double_ended_mixed_iteration() {
    let mut lender = VecLender::new(vec![1, 2, 3, 4, 5]);
    assert_eq!(lender.next(), Some(&1));
    assert_eq!(lender.next_back(), Some(&5));
    assert_eq!(lender.next(), Some(&2));
    assert_eq!(lender.next_back(), Some(&4));
    assert_eq!(lender.next(), Some(&3));
    assert_eq!(lender.next(), None);
    assert_eq!(lender.next_back(), None);
}

// ============================================================================
// DoubleEndedLender adapter tests
// ============================================================================

#[test]
fn double_ended_rev() {
    let mut lender = VecLender::new(vec![1, 2, 3]).rev();
    assert_eq!(lender.next(), Some(&3));
    assert_eq!(lender.next(), Some(&2));
    assert_eq!(lender.next(), Some(&1));
    assert_eq!(lender.next(), None);
}

#[test]
fn double_ended_rev_next_back() {
    // Rev.next_back() should iterate forward through the original
    let mut lender = VecLender::new(vec![1, 2, 3]).rev();
    assert_eq!(lender.next_back(), Some(&1));
    assert_eq!(lender.next_back(), Some(&2));
    assert_eq!(lender.next_back(), Some(&3));
    assert_eq!(lender.next_back(), None);
}

#[test]
fn double_ended_chain() {
    let a = VecLender::new(vec![1, 2]);
    let b = VecLender::new(vec![3, 4]);
    let mut lender = a.chain(b);

    // Forward iteration
    assert_eq!(lender.next(), Some(&1));
    // Backward iteration
    assert_eq!(lender.next_back(), Some(&4));
    assert_eq!(lender.next_back(), Some(&3));
    // Back to forward
    assert_eq!(lender.next(), Some(&2));
    assert_eq!(lender.next(), None);
    assert_eq!(lender.next_back(), None);
}

#[test]
fn double_ended_take() {
    let mut lender = VecLender::new(vec![1, 2, 3, 4, 5]).take(3);
    assert_eq!(lender.next_back(), Some(&3));
    assert_eq!(lender.next_back(), Some(&2));
    assert_eq!(lender.next(), Some(&1));
    assert_eq!(lender.next(), None);
}

#[test]
fn double_ended_skip() {
    let mut lender = VecLender::new(vec![1, 2, 3, 4, 5]).skip(2);
    // Skip 2 from front, so remaining is [3, 4, 5]
    assert_eq!(lender.next_back(), Some(&5));
    assert_eq!(lender.next_back(), Some(&4));
    assert_eq!(lender.next(), Some(&3));
    assert_eq!(lender.next(), None);
}

#[test]
fn double_ended_step_by() {
    // step_by(2) on [1,2,3,4,5,6] yields [1,3,5] forward
    // and [6,4,2] backward from non-stepped perspective,
    // but DoubleEndedLender requires ExactSizeLender
    let mut lender = VecLender::new(vec![1, 2, 3, 4, 5, 6]).step_by(2);
    assert_eq!(lender.next(), Some(&1));
    assert_eq!(lender.next_back(), Some(&5));
    assert_eq!(lender.next(), Some(&3));
    assert_eq!(lender.next(), None);
}

#[test]
fn double_ended_zip() {
    let a = VecLender::new(vec![1, 2, 3]);
    let b = VecLender::new(vec![4, 5, 6]);
    let mut lender = lender::zip(a, b);

    assert_eq!(lender.next_back(), Some((&3, &6)));
    assert_eq!(lender.next(), Some((&1, &4)));
    assert_eq!(lender.next_back(), Some((&2, &5)));
    assert_eq!(lender.next(), None);
}

#[test]
fn double_ended_enumerate() {
    let mut lender = VecLender::new(vec![10, 20, 30]).enumerate();
    assert_eq!(lender.next_back(), Some((2, &30)));
    assert_eq!(lender.next(), Some((0, &10)));
    assert_eq!(lender.next_back(), Some((1, &20)));
    assert_eq!(lender.next(), None);
}

#[test]
fn double_ended_fuse() {
    let mut lender = VecLender::new(vec![1, 2, 3]).fuse();
    assert_eq!(lender.next_back(), Some(&3));
    assert_eq!(lender.next(), Some(&1));
    assert_eq!(lender.next_back(), Some(&2));
    assert_eq!(lender.next(), None);
    assert_eq!(lender.next_back(), None);
}

#[test]
fn double_ended_peekable() {
    let mut lender = VecLender::new(vec![1, 2, 3]).peekable();
    // Peek first (from front)
    assert_eq!(lender.peek(), Some(&&1));
    // Get from back
    assert_eq!(lender.next_back(), Some(&3));
    assert_eq!(lender.next_back(), Some(&2));
    // The peeked element is still there
    assert_eq!(lender.next(), Some(&1));
    assert_eq!(lender.next(), None);
}
