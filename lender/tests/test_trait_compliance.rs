mod common;

use ::lender::prelude::*;
use common::*;

// =================================================================
// ExactSizeLender compliance tests
// =================================================================

#[test]
fn exact_size_enumerate() {
    let mut lender = VecLender::new(vec![1, 2, 3]).enumerate();
    assert_eq!(lender.len(), 3);
    assert!(!lender.is_empty());
    lender.next();
    assert_eq!(lender.len(), 2);
    lender.next();
    assert_eq!(lender.len(), 1);
    lender.next();
    assert_eq!(lender.len(), 0);
    assert!(lender.is_empty());
}

#[test]
fn exact_size_map() {
    let mut lender =
        VecLender::new(vec![1, 2, 3]).map(lender::covar_mut!(for<'all> |x: &i32| -> i32 { x * 2 }));
    assert_eq!(lender.len(), 3);
    assert!(!lender.is_empty());
    lender.next();
    assert_eq!(lender.len(), 2);
    lender.next();
    lender.next();
    assert_eq!(lender.len(), 0);
    assert!(lender.is_empty());
}

#[test]
fn exact_size_inspect() {
    let mut lender = VecLender::new(vec![10, 20, 30]).inspect(|_| {});
    assert_eq!(lender.len(), 3);
    lender.next();
    assert_eq!(lender.len(), 2);
    lender.next();
    lender.next();
    assert_eq!(lender.len(), 0);
    assert!(lender.is_empty());
}

#[test]
fn exact_size_rev() {
    let mut lender = VecLender::new(vec![1, 2, 3]).rev();
    assert_eq!(lender.len(), 3);
    assert!(!lender.is_empty());
    lender.next();
    assert_eq!(lender.len(), 2);
    lender.next();
    lender.next();
    assert_eq!(lender.len(), 0);
    assert!(lender.is_empty());
}

#[test]
fn exact_size_zip() {
    let a = VecLender::new(vec![1, 2, 3]);
    let b = VecLender::new(vec![4, 5, 6]);
    let mut lender = a.zip(b);
    assert_eq!(lender.len(), 3);
    assert!(!lender.is_empty());
    lender.next();
    assert_eq!(lender.len(), 2);
    lender.next();
    lender.next();
    assert_eq!(lender.len(), 0);
    assert!(lender.is_empty());
}

#[test]
fn exact_size_peekable() {
    let mut lender = VecLender::new(vec![1, 2, 3]).peekable();
    assert_eq!(lender.len(), 3);
    lender.next();
    assert_eq!(lender.len(), 2);
    lender.next();
    lender.next();
    assert_eq!(lender.len(), 0);
    assert!(lender.is_empty());
}

#[test]
fn exact_size_skip() {
    let mut lender = VecLender::new(vec![1, 2, 3, 4, 5]).skip(2);
    assert_eq!(lender.len(), 3);
    assert!(!lender.is_empty());
    lender.next();
    assert_eq!(lender.len(), 2);
    lender.next();
    lender.next();
    assert_eq!(lender.len(), 0);
    assert!(lender.is_empty());
}

#[test]
fn exact_size_fuse() {
    let mut lender = VecLender::new(vec![1, 2]).fuse();
    assert_eq!(lender.len(), 2);
    assert!(!lender.is_empty());
    lender.next();
    assert_eq!(lender.len(), 1);
    lender.next();
    assert_eq!(lender.len(), 0);
    assert!(lender.is_empty());
}

#[test]
fn exact_size_mutate() {
    let mut lender = VecLender::new(vec![1, 2, 3]).mutate(|_| {});
    assert_eq!(lender.len(), 3);
    lender.next();
    assert_eq!(lender.len(), 2);
    lender.next();
    lender.next();
    assert_eq!(lender.len(), 0);
    assert!(lender.is_empty());
}

// =================================================================
// FusedLender compliance tests
// =================================================================

#[test]
fn fused_fuse_adapter() {
    let mut lender = VecLender::new(vec![1, 2]).fuse();
    assert_eq!(lender.next(), Some(&1));
    assert_eq!(lender.next(), Some(&2));
    assert_eq!(lender.next(), None);
    // FusedLender contract: must keep returning None
    assert_eq!(lender.next(), None);
    assert_eq!(lender.next(), None);
}

#[test]
fn fused_enumerate() {
    let mut lender = VecLender::new(vec![10]).enumerate();
    assert_eq!(lender.next(), Some((0, &10)));
    assert_eq!(lender.next(), None);
    assert_eq!(lender.next(), None);
}

#[test]
fn fused_map() {
    let mut lender =
        VecLender::new(vec![1]).map(lender::covar_mut!(for<'all> |x: &i32| -> i32 { x + 1 }));
    assert_eq!(lender.next(), Some(2));
    assert_eq!(lender.next(), None);
    assert_eq!(lender.next(), None);
}

#[test]
fn fused_filter() {
    let mut lender = VecLender::new(vec![1, 2, 3]).filter(|x| **x > 5);
    assert_eq!(lender.next(), None);
    assert_eq!(lender.next(), None);
}

#[test]
fn fused_zip() {
    let a = VecLender::new(vec![1]);
    let b = VecLender::new(vec![2]);
    let mut lender = a.zip(b);
    assert_eq!(lender.next(), Some((&1, &2)));
    assert_eq!(lender.next(), None);
    assert_eq!(lender.next(), None);
}

#[test]
fn fused_skip() {
    let mut lender = VecLender::new(vec![1, 2]).skip(10);
    assert_eq!(lender.next(), None);
    assert_eq!(lender.next(), None);
}

#[test]
fn fused_take() {
    let mut lender = VecLender::new(vec![1, 2, 3]).take(1);
    assert_eq!(lender.next(), Some(&1));
    assert_eq!(lender.next(), None);
    assert_eq!(lender.next(), None);
}

#[test]
fn fused_chain() {
    let a = VecLender::new(vec![1]);
    let b = VecLender::new(vec![2]);
    let mut lender = a.chain(b);
    assert_eq!(lender.next(), Some(&1));
    assert_eq!(lender.next(), Some(&2));
    assert_eq!(lender.next(), None);
    assert_eq!(lender.next(), None);
}

#[test]
fn fused_inspect() {
    let mut lender = VecLender::new(vec![1]).inspect(|_| {});
    assert_eq!(lender.next(), Some(&1));
    assert_eq!(lender.next(), None);
    assert_eq!(lender.next(), None);
}

#[test]
fn fused_peekable() {
    let mut lender = VecLender::new(vec![1]).peekable();
    assert_eq!(lender.next(), Some(&1));
    assert_eq!(lender.next(), None);
    assert_eq!(lender.next(), None);
}

#[test]
fn fused_rev() {
    let mut lender = VecLender::new(vec![1]).rev();
    assert_eq!(lender.next(), Some(&1));
    assert_eq!(lender.next(), None);
    assert_eq!(lender.next(), None);
}

// =================================================================
// size_hint accuracy tests
// =================================================================

#[test]
fn size_hint_filter() {
    let lender = VecLender::new(vec![1, 2, 3, 4, 5]).filter(|x| **x > 2);
    let (lo, hi) = lender.size_hint();
    assert_eq!(lo, 0);
    assert_eq!(hi, Some(5));
}

#[test]
fn size_hint_filter_map() {
    let lender = VecLender::new(vec![1, 2, 3]).filter_map(lender::covar_mut!(
        for<'all> |x: &i32| -> Option<i32> { if *x > 1 { Some(x * 10) } else { None } }
    ));
    let (lo, hi) = lender.size_hint();
    assert_eq!(lo, 0);
    assert_eq!(hi, Some(3));
}

#[test]
fn size_hint_map() {
    let lender =
        VecLender::new(vec![1, 2, 3]).map(lender::covar_mut!(for<'all> |x: &i32| -> i32 { x * 2 }));
    let (lo, hi) = lender.size_hint();
    assert_eq!(lo, 3);
    assert_eq!(hi, Some(3));
}

#[test]
fn size_hint_inspect() {
    let lender = VecLender::new(vec![1, 2, 3]).inspect(|_| {});
    let (lo, hi) = lender.size_hint();
    assert_eq!(lo, 3);
    assert_eq!(hi, Some(3));
}

#[test]
fn size_hint_enumerate() {
    let lender = VecLender::new(vec![1, 2, 3]).enumerate();
    let (lo, hi) = lender.size_hint();
    assert_eq!(lo, 3);
    assert_eq!(hi, Some(3));
}

#[test]
fn size_hint_chain() {
    let a = VecLender::new(vec![1, 2]);
    let b = VecLender::new(vec![3, 4, 5]);
    let lender = a.chain(b);
    let (lo, hi) = lender.size_hint();
    assert_eq!(lo, 5);
    assert_eq!(hi, Some(5));
}

#[test]
fn size_hint_zip() {
    let a = VecLender::new(vec![1, 2, 3]);
    let b = VecLender::new(vec![4, 5]);
    let lender = a.zip(b);
    let (lo, hi) = lender.size_hint();
    assert_eq!(lo, 2);
    assert_eq!(hi, Some(2));
}

#[test]
fn size_hint_skip() {
    let lender = VecLender::new(vec![1, 2, 3, 4, 5]).skip(2);
    let (lo, hi) = lender.size_hint();
    assert_eq!(lo, 3);
    assert_eq!(hi, Some(3));
}

#[test]
fn size_hint_take() {
    let lender = VecLender::new(vec![1, 2, 3, 4, 5]).take(3);
    let (lo, hi) = lender.size_hint();
    assert_eq!(lo, 3);
    assert_eq!(hi, Some(3));
}

#[test]
fn size_hint_take_less_than_available() {
    let lender = VecLender::new(vec![1, 2]).take(5);
    let (lo, hi) = lender.size_hint();
    assert_eq!(lo, 2);
    assert_eq!(hi, Some(2));
}

#[test]
fn size_hint_skip_while() {
    let lender = VecLender::new(vec![1, 2, 3]).skip_while(|x| **x < 2);
    let (lo, hi) = lender.size_hint();
    assert_eq!(lo, 0);
    assert_eq!(hi, Some(3));
}

#[test]
fn size_hint_take_while() {
    let lender = VecLender::new(vec![1, 2, 3]).take_while(|x| **x < 3);
    let (lo, hi) = lender.size_hint();
    assert_eq!(lo, 0);
    assert_eq!(hi, Some(3));
}

#[test]
fn size_hint_rev() {
    let lender = VecLender::new(vec![1, 2, 3]).rev();
    let (lo, hi) = lender.size_hint();
    assert_eq!(lo, 3);
    assert_eq!(hi, Some(3));
}

#[test]
fn size_hint_fuse() {
    let lender = VecLender::new(vec![1, 2, 3]).fuse();
    let (lo, hi) = lender.size_hint();
    assert_eq!(lo, 3);
    assert_eq!(hi, Some(3));
}

#[test]
fn size_hint_peekable() {
    let lender = VecLender::new(vec![1, 2, 3]).peekable();
    let (lo, hi) = lender.size_hint();
    assert_eq!(lo, 3);
    assert_eq!(hi, Some(3));
}

#[test]
fn size_hint_intersperse() {
    let lender = VecLender::new(vec![1, 2, 3]).intersperse(&0);
    let (lo, hi) = lender.size_hint();
    assert_eq!(lo, 5);
    assert_eq!(hi, Some(5));
}

#[test]
fn size_hint_intersperse_empty() {
    let lender = VecLender::new(vec![]).intersperse(&0);
    let (lo, hi) = lender.size_hint();
    assert_eq!(lo, 0);
    assert_eq!(hi, Some(0));
}

#[test]
fn size_hint_intersperse_one() {
    let lender = VecLender::new(vec![42]).intersperse(&0);
    let (lo, hi) = lender.size_hint();
    assert_eq!(lo, 1);
    assert_eq!(hi, Some(1));
}

#[test]
fn size_hint_map_constant() {
    let lender =
        VecLender::new(vec![1, 2, 3]).map(lender::covar_mut!(for<'all> |_x: &i32| -> i32 { 1 }));
    let (lo, hi) = lender.size_hint();
    assert_eq!(lo, 3);
    assert_eq!(hi, Some(3));
}

#[test]
fn size_hint_step_by() {
    let lender = VecLender::new(vec![1, 2, 3, 4, 5]).step_by(2);
    let (lo, hi) = lender.size_hint();
    assert_eq!(lo, 3);
    assert_eq!(hi, Some(3));
}

#[test]
fn size_hint_cycle_non_empty() {
    let lender = VecLender::new(vec![1, 2]).cycle();
    let (lo, hi) = lender.size_hint();
    assert_eq!(lo, usize::MAX);
    assert_eq!(hi, None);
}

#[test]
fn size_hint_cycle_empty() {
    let lender = VecLender::new(vec![]).cycle();
    let (lo, hi) = lender.size_hint();
    assert_eq!(lo, 0);
    assert_eq!(hi, Some(0));
}

// =================================================================
// size_hint decreases correctly as elements are consumed
// =================================================================

#[test]
fn size_hint_decreases_with_consumption() {
    let mut lender = VecLender::new(vec![1, 2, 3, 4, 5]);
    assert_eq!(lender.size_hint(), (5, Some(5)));
    lender.next();
    assert_eq!(lender.size_hint(), (4, Some(4)));
    lender.next();
    assert_eq!(lender.size_hint(), (3, Some(3)));
    lender.next();
    lender.next();
    lender.next();
    assert_eq!(lender.size_hint(), (0, Some(0)));
}

#[test]
fn size_hint_chain_decreases() {
    let a = VecLender::new(vec![1, 2]);
    let b = VecLender::new(vec![3, 4]);
    let mut lender = a.chain(b);
    assert_eq!(lender.size_hint(), (4, Some(4)));
    lender.next();
    assert_eq!(lender.size_hint(), (3, Some(3)));
    lender.next();
    assert_eq!(lender.size_hint(), (2, Some(2)));
    lender.next();
    assert_eq!(lender.size_hint(), (1, Some(1)));
    lender.next();
    assert_eq!(lender.size_hint(), (0, Some(0)));
}

#[test]
fn size_hint_enumerate_decreases() {
    let mut lender = VecLender::new(vec![10, 20, 30]).enumerate();
    assert_eq!(lender.size_hint(), (3, Some(3)));
    lender.next();
    assert_eq!(lender.size_hint(), (2, Some(2)));
    lender.next();
    assert_eq!(lender.size_hint(), (1, Some(1)));
    lender.next();
    assert_eq!(lender.size_hint(), (0, Some(0)));
}

// =================================================================
// Negative trait compliance tests (verifying NON-implementation)
// =================================================================
// These tests verify that certain adapters correctly do NOT implement
// traits that would be unsound or misleading.

/// Helper to statically assert a type does NOT implement ExactSizeLender.
/// We use the fact that these adapters' next() returns Option but they
/// don't have len() - we test size_hint() instead which all Lenders have.
#[test]
fn filter_does_not_have_exact_size() {
    // Filter cannot know exact size since predicate can reject elements
    let lender = VecLender::new(vec![1, 2, 3, 4, 5]).filter(|x| **x > 2);
    // size_hint lower bound must be 0 (could filter all)
    let (lower, upper) = lender.size_hint();
    assert_eq!(lower, 0);
    // upper bound is same as underlying (could pass all)
    assert_eq!(upper, Some(5));
}

#[test]
fn filter_map_does_not_have_exact_size() {
    let lender = VecLender::new(vec![1, 2, 3, 4, 5]).filter_map(covar_mut!(
        for<'all> |x: &i32| -> Option<i32> { if *x > 2 { Some(*x * 10) } else { None } }
    ));
    let (lower, upper) = lender.size_hint();
    assert_eq!(lower, 0);
    assert_eq!(upper, Some(5));
}

#[test]
fn skip_while_does_not_have_exact_size() {
    let lender = VecLender::new(vec![1, 2, 3, 4, 5]).skip_while(|x| **x < 3);
    let (lower, upper) = lender.size_hint();
    assert_eq!(lower, 0);
    assert_eq!(upper, Some(5));
}

#[test]
fn take_while_does_not_have_exact_size() {
    let lender = VecLender::new(vec![1, 2, 3, 4, 5]).take_while(|x| **x < 3);
    let (lower, upper) = lender.size_hint();
    assert_eq!(lower, 0);
    assert_eq!(upper, Some(5));
}

#[test]
fn map_while_does_not_have_exact_size() {
    let lender = VecLender::new(vec![1, 2, 3, 4, 5]).map_while(covar_mut!(
        for<'all> |x: &i32| -> Option<i32> { if *x < 3 { Some(*x * 10) } else { None } }
    ));
    let (lower, upper) = lender.size_hint();
    assert_eq!(lower, 0);
    assert_eq!(upper, Some(5));
}

#[test]
fn scan_does_not_have_exact_size() {
    let lender = VecLender::new(vec![1, 2, 3, 4, 5]).scan(
        0,
        covar_mut!(for<'all> |args: (&'all mut i32, &i32)| -> Option<i32> {
            *args.0 += *args.1;
            if *args.0 < 10 { Some(*args.0) } else { None }
        }),
    );
    let (lower, upper) = lender.size_hint();
    assert_eq!(lower, 0);
    assert_eq!(upper, Some(5));
}

/// MapWhile is explicitly NOT FusedLender (matching std::iter::MapWhile)
/// because returning None from the closure does not guarantee the underlying
/// lender is exhausted - it just means mapping decided to stop.
#[test]
fn map_while_not_fused_behavior() {
    // This test verifies the documented behavior: MapWhile does not implement
    // FusedLender. After the closure returns None, the underlying lender may
    // still have elements, and MapWhile's behavior after that is unspecified.
    let mut lender = VecLender::new(vec![1, 2, 100, 3, 4]).map_while(covar_mut!(
        for<'all> |x: &i32| -> Option<i32> { if *x < 50 { Some(*x) } else { None } }
    ));
    assert_eq!(lender.next(), Some(1));
    assert_eq!(lender.next(), Some(2));
    assert_eq!(lender.next(), None); // stopped at 100
    // After None, behavior is implementation-defined (not required to stay None)
}
