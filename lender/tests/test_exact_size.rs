mod common;
use ::lender::prelude::*;
use common::*;

// ============================================================================
// ExactSizeLender trait method tests
// ============================================================================

#[test]
fn test_exact_size_lender_len() {
    let lender = VecLender::new(vec![1, 2, 3, 4, 5]);
    assert_eq!(lender.len(), 5);
}

#[test]
fn test_exact_size_lender_len_after_iteration() {
    let mut lender = VecLender::new(vec![1, 2, 3, 4, 5]);
    assert_eq!(lender.len(), 5);
    lender.next();
    assert_eq!(lender.len(), 4);
    lender.next();
    lender.next();
    assert_eq!(lender.len(), 2);
}

#[test]
fn test_exact_size_lender_len_with_next_back() {
    let mut lender = VecLender::new(vec![1, 2, 3, 4, 5]);
    assert_eq!(lender.len(), 5);
    lender.next_back();
    assert_eq!(lender.len(), 4);
    lender.next();
    assert_eq!(lender.len(), 3);
}

#[test]
fn test_exact_size_lender_is_empty() {
    let mut lender = VecLender::new(vec![1]);
    assert!(!lender.is_empty());
    lender.next();
    assert!(lender.is_empty());
}

#[test]
fn test_exact_size_empty() {
    let lender = VecLender::new(vec![]);
    assert_eq!(lender.len(), 0);
    assert!(lender.is_empty());
}

// ============================================================================
// ExactSizeLender with adapters
// ============================================================================

#[test]
fn test_exact_size_take() {
    let lender = VecLender::new(vec![1, 2, 3, 4, 5]).take(3);
    assert_eq!(lender.len(), 3);
}

#[test]
fn test_exact_size_take_larger_than_source() {
    let lender = VecLender::new(vec![1, 2]).take(10);
    assert_eq!(lender.len(), 2); // min(10, 2) = 2
}

#[test]
fn test_exact_size_skip() {
    let lender = VecLender::new(vec![1, 2, 3, 4, 5]).skip(2);
    assert_eq!(lender.len(), 3);
}

#[test]
fn test_exact_size_skip_larger_than_source() {
    let lender = VecLender::new(vec![1, 2]).skip(10);
    assert_eq!(lender.len(), 0);
}

#[test]
fn test_exact_size_enumerate() {
    let lender = VecLender::new(vec![1, 2, 3]).enumerate();
    assert_eq!(lender.len(), 3);
}

#[test]
fn test_exact_size_zip() {
    let a = VecLender::new(vec![1, 2, 3]);
    let b = VecLender::new(vec![4, 5]);
    let zipped = lender::zip(a, b);
    assert_eq!(zipped.len(), 2); // min(3, 2) = 2
}

#[test]
fn test_exact_size_step_by() {
    // [1, 2, 3, 4, 5] with step 2 yields [1, 3, 5] = 3 elements
    let lender = VecLender::new(vec![1, 2, 3, 4, 5]).step_by(2);
    assert_eq!(lender.len(), 3);
}

#[test]
fn test_exact_size_fuse() {
    let lender = VecLender::new(vec![1, 2, 3]).fuse();
    assert_eq!(lender.len(), 3);
}

#[test]
fn test_exact_size_rev() {
    let lender = VecLender::new(vec![1, 2, 3]).rev();
    assert_eq!(lender.len(), 3);
}

#[test]
fn test_exact_size_peekable() {
    let lender = VecLender::new(vec![1, 2, 3]).peekable();
    assert_eq!(lender.len(), 3);
}

#[test]
fn test_exact_size_peekable_after_peek() {
    let mut lender = VecLender::new(vec![1, 2, 3]).peekable();
    lender.peek();
    assert_eq!(lender.len(), 3); // peek doesn't consume
}

#[test]
fn test_exact_size_chain_size_hint() {
    let a = VecLender::new(vec![1, 2]);
    let b = VecLender::new(vec![3, 4, 5]);
    let chained = a.chain(b);
    // Chain's size_hint combines both lenders
    let (lower, upper) = chained.size_hint();
    assert_eq!(lower, 5);
    assert_eq!(upper, Some(5));
}

// ============================================================================
// ExactSizeLender consistency tests
// ============================================================================

#[test]
fn test_exact_size_matches_size_hint() {
    let lender = VecLender::new(vec![1, 2, 3, 4, 5]);
    let (lower, upper) = lender.size_hint();
    let len = lender.len();
    assert_eq!(lower, len);
    assert_eq!(upper, Some(len));
}

#[test]
fn test_exact_size_decrements_correctly() {
    let mut lender = VecLender::new(vec![1, 2, 3]);

    for expected_len in (0..=3).rev() {
        assert_eq!(lender.len(), expected_len);
        if expected_len > 0 {
            lender.next();
        }
    }
}

#[test]
fn test_exact_size_with_nth() {
    let mut lender = VecLender::new(vec![1, 2, 3, 4, 5]);
    assert_eq!(lender.len(), 5);
    lender.nth(2); // consume 3 elements (0, 1, 2)
    assert_eq!(lender.len(), 2);
}

#[test]
fn test_exact_size_with_advance_by() {
    let mut lender = VecLender::new(vec![1, 2, 3, 4, 5]);
    assert_eq!(lender.len(), 5);
    lender.advance_by(3).unwrap();
    assert_eq!(lender.len(), 2);
}
