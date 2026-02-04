//! Tests for element selection adapters: StepBy, Filter, FilterMap, Skip, SkipWhile, Take, TakeWhile

#![allow(clippy::unnecessary_fold)]

mod common;
use ::lender::prelude::*;
use common::*;

// ============================================================================
// StepBy adapter tests (Lender)
// Semantics: step_by(n) returns every nth element, starting with the first.
// ============================================================================

#[test]
fn step_by_basic() {
    // Documented example: step_by(2) yields elements at indices 0, 2, 4, 6, 8...
    let mut stepped = VecLender::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]).step_by(2);

    assert_eq!(stepped.next(), Some(&1)); // index 0
    assert_eq!(stepped.next(), Some(&3)); // index 2
    assert_eq!(stepped.next(), Some(&5)); // index 4
    assert_eq!(stepped.next(), Some(&7)); // index 6
    assert_eq!(stepped.next(), Some(&9)); // index 8
    assert_eq!(stepped.next(), None);
}

#[test]
fn step_by_step_1() {
    // step_by(1) should yield all elements
    let mut stepped = VecLender::new(vec![1, 2, 3]).step_by(1);

    assert_eq!(stepped.next(), Some(&1));
    assert_eq!(stepped.next(), Some(&2));
    assert_eq!(stepped.next(), Some(&3));
    assert_eq!(stepped.next(), None);
}

#[test]
fn step_by_step_larger_than_len() {
    // step_by(10) on 3 elements: only first element
    let mut stepped = VecLender::new(vec![1, 2, 3]).step_by(10);

    assert_eq!(stepped.next(), Some(&1));
    assert_eq!(stepped.next(), None);
}

#[test]
fn step_by_empty() {
    let mut stepped = VecLender::new(vec![]).step_by(2);
    assert_eq!(stepped.next(), None);
}

#[test]
#[should_panic(expected = "assertion `left != right` failed")]
fn step_by_zero_panics() {
    // step_by(0) should panic
    let _ = VecLender::new(vec![1, 2, 3]).step_by(0);
}

#[test]
fn step_by_double_ended() {
    // DoubleEndedLender: next_back on step_by(2) should yield last element at
    // a step position, working backwards
    // For [1,2,3,4,5,6] with step 2: positions 0,2,4 -> values 1,3,5
    // next_back should return 5, 3, 1
    let mut stepped = VecLender::new(vec![1, 2, 3, 4, 5, 6]).step_by(2);

    assert_eq!(stepped.next_back(), Some(&5)); // last step position
    assert_eq!(stepped.next_back(), Some(&3));
    assert_eq!(stepped.next_back(), Some(&1));
    assert_eq!(stepped.next_back(), None);
}

#[test]
fn step_by_mixed_forward_backward() {
    // Mixed iteration
    let mut stepped = VecLender::new(vec![1, 2, 3, 4, 5, 6, 7]).step_by(2);
    // Positions: 0,2,4,6 -> values 1,3,5,7

    assert_eq!(stepped.next(), Some(&1)); // front: 1
    assert_eq!(stepped.next_back(), Some(&7)); // back: 7
    assert_eq!(stepped.next(), Some(&3)); // front: 3
    assert_eq!(stepped.next_back(), Some(&5)); // back: 5
    assert_eq!(stepped.next(), None);
    assert_eq!(stepped.next_back(), None);
}

#[test]
fn step_by_nth() {
    // nth on step_by should skip appropriately
    let mut stepped = VecLender::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9]).step_by(2);
    // Positions: 0,2,4,6,8 -> values 1,3,5,7,9

    // nth(0) is same as next()
    assert_eq!(stepped.nth(0), Some(&1));
    // nth(1) skips one step position (3) and returns next (5)
    assert_eq!(stepped.nth(1), Some(&5));
    // nth(0) returns 7
    assert_eq!(stepped.nth(0), Some(&7));
    // nth(1) would need to skip 9 and get next, but only 9 left
    assert_eq!(stepped.nth(1), None);
}

#[test]
fn step_by_nth_back() {
    let mut stepped = VecLender::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9]).step_by(2);
    // Positions: 0,2,4,6,8 -> values 1,3,5,7,9

    // nth_back(0) returns last step position (9)
    assert_eq!(stepped.nth_back(0), Some(&9));
    // nth_back(1) skips 7 and returns 5
    assert_eq!(stepped.nth_back(1), Some(&5));
    // nth_back(0) returns 3
    assert_eq!(stepped.nth_back(0), Some(&3));
    // nth_back(0) returns 1
    assert_eq!(stepped.nth_back(0), Some(&1));
    assert_eq!(stepped.nth_back(0), None);
}

#[test]
fn step_by_size_hint() {
    // For [1,2,3,4,5] with step 2: positions 0,2,4 -> 3 elements
    let stepped = VecLender::new(vec![1, 2, 3, 4, 5]).step_by(2);
    assert_eq!(stepped.size_hint(), (3, Some(3)));

    // For [1,2,3,4,5,6] with step 2: positions 0,2,4 -> 3 elements
    let stepped2 = VecLender::new(vec![1, 2, 3, 4, 5, 6]).step_by(2);
    assert_eq!(stepped2.size_hint(), (3, Some(3)));

    // For [1,2,3,4,5,6,7] with step 2: positions 0,2,4,6 -> 4 elements
    let stepped3 = VecLender::new(vec![1, 2, 3, 4, 5, 6, 7]).step_by(2);
    assert_eq!(stepped3.size_hint(), (4, Some(4)));

    // Empty
    let stepped_empty = VecLender::new(vec![]).step_by(2);
    assert_eq!(stepped_empty.size_hint(), (0, Some(0)));
}

#[test]
fn step_by_size_hint_after_iteration() {
    let mut stepped = VecLender::new(vec![1, 2, 3, 4, 5]).step_by(2);
    // Initially: positions 0,2,4 -> 3 elements
    assert_eq!(stepped.size_hint(), (3, Some(3)));

    stepped.next(); // consumed position 0
    assert_eq!(stepped.size_hint(), (2, Some(2)));

    stepped.next(); // consumed position 2
    assert_eq!(stepped.size_hint(), (1, Some(1)));

    stepped.next(); // consumed position 4
    assert_eq!(stepped.size_hint(), (0, Some(0)));
}

#[test]
fn step_by_fold() {
    let sum = VecLender::new(vec![1, 2, 3, 4, 5, 6])
        .step_by(2)
        .fold(0, |acc, x| acc + *x);
    // Positions 0,2,4 -> values 1,3,5 -> sum = 9
    assert_eq!(sum, 9);
}

#[test]
fn step_by_rfold() {
    let mut order = Vec::new();
    VecLender::new(vec![1, 2, 3, 4, 5, 6])
        .step_by(2)
        .rfold((), |(), x| order.push(*x));
    // Positions 0,2,4 -> values 1,3,5, reversed -> [5, 3, 1]
    assert_eq!(order, vec![5, 3, 1]);
}

#[test]
fn step_by_exact_size() {
    use lender::ExactSizeLender;

    let mut stepped = VecLender::new(vec![1, 2, 3, 4, 5, 6, 7]).step_by(2);
    // Positions 0,2,4,6 -> 4 elements
    assert_eq!(stepped.len(), 4);
    assert!(!stepped.is_empty());

    stepped.next();
    assert_eq!(stepped.len(), 3);

    stepped.next();
    stepped.next();
    stepped.next();
    assert_eq!(stepped.len(), 0);
    assert!(stepped.is_empty());
}

#[test]
fn step_by_into_inner() {
    let stepped = VecLender::new(vec![1, 2, 3]).step_by(2);
    let inner = stepped.into_inner();
    assert_eq!(inner.data, vec![1, 2, 3]);
}

#[test]
fn step_by_into_parts() {
    // into_parts should return the original step value (not the internal step - 1)
    let stepped = VecLender::new(vec![1, 2, 3]).step_by(3);
    let (inner, step) = stepped.into_parts();
    assert_eq!(inner.data, vec![1, 2, 3]);
    assert_eq!(step, 3);

    let stepped2 = VecLender::new(vec![1]).step_by(1);
    let (_, step2) = stepped2.into_parts();
    assert_eq!(step2, 1);
}

#[test]
fn step_by_fold_additional() {
    let sum = VecLender::new(vec![1, 2, 3, 4, 5])
        .step_by(2)
        .fold(0, |acc, x| acc + *x);
    // Elements: 1, 3, 5
    assert_eq!(sum, 9);
}

#[test]
fn step_by_try_fold_additional() {
    let result: Option<i32> = VecLender::new(vec![1, 2, 3, 4, 5])
        .step_by(2)
        .try_fold(0, |acc, x| Some(acc + *x));
    assert_eq!(result, Some(9));
}

// ============================================================================
// Filter adapter tests (Lender)
// ============================================================================

#[test]
fn filter_basic() {
    let mut filtered = VecLender::new(vec![1, 2, 3, 4, 5, 6]).filter(|x| **x % 2 == 0);

    assert_eq!(filtered.next(), Some(&2));
    assert_eq!(filtered.next(), Some(&4));
    assert_eq!(filtered.next(), Some(&6));
    assert_eq!(filtered.next(), None);
}

#[test]
fn filter_empty_result() {
    let mut filtered = VecLender::new(vec![1, 3, 5]).filter(|x| **x % 2 == 0);
    assert_eq!(filtered.next(), None);
}

#[test]
fn filter_fold() {
    let sum = VecLender::new(vec![1, 2, 3, 4, 5, 6])
        .filter(|x| **x % 2 == 0)
        .fold(0, |acc, x| acc + *x);
    // 2 + 4 + 6 = 12
    assert_eq!(sum, 12);
}

#[test]
fn filter_double_ended() {
    let mut filtered = VecLender::new(vec![1, 2, 3, 4, 5, 6]).filter(|x| **x % 2 == 0);

    assert_eq!(filtered.next_back(), Some(&6));
    assert_eq!(filtered.next(), Some(&2));
    assert_eq!(filtered.next_back(), Some(&4));
    assert_eq!(filtered.next(), None);
}

#[test]
fn filter_fold_additional() {
    let sum = VecLender::new(vec![1, 2, 3, 4, 5, 6])
        .filter(|x| **x % 2 == 0)
        .fold(0, |acc, x| acc + *x);
    assert_eq!(sum, 12); // 2 + 4 + 6
}

#[test]
fn filter_try_fold_additional() {
    let result: Option<i32> = VecLender::new(vec![1, 2, 3, 4, 5])
        .filter(|x| **x % 2 == 1)
        .try_fold(0, |acc, x| Some(acc + *x));
    assert_eq!(result, Some(9)); // 1 + 3 + 5
}

#[test]
fn filter_rfold_additional() {
    use lender::DoubleEndedLender;

    let values: Vec<i32> = VecLender::new(vec![1, 2, 3, 4, 5, 6])
        .filter(|x| **x % 2 == 0)
        .rfold(Vec::new(), |mut acc, x| {
            acc.push(*x);
            acc
        });
    assert_eq!(values, vec![6, 4, 2]);
}

#[test]
fn filter_into_inner() {
    let filter = VecLender::new(vec![1, 2, 3, 4, 5]).filter(|x| **x % 2 == 0);
    let lender = filter.into_inner();
    assert_eq!(lender.count(), 5);
}

#[test]
fn filter_into_parts_additional() {
    let filter = VecLender::new(vec![1, 2, 3, 4, 5]).filter(|x| **x % 2 == 0);
    let (lender, _predicate) = filter.into_parts();
    assert_eq!(lender.count(), 5);
}

#[test]
fn filter_try_rfold_additional() {
    let result: Option<i32> = VecLender::new(vec![1, 2, 3, 4, 5, 6])
        .filter(|x| **x % 2 == 0)
        .try_rfold(0, |acc, x| Some(acc + *x));
    // Even numbers in reverse: 6, 4, 2
    assert_eq!(result, Some(12));
}

// ============================================================================
// FilterMap adapter tests
// Semantics: filter and map in one operation
// ============================================================================

#[test]
fn filter_map_basic() {
    let mut fm = VecLender::new(vec![1, 2, 3, 4, 5])
        .filter_map(|x: &i32| if *x % 2 == 0 { Some(*x * 10) } else { None });

    assert_eq!(fm.next(), Some(20));
    assert_eq!(fm.next(), Some(40));
    assert_eq!(fm.next(), None);
}

#[test]
fn filter_map_all_none() {
    let mut fm = VecLender::new(vec![1, 3, 5])
        .filter_map(|x: &i32| if *x % 2 == 0 { Some(*x * 10) } else { None });
    assert_eq!(fm.next(), None);
}

#[test]
fn filter_map_all_some() {
    let mut fm = VecLender::new(vec![2, 4, 6]).filter_map(|x: &i32| Some(*x / 2));

    assert_eq!(fm.next(), Some(1));
    assert_eq!(fm.next(), Some(2));
    assert_eq!(fm.next(), Some(3));
    assert_eq!(fm.next(), None);
}

#[test]
fn filter_map_fold() {
    let sum = VecLender::new(vec![1, 2, 3, 4, 5])
        .filter_map(|x: &i32| if *x % 2 == 0 { Some(*x) } else { None })
        .fold(0, |acc, x| acc + x);
    // 2 + 4 = 6
    assert_eq!(sum, 6);
}

#[test]
fn filter_map_into_inner() {
    let filter_map =
        VecLender::new(vec![1, 2, 3]).filter_map(hrc_mut!(for<'all> |x: &i32| -> Option<i32> {
            if *x % 2 == 0 { Some(*x * 2) } else { None }
        }));
    let lender = filter_map.into_inner();
    assert_eq!(lender.count(), 3);
}

#[test]
fn filter_map_into_parts() {
    let filter_map = VecLender::new(vec![1, 2, 3])
        .filter_map(hrc_mut!(for<'all> |x: &i32| -> Option<i32> { Some(*x) }));
    let (lender, _f) = filter_map.into_parts();
    assert_eq!(lender.count(), 3);
}

// FilterMap unsafe transmute (covers unsafe at lines 50, 70)
#[test]
fn filter_map_double_ended_coverage() {
    use lender::DoubleEndedLender;

    let mut fm = VecLender::new(vec![1, 2, 3, 4, 5])
        .filter_map(|x: &i32| if *x % 2 == 0 { Some(*x * 2) } else { None });
    // Use next_back to exercise the DoubleEndedLender unsafe path
    assert_eq!(fm.next_back(), Some(8)); // 4 * 2
    assert_eq!(fm.next(), Some(4)); // 2 * 2
    assert_eq!(fm.next_back(), None);
}

// ============================================================================
// Skip adapter tests
// ============================================================================

#[test]
fn skip_basic() {
    let mut skipped = VecLender::new(vec![1, 2, 3, 4, 5]).skip(2);

    assert_eq!(skipped.next(), Some(&3));
    assert_eq!(skipped.next(), Some(&4));
    assert_eq!(skipped.next(), Some(&5));
    assert_eq!(skipped.next(), None);
}

#[test]
fn skip_more_than_length() {
    let mut skipped = VecLender::new(vec![1, 2, 3]).skip(10);
    assert_eq!(skipped.next(), None);
}

#[test]
fn skip_double_ended() {
    let mut skipped = VecLender::new(vec![1, 2, 3, 4, 5]).skip(2);

    // next_back should return from back of remaining
    assert_eq!(skipped.next_back(), Some(&5));
    assert_eq!(skipped.next_back(), Some(&4));
    assert_eq!(skipped.next_back(), Some(&3));
    assert_eq!(skipped.next_back(), None);
}

#[test]
fn skip_nth() {
    let mut skipped = VecLender::new(vec![1, 2, 3, 4, 5]).skip(2);
    // After skip(2), elements are 3, 4, 5
    // nth(1) skips 3 and returns 4
    assert_eq!(skipped.nth(1), Some(&4));
    assert_eq!(skipped.next(), Some(&5));
}

#[test]
fn skip_fold() {
    let sum = VecLender::new(vec![1, 2, 3, 4, 5])
        .skip(2)
        .fold(0, |acc, x| acc + *x);
    // 3 + 4 + 5 = 12
    assert_eq!(sum, 12);
}

#[test]
fn skip_into_inner() {
    let skip = VecLender::new(vec![1, 2, 3, 4, 5]).skip(2);
    let lender = skip.into_inner();
    assert_eq!(lender.count(), 5);
}

#[test]
fn skip_into_parts() {
    let skip = VecLender::new(vec![1, 2, 3, 4, 5]).skip(2);
    let (lender, n) = skip.into_parts();
    assert_eq!(lender.count(), 5);
    assert_eq!(n, 2);
}

#[test]
fn skip_fold_additional() {
    let sum = VecLender::new(vec![1, 2, 3, 4, 5])
        .skip(2)
        .fold(0, |acc, x| acc + *x);
    // Skips 1, 2; sums 3 + 4 + 5 = 12
    assert_eq!(sum, 12);
}

#[test]
fn skip_try_fold_additional() {
    let result: Option<i32> = VecLender::new(vec![1, 2, 3, 4, 5])
        .skip(2)
        .try_fold(0, |acc, x| Some(acc + *x));
    assert_eq!(result, Some(12));
}

#[test]
fn skip_nth_additional() {
    let mut skipped = VecLender::new(vec![1, 2, 3, 4, 5]).skip(2);
    // After skip(2), we have [3, 4, 5]
    assert_eq!(skipped.nth(1), Some(&4)); // [3, 4, 5] -> nth(1) = 4
}

#[test]
fn skip_rfold_additional() {
    use lender::DoubleEndedLender;

    let values: Vec<i32> =
        VecLender::new(vec![1, 2, 3, 4, 5])
            .skip(2)
            .rfold(Vec::new(), |mut acc, x| {
                acc.push(*x);
                acc
            });
    // Skips 1, 2; rfolds 5, 4, 3
    assert_eq!(values, vec![5, 4, 3]);
}

#[test]
fn skip_rfold() {
    let mut values = Vec::new();
    VecLender::new(vec![1, 2, 3, 4, 5])
        .skip(2)
        .rfold((), |(), x| {
            values.push(*x);
        });
    // skip(2) leaves [3, 4, 5], rfold processes: 5, 4, 3
    assert_eq!(values, vec![5, 4, 3]);
}

#[test]
fn skip_advance_back_by() {
    use lender::DoubleEndedLender;

    let mut skip = VecLender::new(vec![1, 2, 3, 4, 5]).skip(2);
    // skip(2) leaves [3, 4, 5], advance_back_by(2) skips 5, 4
    assert_eq!(skip.advance_back_by(2), Ok(()));
    assert_eq!(skip.next(), Some(&3));
    assert_eq!(skip.next(), None);
}

#[test]
fn skip_advance_back_by_exact() {
    use lender::DoubleEndedLender;

    let mut skip = VecLender::new(vec![1, 2, 3, 4, 5]).skip(2);
    // skip(2) leaves [3, 4, 5], advance_back_by(3) exhausts all
    assert_eq!(skip.advance_back_by(3), Ok(()));
    assert_eq!(skip.next(), None);
}

#[test]
fn skip_advance_back_by_past_end() {
    use core::num::NonZeroUsize;
    use lender::DoubleEndedLender;

    let mut skip = VecLender::new(vec![1, 2, 3, 4, 5]).skip(2);
    // skip(2) leaves 3 elements, trying to advance 5 should fail with 2 remaining
    assert_eq!(skip.advance_back_by(5), Err(NonZeroUsize::new(2).unwrap()));
}

// ============================================================================
// SkipWhile adapter tests
// Semantics: skip elements while predicate is true, then yield all remaining
// ============================================================================

#[test]
fn skip_while_basic() {
    let mut skipped = VecLender::new(vec![1, 2, 3, 4, 5]).skip_while(|x| **x < 3);

    // Skips 1, 2; yields 3, 4, 5
    assert_eq!(skipped.next(), Some(&3));
    assert_eq!(skipped.next(), Some(&4));
    assert_eq!(skipped.next(), Some(&5));
    assert_eq!(skipped.next(), None);
}

#[test]
fn skip_while_none_skipped() {
    // Predicate is false from the start
    let mut skipped = VecLender::new(vec![5, 4, 3, 2, 1]).skip_while(|x| **x < 3);

    assert_eq!(skipped.next(), Some(&5));
    assert_eq!(skipped.next(), Some(&4));
}

#[test]
fn skip_while_all_skipped() {
    // Predicate is always true
    let mut skipped = VecLender::new(vec![1, 2, 3]).skip_while(|x| **x < 10);
    assert_eq!(skipped.next(), None);
}

#[test]
fn skip_while_empty() {
    let mut skipped = VecLender::new(vec![]).skip_while(|x| **x < 3);
    assert_eq!(skipped.next(), None);
}

#[test]
fn skip_while_fold() {
    let sum = VecLender::new(vec![1, 2, 3, 4, 5])
        .skip_while(|x| **x < 3)
        .fold(0, |acc, x| acc + *x);
    // 3 + 4 + 5 = 12
    assert_eq!(sum, 12);
}

#[test]
fn skip_while_into_inner() {
    let skip_while = VecLender::new(vec![1, 2, 3, 4, 5]).skip_while(|x| **x < 3);
    let lender = skip_while.into_inner();
    assert_eq!(lender.count(), 5);
}

#[test]
fn skip_while_into_parts() {
    let skip_while = VecLender::new(vec![1, 2, 3, 4, 5]).skip_while(|x| **x < 3);
    let (lender, _predicate) = skip_while.into_parts();
    assert_eq!(lender.count(), 5);
}

#[test]
fn skip_while_fold_additional() {
    let sum = VecLender::new(vec![1, 2, 3, 4, 5])
        .skip_while(|x| **x < 3)
        .fold(0, |acc, x| acc + *x);
    // Skips 1, 2; sums 3 + 4 + 5 = 12
    assert_eq!(sum, 12);
}

#[test]
fn skip_while_try_fold_additional() {
    let result: Option<i32> = VecLender::new(vec![1, 2, 3, 4, 5])
        .skip_while(|x| **x < 3)
        .try_fold(0, |acc, x| Some(acc + *x));
    assert_eq!(result, Some(12));
}

// ============================================================================
// Take adapter tests
// ============================================================================

#[test]
fn take_basic() {
    let mut taken = VecLender::new(vec![1, 2, 3, 4, 5]).take(3);

    assert_eq!(taken.next(), Some(&1));
    assert_eq!(taken.next(), Some(&2));
    assert_eq!(taken.next(), Some(&3));
    assert_eq!(taken.next(), None);
}

#[test]
fn take_more_than_length() {
    let mut taken = VecLender::new(vec![1, 2]).take(10);

    assert_eq!(taken.next(), Some(&1));
    assert_eq!(taken.next(), Some(&2));
    assert_eq!(taken.next(), None);
}

#[test]
fn take_double_ended() {
    let mut taken = VecLender::new(vec![1, 2, 3, 4, 5]).take(3);

    // next_back returns from back of taken portion
    assert_eq!(taken.next_back(), Some(&3));
    assert_eq!(taken.next_back(), Some(&2));
    assert_eq!(taken.next_back(), Some(&1));
    assert_eq!(taken.next_back(), None);
}

#[test]
fn take_nth() {
    let mut taken = VecLender::new(vec![1, 2, 3, 4, 5]).take(4);
    // nth(2) skips 1, 2 and returns 3
    assert_eq!(taken.nth(2), Some(&3));
    assert_eq!(taken.next(), Some(&4));
    assert_eq!(taken.next(), None);
}

#[test]
fn take_fold() {
    let sum = VecLender::new(vec![1, 2, 3, 4, 5])
        .take(3)
        .fold(0, |acc, x| acc + *x);
    // 1 + 2 + 3 = 6
    assert_eq!(sum, 6);
}

#[test]
fn take_into_inner() {
    let take = VecLender::new(vec![1, 2, 3, 4, 5]).take(3);
    let lender = take.into_inner();
    assert_eq!(lender.count(), 5);
}

#[test]
fn take_into_parts() {
    let take = VecLender::new(vec![1, 2, 3, 4, 5]).take(3);
    let (lender, n) = take.into_parts();
    assert_eq!(lender.count(), 5);
    assert_eq!(n, 3);
}

#[test]
fn take_try_fold_additional() {
    let result: Option<i32> = VecLender::new(vec![1, 2, 3, 4, 5])
        .take(3)
        .try_fold(0, |acc, x| Some(acc + *x));
    // Takes 1, 2, 3
    assert_eq!(result, Some(6));
}

#[test]
fn take_try_rfold_additional() {
    let result: Option<i32> = VecLender::new(vec![1, 2, 3, 4, 5])
        .take(3)
        .try_rfold(0, |acc, x| Some(acc + *x));
    assert_eq!(result, Some(6));
}

#[test]
fn advance_by_take_additional() {
    let mut taken = VecLender::new(vec![1, 2, 3, 4, 5]).take(4);
    assert_eq!(taken.advance_by(3), Ok(())); // Skip 1, 2, 3
    assert_eq!(taken.next(), Some(&4));
}

#[test]
fn advance_back_by_take_additional() {
    use lender::DoubleEndedLender;

    let mut taken = VecLender::new(vec![1, 2, 3, 4, 5]).take(4);
    assert_eq!(taken.advance_back_by(2), Ok(())); // Skip 4, 3
    assert_eq!(taken.next_back(), Some(&2));
}

#[test]
fn take_rfold() {
    let mut values = Vec::new();
    VecLender::new(vec![1, 2, 3, 4, 5])
        .take(3)
        .rfold((), |(), x| {
            values.push(*x);
        });
    // take(3) gives [1, 2, 3], rfold processes: 3, 2, 1
    assert_eq!(values, vec![3, 2, 1]);
}

// ============================================================================
// TakeWhile adapter tests
// Semantics: yield elements while predicate is true, then stop
// ============================================================================

#[test]
fn take_while_basic() {
    let mut taken = VecLender::new(vec![1, 2, 3, 4, 5]).take_while(|x| **x < 4);

    assert_eq!(taken.next(), Some(&1));
    assert_eq!(taken.next(), Some(&2));
    assert_eq!(taken.next(), Some(&3));
    assert_eq!(taken.next(), None);
    // Should stay None even though 4, 5 remain in underlying lender
    assert_eq!(taken.next(), None);
}

#[test]
fn take_while_none_taken() {
    // Predicate is false from the start
    let mut taken = VecLender::new(vec![5, 4, 3]).take_while(|x| **x < 3);
    assert_eq!(taken.next(), None);
}

#[test]
fn take_while_all_taken() {
    // Predicate is always true
    let mut taken = VecLender::new(vec![1, 2, 3]).take_while(|x| **x < 10);

    assert_eq!(taken.next(), Some(&1));
    assert_eq!(taken.next(), Some(&2));
    assert_eq!(taken.next(), Some(&3));
    assert_eq!(taken.next(), None);
}

#[test]
fn take_while_empty() {
    let mut taken = VecLender::new(vec![]).take_while(|x| **x < 3);
    assert_eq!(taken.next(), None);
}

#[test]
fn take_while_fold() {
    let sum = VecLender::new(vec![1, 2, 3, 4, 5])
        .take_while(|x| **x < 4)
        .fold(0, |acc, x| acc + *x);
    // 1 + 2 + 3 = 6
    assert_eq!(sum, 6);
}

#[test]
fn take_while_into_inner() {
    let take_while = VecLender::new(vec![1, 2, 3, 4, 5]).take_while(|x| **x < 3);
    let lender = take_while.into_inner();
    assert_eq!(lender.count(), 5);
}

#[test]
fn take_while_into_parts() {
    let take_while = VecLender::new(vec![1, 2, 3, 4, 5]).take_while(|x| **x < 3);
    let (lender, _predicate) = take_while.into_parts();
    assert_eq!(lender.count(), 5);
}

#[test]
fn take_while_try_fold_additional() {
    let result: Option<i32> = VecLender::new(vec![1, 2, 3, 4, 5])
        .take_while(|x| **x < 4)
        .try_fold(0, |acc, x| Some(acc + *x));
    // Takes 1, 2, 3 (until 4 fails condition)
    assert_eq!(result, Some(6));
}

