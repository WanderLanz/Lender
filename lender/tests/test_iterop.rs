mod common;
use ::lender::prelude::*;
use common::*;

// ============================================================================
// MapIntoIter adapter tests - converts lender to iterator via mapping
// ============================================================================

#[test]
fn map_into_iter_basic() {
    // MapIntoIter converts a lender to an iterator by applying a function
    let iter = VecLender::new(vec![1, 2, 3]).map_into_iter(|x| *x * 2);
    let collected: Vec<i32> = iter.collect();
    assert_eq!(collected, vec![2, 4, 6]);
}

#[test]
fn map_into_iter_size_hint() {
    let iter = VecLender::new(vec![1, 2, 3, 4, 5]).map_into_iter(|x| *x);
    assert_eq!(iter.size_hint(), (5, Some(5)));
}

#[test]
fn map_into_iter_double_ended() {
    let mut iter = VecLender::new(vec![1, 2, 3]).map_into_iter(|x| *x * 2);
    assert_eq!(iter.next_back(), Some(6));
    assert_eq!(iter.next(), Some(2));
    assert_eq!(iter.next_back(), Some(4));
    assert_eq!(iter.next(), None);
}

#[test]
fn map_into_iter_exact_size() {
    let iter = VecLender::new(vec![1, 2, 3]).map_into_iter(|x| *x);
    assert_eq!(iter.len(), 3);
}

#[test]
fn map_into_iter_into_inner() {
    let iter = VecLender::new(vec![1, 2, 3]).map_into_iter(|x| *x * 2);
    let lender = iter.into_inner();
    assert_eq!(lender.count(), 3);
}

#[test]
fn map_into_iter_into_parts() {
    let iter = VecLender::new(vec![1, 2, 3]).map_into_iter(|x| *x * 2);
    let (lender, _f) = iter.into_parts();
    assert_eq!(lender.count(), 3);
}

#[test]
fn map_into_iter_fold() {
    let sum = VecLender::new(vec![1, 2, 3])
        .map_into_iter(|x| *x * 2)
        .sum::<i32>();
    assert_eq!(sum, 12);
}

#[test]
fn map_into_iter_fold_empty() {
    let sum = VecLender::new(vec![])
        .map_into_iter(|x: &i32| *x * 2)
        .sum::<i32>();
    assert_eq!(sum, 0);
}

#[test]
fn map_into_iter_rfold() {
    let result = VecLender::new(vec![1, 2, 3])
        .map_into_iter(|x| *x * 2)
        .rfold(Vec::new(), |mut acc, x| {
            acc.push(x);
            acc
        });
    assert_eq!(result, vec![6, 4, 2]);
}

#[test]
fn map_into_iter_rfold_empty() {
    let result = VecLender::new(vec![])
        .map_into_iter(|x: &i32| *x * 2)
        .rfold(Vec::new(), |mut acc, x| {
            acc.push(x);
            acc
        });
    assert!(result.is_empty());
}

// ============================================================================
// Iter adapter tests - converts lender to iterator when Lend is 'static-like
// ============================================================================

#[test]
fn iter_adapter_basic() {
    // Iter works when the lend type can outlive the lender borrow
    // Using from_iter which yields owned values
    let iter = vec![1, 2, 3].into_iter().into_lender().iter();
    let collected: Vec<i32> = iter.collect();
    assert_eq!(collected, vec![1, 2, 3]);
}

#[test]
fn iter_adapter_size_hint() {
    let iter = vec![1, 2, 3, 4, 5].into_iter().into_lender().iter();
    assert_eq!(iter.size_hint(), (5, Some(5)));
}

#[test]
fn iter_adapter_double_ended() {
    let mut iter = vec![1, 2, 3].into_iter().into_lender().iter();
    assert_eq!(iter.next_back(), Some(3));
    assert_eq!(iter.next(), Some(1));
    assert_eq!(iter.next_back(), Some(2));
    assert_eq!(iter.next(), None);
}

#[test]
fn iter_adapter_exact_size() {
    let iter = vec![1, 2, 3].into_iter().into_lender().iter();
    assert_eq!(iter.len(), 3);
}

#[test]
fn iter_adapter_into_inner() {
    let iter = vec![1, 2, 3].into_iter().into_lender().iter();
    let lender = iter.into_inner();
    assert_eq!(lender.count(), 3);
}

// Note: Owned adapter has complex HRTB trait bounds that are difficult to satisfy
// in tests with lend_iter. The owned() method works with specific lenders that have
// the right ToOwned implementations. Coverage is partial due to these constraints.
