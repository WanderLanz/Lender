//! Tests for element transformation adapters: Map, Enumerate, Inspect, Mutate, Scan, MapWhile, Cloned, Copied, Owned

#![allow(clippy::unnecessary_fold)]

mod common;
use ::lender::prelude::*;
use common::*;

// ============================================================================
// Enumerate adapter tests (Lender)
// Semantics: enumerate() pairs each element with its index starting from 0
// ============================================================================

#[test]
fn enumerate_basic() {
    let mut enumerated = VecLender::new(vec![10, 20, 30]).enumerate();

    assert_eq!(enumerated.next(), Some((0, &10)));
    assert_eq!(enumerated.next(), Some((1, &20)));
    assert_eq!(enumerated.next(), Some((2, &30)));
    assert_eq!(enumerated.next(), None);
}

#[test]
fn enumerate_empty() {
    let mut enumerated = VecLender::new(vec![]).enumerate();
    assert_eq!(enumerated.next(), None);
}

#[test]
fn enumerate_double_ended() {
    let mut enumerated = VecLender::new(vec![10, 20, 30, 40]).enumerate();

    // next_back yields from back with correct indices
    assert_eq!(enumerated.next_back(), Some((3, &40)));
    assert_eq!(enumerated.next_back(), Some((2, &30)));
    assert_eq!(enumerated.next(), Some((0, &10)));
    assert_eq!(enumerated.next(), Some((1, &20)));
    assert_eq!(enumerated.next(), None);
    assert_eq!(enumerated.next_back(), None);
}

#[test]
fn enumerate_size_hint() {
    let enumerated = VecLender::new(vec![1, 2, 3]).enumerate();
    assert_eq!(enumerated.size_hint(), (3, Some(3)));
}

#[test]
fn enumerate_nth() {
    let mut enumerated = VecLender::new(vec![10, 20, 30, 40, 50]).enumerate();

    // nth(2) skips (0,10) and (1,20), returns (2,30)
    assert_eq!(enumerated.nth(2), Some((2, &30)));
    // nth(0) returns next
    assert_eq!(enumerated.nth(0), Some((3, &40)));
}

#[test]
fn enumerate_nth_back() {
    let mut enumerated = VecLender::new(vec![10, 20, 30, 40, 50]).enumerate();

    // nth_back(1) skips (4,50), returns (3,40)
    assert_eq!(enumerated.nth_back(1), Some((3, &40)));
    // nth_back(0) returns (2,30)
    assert_eq!(enumerated.nth_back(0), Some((2, &30)));
}

#[test]
fn enumerate_fold() {
    let sum_of_indices = VecLender::new(vec![10, 20, 30])
        .enumerate()
        .fold(0, |acc, (i, _)| acc + i);
    assert_eq!(sum_of_indices, 3); // 0 + 1 + 2
}

#[test]
fn enumerate_rfold() {
    let mut order = Vec::new();
    VecLender::new(vec![10, 20, 30])
        .enumerate()
        .rfold((), |(), (i, v)| {
            order.push((i, *v));
        });
    // Should be reversed: (2,30), (1,20), (0,10)
    assert_eq!(order, vec![(2, 30), (1, 20), (0, 10)]);
}

#[test]
fn enumerate_exact_size() {
    use lender::ExactSizeLender;

    let mut enumerated = VecLender::new(vec![1, 2, 3]).enumerate();
    assert_eq!(enumerated.len(), 3);

    enumerated.next();
    assert_eq!(enumerated.len(), 2);

    enumerated.next();
    enumerated.next();
    assert_eq!(enumerated.len(), 0);
}

#[test]
fn enumerate_into_inner() {
    let enumerate = VecLender::new(vec![10, 20, 30]).enumerate();
    let lender = enumerate.into_inner();
    assert_eq!(lender.count(), 3);
}

#[test]
fn enumerate_try_fold_additional() {
    let result: Option<usize> = VecLender::new(vec![10, 20, 30])
        .enumerate()
        .try_fold(0, |acc, (i, _)| Some(acc + i));
    // Indices: 0, 1, 2 -> sum = 3
    assert_eq!(result, Some(3));
}

#[test]
fn enumerate_try_rfold_additional() {
    let result: Option<usize> = VecLender::new(vec![10, 20, 30])
        .enumerate()
        .try_rfold(0, |acc, (i, _)| Some(acc + i));
    assert_eq!(result, Some(3));
}

#[test]
fn enumerate_rfold_additional() {
    let mut indices = Vec::new();
    VecLender::new(vec![10, 20, 30])
        .enumerate()
        .rfold((), |(), (i, _v)| {
            indices.push(i);
        });
    // Should process in reverse: (2,30), (1,20), (0,10)
    assert_eq!(indices, vec![2, 1, 0]);
}

// ============================================================================
// Inspect adapter tests
// ============================================================================

#[test]
fn inspect_basic() {
    let mut inspected_values = Vec::new();
    let mut lender = VecLender::new(vec![1, 2, 3]).inspect(|x| inspected_values.push(**x));

    assert_eq!(lender.next(), Some(&1));
    assert_eq!(lender.next(), Some(&2));
    assert_eq!(lender.next(), Some(&3));

    assert_eq!(inspected_values, vec![1, 2, 3]);
}

#[test]
fn inspect_fold() {
    let mut inspected = Vec::new();
    let sum = VecLender::new(vec![1, 2, 3])
        .inspect(|x| inspected.push(**x))
        .fold(0, |acc, x| acc + *x);

    assert_eq!(sum, 6);
    assert_eq!(inspected, vec![1, 2, 3]);
}

#[test]
fn inspect_double_ended() {
    let mut inspected = Vec::new();
    let mut lender = VecLender::new(vec![1, 2, 3]).inspect(|x| inspected.push(**x));

    assert_eq!(lender.next_back(), Some(&3));
    assert_eq!(lender.next(), Some(&1));
    assert_eq!(inspected, vec![3, 1]);
}

#[test]
fn inspect_double_ended_fold() {
    let mut inspected = Vec::new();
    let values: Vec<i32> = VecLender::new(vec![1, 2, 3])
        .inspect(|x| inspected.push(**x))
        .rfold(Vec::new(), |mut acc, x| {
            acc.push(*x);
            acc
        });
    assert_eq!(values, vec![3, 2, 1]);
    assert_eq!(inspected, vec![3, 2, 1]);
}

#[test]
fn inspect_into_inner() {
    let inspect = VecLender::new(vec![1, 2, 3]).inspect(|_| {});
    let lender = inspect.into_inner();
    assert_eq!(lender.count(), 3);
}

#[test]
fn inspect_into_parts() {
    let inspect = VecLender::new(vec![1, 2, 3]).inspect(|_| {});
    let (lender, _f) = inspect.into_parts();
    assert_eq!(lender.count(), 3);
}

#[test]
fn inspect_try_fold_additional() {
    let mut inspected = Vec::new();
    let result: Option<i32> = VecLender::new(vec![1, 2, 3])
        .inspect(|x| inspected.push(**x))
        .try_fold(0, |acc, x| Some(acc + *x));
    assert_eq!(result, Some(6));
    assert_eq!(inspected, vec![1, 2, 3]);
}

#[test]
fn inspect_try_rfold_additional() {
    use lender::DoubleEndedLender;

    let mut inspected = Vec::new();
    let result: Option<i32> = VecLender::new(vec![1, 2, 3])
        .inspect(|x| inspected.push(**x))
        .try_rfold(0, |acc, x| Some(acc + *x));
    assert_eq!(result, Some(6));
    assert_eq!(inspected, vec![3, 2, 1]); // Reverse order
}

// ============================================================================
// ============================================================================
// Mutate adapter tests
// ============================================================================

#[test]
fn mutate_basic() {
    let mut data = [1, 2, 3];
    let mut lender = data.iter_mut().into_lender().mutate(|x| **x *= 10);

    assert_eq!(lender.next().map(|x| *x), Some(10));
    assert_eq!(lender.next().map(|x| *x), Some(20));
    assert_eq!(lender.next().map(|x| *x), Some(30));
}

#[test]
fn mutate_fold() {
    let mut data = [1, 2, 3];
    let sum = data
        .iter_mut()
        .into_lender()
        .mutate(|x| **x *= 10)
        .fold(0, |acc, x| acc + *x);
    assert_eq!(sum, 60);
}

#[test]
fn mutate_double_ended() {
    let mut data = [1, 2, 3];
    let mut mutated = data.iter_mut().into_lender().mutate(|x| **x += 100);

    assert_eq!(mutated.next_back().map(|x| *x), Some(103));
    assert_eq!(mutated.next().map(|x| *x), Some(101));
    assert_eq!(mutated.next_back().map(|x| *x), Some(102));
}

#[test]
fn mutate_size_hint_additional() {
    let data = [1, 2, 3];
    let lender = data.iter().into_lender().mutate(|_| {});
    assert_eq!(lender.size_hint(), (3, Some(3)));
}

#[test]
fn mutate_try_fold_additional() {
    let mut data = [1, 2, 3];
    let result: Option<i32> = data
        .iter_mut()
        .into_lender()
        .mutate(|x| **x *= 2)
        .try_fold(0, |acc, x| Some(acc + *x));
    assert_eq!(result, Some(12));
}

#[test]
fn mutate_into_inner_additional() {
    let data = [1, 2, 3];
    let mutate = data.iter().into_lender().mutate(|_| {});
    let lender = mutate.into_inner();
    assert_eq!(lender.count(), 3);
}

#[test]
fn mutate_into_parts_additional() {
    let data = [1, 2, 3];
    let mutate = data.iter().into_lender().mutate(|_| {});
    let (lender, _f) = mutate.into_parts();
    assert_eq!(lender.count(), 3);
}

// ============================================================================
// Map adapter tests
// ============================================================================

#[test]
fn map_basic() {
    let mut mapped =
        VecLender::new(vec![1, 2, 3]).map(covar_mut!(for<'lend> |x: &'lend i32| -> i32 { *x * 2 }));

    assert_eq!(mapped.next(), Some(2));
    assert_eq!(mapped.next(), Some(4));
    assert_eq!(mapped.next(), Some(6));
    assert_eq!(mapped.next(), None);
}

#[test]
fn map_fold() {
    let sum = VecLender::new(vec![1, 2, 3])
        .map(covar_mut!(for<'lend> |x: &'lend i32| -> i32 { *x * 10 }))
        .fold(0, |acc, x| acc + x);
    assert_eq!(sum, 60);
}

#[test]
fn map_double_ended() {
    let mut mapped = VecLender::new(vec![1, 2, 3])
        .map(covar_mut!(for<'lend> |x: &'lend i32| -> i32 { *x * 10 }));

    assert_eq!(mapped.next_back(), Some(30));
    assert_eq!(mapped.next(), Some(10));
    assert_eq!(mapped.next_back(), Some(20));
    assert_eq!(mapped.next(), None);
}

#[test]
fn map_fold_additional() {
    let sum = VecLender::new(vec![1, 2, 3])
        .map(covar_mut!(for<'lend> |x: &'lend i32| -> i32 { *x * 2 }))
        .fold(0, |acc, x| acc + x);
    assert_eq!(sum, 12); // 2 + 4 + 6
}

#[test]
fn map_try_fold_additional() {
    let result: Option<i32> = VecLender::new(vec![1, 2, 3])
        .map(covar_mut!(for<'lend> |x: &'lend i32| -> i32 { *x * 2 }))
        .try_fold(0, |acc, x| Some(acc + x));
    assert_eq!(result, Some(12));
}

#[test]
fn map_rfold_additional() {
    let values: Vec<i32> = VecLender::new(vec![1, 2, 3])
        .map(covar_mut!(for<'lend> |x: &'lend i32| -> i32 { *x * 2 }))
        .rfold(Vec::new(), |mut acc, x| {
            acc.push(x);
            acc
        });
    assert_eq!(values, vec![6, 4, 2]);
}

#[test]
fn map_into_inner() {
    let map =
        VecLender::new(vec![1, 2, 3]).map(covar_mut!(for<'lend> |x: &'lend i32| -> i32 { *x * 2 }));
    let lender = map.into_inner();
    assert_eq!(lender.count(), 3);
}

#[test]
fn map_into_parts_additional() {
    let map =
        VecLender::new(vec![1, 2, 3]).map(covar_mut!(for<'lend> |x: &'lend i32| -> i32 { *x * 2 }));
    let (lender, _f) = map.into_parts();
    assert_eq!(lender.count(), 3);
}

#[test]
fn map_try_rfold_additional() {
    let result: Option<i32> = VecLender::new(vec![1, 2, 3])
        .map(covar_mut!(for<'lend> |x: &'lend i32| -> i32 { *x * 2 }))
        .try_rfold(0, |acc, x| Some(acc + x));
    assert_eq!(result, Some(12)); // 6 + 4 + 2 = 12
}

// ============================================================================
// Scan adapter tests
// Semantics: like fold but yields intermediate states
// Note: scan requires covar_mut! macro for the closure
// ============================================================================

#[test]
fn scan_basic() {
    let mut scanned = VecLender::new(vec![1, 2, 3]).scan(
        0,
        covar_mut!(for<'all> |args: (&'all mut i32, &i32)| -> Option<i32> {
            *args.0 += *args.1;
            Some(*args.0)
        }),
    );

    // Running sum: 1, 3, 6
    assert_eq!(scanned.next(), Some(1));
    assert_eq!(scanned.next(), Some(3));
    assert_eq!(scanned.next(), Some(6));
    assert_eq!(scanned.next(), None);
}

#[test]
fn scan_early_termination() {
    // scan can terminate early by returning None
    let mut scanned = VecLender::new(vec![1, 2, 3, 4, 5]).scan(
        0,
        covar_mut!(for<'all> |args: (&'all mut i32, &i32)| -> Option<i32> {
            *args.0 += *args.1;
            if *args.0 > 5 { None } else { Some(*args.0) }
        }),
    );

    assert_eq!(scanned.next(), Some(1)); // state = 1
    assert_eq!(scanned.next(), Some(3)); // state = 3
    // x=3: state=6, 6 > 5, return None
    assert_eq!(scanned.next(), None);
}

#[test]
fn scan_into_inner() {
    let scan = VecLender::new(vec![1, 2, 3]).scan(
        0,
        covar_mut!(for<'all> |args: (&'all mut i32, &i32)| -> Option<i32> {
            Some(*args.0 + *args.1)
        }),
    );
    let lender = scan.into_inner();
    assert_eq!(lender.count(), 3);
}

#[test]
fn scan_into_parts() {
    let scan = VecLender::new(vec![1, 2, 3]).scan(
        10,
        covar_mut!(for<'all> |args: (&'all mut i32, &i32)| -> Option<i32> {
            Some(*args.0 + *args.1)
        }),
    );
    let (lender, state, _f) = scan.into_parts();
    assert_eq!(lender.count(), 3);
    assert_eq!(state, 10); // Initial state
}

#[test]
fn scan_size_hint_additional() {
    let scan = VecLender::new(vec![1, 2, 3, 4, 5]).scan(
        0,
        covar_mut!(for<'all> |args: (&'all mut i32, &i32)| -> Option<i32> { Some(*args.1) }),
    );
    // Scan can terminate early, so lower bound is 0
    let (lower, upper) = scan.size_hint();
    assert_eq!(lower, 0);
    assert_eq!(upper, Some(5));
}

// ============================================================================
// MapWhile adapter tests
// Semantics: like map but stops when function returns None
// Note: map_while requires covar_mut! macro for the closure
// ============================================================================

#[test]
fn map_while_basic() {
    let mut mw = VecLender::new(vec![1, 2, 3, 4, 5]).map_while(covar_mut!(
        for<'all> |x: &i32| -> Option<i32> { if *x < 4 { Some(*x * 10) } else { None } }
    ));

    assert_eq!(mw.next(), Some(10));
    assert_eq!(mw.next(), Some(20));
    assert_eq!(mw.next(), Some(30));
    assert_eq!(mw.next(), None);
}

#[test]
fn map_while_all_mapped() {
    let mut mw =
        VecLender::new(vec![1, 2, 3]).map_while(covar_mut!(for<'all> |x: &i32| -> Option<i32> {
            Some(*x * 2)
        }));

    assert_eq!(mw.next(), Some(2));
    assert_eq!(mw.next(), Some(4));
    assert_eq!(mw.next(), Some(6));
    assert_eq!(mw.next(), None);
}

#[test]
fn map_while_immediate_none() {
    let mut mw =
        VecLender::new(vec![5, 4, 3]).map_while(covar_mut!(for<'all> |x: &i32| -> Option<i32> {
            if *x < 4 { Some(*x) } else { None }
        }));
    assert_eq!(mw.next(), None);
}

#[test]
fn map_while_into_inner() {
    let map_while =
        VecLender::new(vec![1, 2, 3]).map_while(covar_mut!(for<'all> |x: &i32| -> Option<i32> {
            Some(*x * 2)
        }));
    let lender = map_while.into_inner();
    assert_eq!(lender.count(), 3);
}

#[test]
fn map_while_into_parts() {
    let map_while =
        VecLender::new(vec![1, 2, 3]).map_while(covar_mut!(for<'all> |x: &i32| -> Option<i32> {
            Some(*x * 2)
        }));
    let (lender, _predicate) = map_while.into_parts();
    assert_eq!(lender.count(), 3);
}

#[test]
fn map_while_size_hint_additional() {
    let map_while = VecLender::new(vec![1, 2, 3, 4, 5])
        .map_while(covar_mut!(for<'all> |x: &i32| -> Option<i32> { Some(*x) }));
    // MapWhile can terminate early, so lower bound is 0
    let (lower, upper) = map_while.size_hint();
    assert_eq!(lower, 0);
    assert_eq!(upper, Some(5));
}

#[test]
fn map_while_all_some() {
    // When all return Some, all elements are yielded
    let values: Vec<i32> = VecLender::new(vec![1, 2, 3])
        .map_while(covar_mut!(for<'all> |x: &i32| -> Option<i32> {
            Some(*x * 10)
        }))
        .fold(Vec::new(), |mut acc, x| {
            acc.push(x);
            acc
        });
    assert_eq!(values, vec![10, 20, 30]);
}

// ============================================================================
// Cloned adapter tests
// ============================================================================

#[test]
fn cloned_basic() {
    let lender = VecLender::new(vec![1, 2, 3]);
    let result: Vec<i32> = lender.cloned().collect();
    assert_eq!(result, vec![1, 2, 3]);
}

#[test]
fn cloned_empty() {
    let lender = VecLender::new(vec![]);
    let result: Vec<i32> = lender.cloned().collect();
    assert!(result.is_empty());
}

#[test]
fn cloned_double_ended() {
    let lender = VecLender::new(vec![1, 2, 3]);
    let mut iter = lender.cloned();
    assert_eq!(iter.next_back(), Some(3));
    assert_eq!(iter.next(), Some(1));
    assert_eq!(iter.next_back(), Some(2));
    assert_eq!(iter.next(), None);
}

#[test]
fn cloned_exact_size() {
    let lender = VecLender::new(vec![1, 2, 3]);
    let iter = lender.cloned();
    assert_eq!(iter.len(), 3);
    assert_eq!(iter.size_hint(), (3, Some(3)));
}

#[test]
fn cloned_into_inner() {
    let lender = VecLender::new(vec![1, 2, 3]);
    let cloned = lender.cloned();
    let mut inner = cloned.into_inner();
    assert_eq!(inner.next(), Some(&1));
}

#[test]
fn cloned_fold() {
    let lender = VecLender::new(vec![1, 2, 3]);
    let sum = lender.cloned().fold(0, |acc, x| acc + x);
    assert_eq!(sum, 6);
}

#[test]
fn cloned_fold_empty() {
    let lender = VecLender::new(vec![]);
    let sum: i32 = lender.cloned().fold(0, |acc, x| acc + x);
    assert_eq!(sum, 0);
}

#[test]
fn cloned_rfold() {
    let lender = VecLender::new(vec![1, 2, 3]);
    let result = lender.cloned().rfold(Vec::new(), |mut acc, x| {
        acc.push(x);
        acc
    });
    assert_eq!(result, vec![3, 2, 1]);
}

#[test]
fn cloned_rfold_empty() {
    let lender = VecLender::new(vec![]);
    let result: Vec<i32> = lender.cloned().rfold(Vec::new(), |mut acc, x| {
        acc.push(x);
        acc
    });
    assert!(result.is_empty());
}

#[test]
fn cloned_count() {
    let lender = VecLender::new(vec![1, 2, 3, 4, 5]);
    assert_eq!(lender.cloned().count(), 5);
}

#[test]
fn cloned_count_empty() {
    let lender = VecLender::new(vec![]);
    assert_eq!(lender.cloned().count(), 0);
}

#[test]
fn cloned_nth() {
    let lender = VecLender::new(vec![10, 20, 30, 40, 50]);
    let mut iter = lender.cloned();
    assert_eq!(iter.nth(0), Some(10));
    assert_eq!(iter.nth(1), Some(30));
    assert_eq!(iter.nth(2), None);
}

#[test]
fn cloned_nth_out_of_bounds() {
    let lender = VecLender::new(vec![1, 2, 3]);
    let mut iter = lender.cloned();
    assert_eq!(iter.nth(10), None);
}

// ============================================================================
// Copied adapter tests
// ============================================================================

#[test]
fn copied_basic() {
    let lender = VecLender::new(vec![10, 20, 30]);
    let result: Vec<i32> = lender.copied().collect();
    assert_eq!(result, vec![10, 20, 30]);
}

#[test]
fn copied_empty() {
    let lender = VecLender::new(vec![]);
    let result: Vec<i32> = lender.copied().collect();
    assert!(result.is_empty());
}

#[test]
fn copied_double_ended() {
    let lender = VecLender::new(vec![10, 20, 30]);
    let mut iter = lender.copied();
    assert_eq!(iter.next(), Some(10));
    assert_eq!(iter.next_back(), Some(30));
    assert_eq!(iter.next(), Some(20));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_back(), None);
}

#[test]
fn copied_exact_size() {
    let lender = VecLender::new(vec![10, 20, 30]);
    let iter = lender.copied();
    assert_eq!(iter.len(), 3);
    assert_eq!(iter.size_hint(), (3, Some(3)));
}

#[test]
fn copied_into_inner() {
    let lender = VecLender::new(vec![10, 20]);
    let copied = lender.copied();
    let mut inner = copied.into_inner();
    assert_eq!(inner.next(), Some(&10));
}

#[test]
fn copied_fold() {
    let lender = VecLender::new(vec![10, 20, 30]);
    let sum = lender.copied().fold(0, |acc, x| acc + x);
    assert_eq!(sum, 60);
}

#[test]
fn copied_fold_empty() {
    let lender = VecLender::new(vec![]);
    let sum = lender.copied().fold(0, |acc, x| acc + x);
    assert_eq!(sum, 0);
}

#[test]
fn copied_rfold() {
    let lender = VecLender::new(vec![10, 20, 30]);
    let result = lender.copied().rfold(Vec::new(), |mut acc, x| {
        acc.push(x);
        acc
    });
    assert_eq!(result, vec![30, 20, 10]);
}

#[test]
fn copied_rfold_empty() {
    let lender = VecLender::new(vec![]);
    let result: Vec<i32> = lender.copied().rfold(Vec::new(), |mut acc, x| {
        acc.push(x);
        acc
    });
    assert!(result.is_empty());
}

#[test]
fn copied_count() {
    let lender = VecLender::new(vec![10, 20, 30, 40]);
    assert_eq!(lender.copied().count(), 4);
}

#[test]
fn copied_count_empty() {
    let lender = VecLender::new(vec![]);
    assert_eq!(lender.copied().count(), 0);
}

#[test]
fn copied_nth() {
    let lender = VecLender::new(vec![100, 200, 300, 400, 500]);
    let mut iter = lender.copied();
    assert_eq!(iter.nth(0), Some(100));
    assert_eq!(iter.nth(2), Some(400));
    assert_eq!(iter.nth(1), None);
}

#[test]
fn copied_nth_out_of_bounds() {
    let lender = VecLender::new(vec![1, 2]);
    let mut iter = lender.copied();
    assert_eq!(iter.nth(5), None);
}

// ============================================================================
// Owned adapter tests
// ============================================================================

#[test]
fn owned_basic() {
    // owned() requires for<'all> Lend<'all, L>: ToOwned<Owned = T> where T is
    // lifetime-independent. Use into_lender() from an iterator yielding values.
    let lender = [1, 2, 3].into_iter().into_lender();
    let result: Vec<i32> = lender.owned().collect();
    assert_eq!(result, vec![1, 2, 3]);
}

#[test]
fn owned_empty() {
    let lender = std::iter::empty::<i32>().into_lender();
    let result: Vec<i32> = lender.owned().collect();
    assert!(result.is_empty());
}

#[test]
fn owned_into_inner() {
    let lender = [1, 2].into_iter().into_lender();
    let owned = lender.owned();
    let mut inner = owned.into_inner();
    assert_eq!(inner.next(), Some(1));
}

#[test]
fn owned_fold() {
    let lender = [1, 2, 3].into_iter().into_lender();
    let sum = lender.owned().fold(0, |acc, x| acc + x);
    assert_eq!(sum, 6);
}

#[test]
fn owned_fold_empty() {
    let lender = std::iter::empty::<i32>().into_lender();
    let sum = lender.owned().fold(0, |acc, x| acc + x);
    assert_eq!(sum, 0);
}

#[test]
fn owned_rfold() {
    let lender = [1, 2, 3].into_iter().into_lender();
    let result = lender.owned().rfold(Vec::new(), |mut acc, x| {
        acc.push(x);
        acc
    });
    assert_eq!(result, vec![3, 2, 1]);
}

#[test]
fn owned_rfold_empty() {
    let lender = std::iter::empty::<i32>().into_lender();
    let result = lender.owned().rfold(Vec::<i32>::new(), |mut acc, x| {
        acc.push(x);
        acc
    });
    assert!(result.is_empty());
}

#[test]
fn owned_count() {
    let lender = [1, 2, 3, 4, 5, 6].into_iter().into_lender();
    assert_eq!(lender.owned().count(), 6);
}

#[test]
fn owned_count_empty() {
    let lender = std::iter::empty::<i32>().into_lender();
    assert_eq!(lender.owned().count(), 0);
}

#[test]
fn owned_nth() {
    let lender = [10, 20, 30, 40, 50].into_iter().into_lender();
    let mut iter = lender.owned();
    assert_eq!(iter.nth(1), Some(20));
    assert_eq!(iter.nth(1), Some(40));
    assert_eq!(iter.nth(0), Some(50));
    assert_eq!(iter.nth(0), None);
}

#[test]
fn owned_nth_out_of_bounds() {
    let lender = [1, 2, 3].into_iter().into_lender();
    let mut iter = lender.owned();
    assert_eq!(iter.nth(100), None);
}
