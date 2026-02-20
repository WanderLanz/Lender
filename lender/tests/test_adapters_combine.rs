//! Tests for combination adapters: Zip, Intersperse, Flatten, FlatMap

#![allow(clippy::unnecessary_fold)]

mod common;
use ::lender::FromIter;
use ::lender::prelude::*;
use common::*;

// ============================================================================
// Zip adapter tests
// ============================================================================

#[test]
fn test_zip_basic() {
    let mut zipped = VecLender::new(vec![1, 2, 3]).zip(VecLender::new(vec![10, 20, 30]));

    assert_eq!(zipped.next(), Some((&1, &10)));
    assert_eq!(zipped.next(), Some((&2, &20)));
    assert_eq!(zipped.next(), Some((&3, &30)));
    assert_eq!(zipped.next(), None);
}

#[test]
fn test_zip_different_lengths() {
    // Stops at shorter lender
    let mut zipped = VecLender::new(vec![1, 2]).zip(VecLender::new(vec![10, 20, 30]));

    assert_eq!(zipped.next(), Some((&1, &10)));
    assert_eq!(zipped.next(), Some((&2, &20)));
    assert_eq!(zipped.next(), None);
}

#[test]
fn test_zip_double_ended() {
    let mut zipped = VecLender::new(vec![1, 2, 3]).zip(VecLender::new(vec![10, 20, 30]));

    assert_eq!(zipped.next_back(), Some((&3, &30)));
    assert_eq!(zipped.next_back(), Some((&2, &20)));
    assert_eq!(zipped.next_back(), Some((&1, &10)));
    assert_eq!(zipped.next_back(), None);
}

#[test]
fn test_zip_fold() {
    let sum = VecLender::new(vec![1, 2, 3])
        .zip(VecLender::new(vec![10, 20, 30]))
        .fold(0, |acc, (a, b)| acc + *a + *b);
    // (1+10) + (2+20) + (3+30) = 66
    assert_eq!(sum, 66);
}

#[test]
fn test_zip_nth() {
    let mut zipped = VecLender::new(vec![1, 2, 3, 4]).zip(VecLender::new(vec![10, 20, 30, 40]));
    assert_eq!(zipped.nth(2), Some((&3, &30)));
    assert_eq!(zipped.next(), Some((&4, &40)));
}

#[test]
fn test_zip_size_hint_additional() {
    let zipped = VecLender::new(vec![1, 2, 3]).zip(VecLender::new(vec![4, 5, 6, 7]));
    // Zip takes the minimum
    assert_eq!(zipped.size_hint(), (3, Some(3)));
}

#[test]
fn test_zip_try_fold_additional() {
    let result: Option<i32> = VecLender::new(vec![1, 2, 3])
        .zip(VecLender::new(vec![4, 5, 6]))
        .try_fold(0, |acc, (a, b)| Some(acc + *a + *b));
    assert_eq!(result, Some(21)); // (1+4) + (2+5) + (3+6) = 5 + 7 + 9 = 21
}

#[test]
fn test_zip_try_rfold_additional() {
    let result: Option<i32> = VecLender::new(vec![1, 2, 3])
        .zip(VecLender::new(vec![4, 5, 6]))
        .try_rfold(0, |acc, (a, b)| Some(acc + *a + *b));
    assert_eq!(result, Some(21));
}

#[test]
fn test_zip_into_inner() {
    let zip = VecLender::new(vec![1, 2, 3]).zip(VecLender::new(vec![4, 5, 6]));
    let (a, b) = zip.into_inner();
    assert_eq!(a.count(), 3);
    assert_eq!(b.count(), 3);
}

#[test]
fn test_zip_rfold() {
    let mut values = Vec::new();
    VecLender::new(vec![1, 2, 3])
        .zip(VecLender::new(vec![10, 20, 30]))
        .rfold((), |(), (a, b)| {
            values.push((*a, *b));
        });
    assert_eq!(values, vec![(3, 30), (2, 20), (1, 10)]);
}

// ============================================================================
// Intersperse adapter tests (Lender)
// Semantics: insert separator between elements
// ============================================================================

#[test]
fn test_intersperse_basic() {
    let mut interspersed = VecLender::new(vec![1, 2, 3]).intersperse(&0);

    assert_eq!(interspersed.next(), Some(&1));
    assert_eq!(interspersed.next(), Some(&0)); // separator
    assert_eq!(interspersed.next(), Some(&2));
    assert_eq!(interspersed.next(), Some(&0)); // separator
    assert_eq!(interspersed.next(), Some(&3));
    assert_eq!(interspersed.next(), None);
}

#[test]
fn test_intersperse_single_element() {
    let mut interspersed = VecLender::new(vec![42]).intersperse(&0);

    assert_eq!(interspersed.next(), Some(&42));
    assert_eq!(interspersed.next(), None);
}

#[test]
fn test_intersperse_empty() {
    let mut interspersed = VecLender::new(vec![]).intersperse(&0);
    assert_eq!(interspersed.next(), None);
}

#[test]
fn test_intersperse_fold() {
    let sum = VecLender::new(vec![1, 2, 3])
        .intersperse(&10)
        .fold(0, |acc, x| acc + *x);
    // 1 + 10 + 2 + 10 + 3 = 26
    assert_eq!(sum, 26);
}

#[test]
fn test_intersperse_with_basic() {
    const SEP1: i32 = 10;
    const SEP2: i32 = 20;
    let mut counter = 0;
    let mut interspersed = VecLender::new(vec![1, 2, 3]).intersperse_with(|| {
        counter += 1;
        if counter == 1 { &SEP1 } else { &SEP2 }
    });

    assert_eq!(interspersed.next(), Some(&1));
    assert_eq!(interspersed.next(), Some(&10)); // counter = 1
    assert_eq!(interspersed.next(), Some(&2));
    assert_eq!(interspersed.next(), Some(&20)); // counter = 2
    assert_eq!(interspersed.next(), Some(&3));
    assert_eq!(interspersed.next(), None);
}

#[test]
fn test_intersperse_into_inner() {
    let intersperse = VecLender::new(vec![1, 2, 3]).intersperse(&0);
    let lender = intersperse.into_inner();
    assert_eq!(lender.count(), 3);
}

#[test]
fn test_intersperse_try_fold_additional() {
    let result: Option<i32> = VecLender::new(vec![1, 2, 3])
        .intersperse(&10)
        .try_fold(0, |acc, x| Some(acc + *x));
    // 1 + 10 + 2 + 10 + 3 = 26
    assert_eq!(result, Some(26));
}

// Intersperse with separator clone (covers unsafe transmute in next)
#[test]
fn test_intersperse_separator_coverage() {
    let mut intersperse = VecLender::new(vec![1, 2, 3]).intersperse(&0);
    // Consume all to exercise the separator clone path
    let mut results = Vec::new();
    while let Some(x) = intersperse.next() {
        results.push(*x);
    }
    assert_eq!(results, vec![1, 0, 2, 0, 3]);
}

// IntersperseWith (covers unsafe transmute in next)
#[test]
fn test_intersperse_with_coverage() {
    let mut intersperse = VecLender::new(vec![1, 2, 3]).intersperse_with(|| &0);
    let mut results = Vec::new();
    while let Some(x) = intersperse.next() {
        results.push(*x);
    }
    assert_eq!(results, vec![1, 0, 2, 0, 3]);
}

#[test]
fn test_intersperse_try_fold_early_exit() {
    // try_fold that stops early via None
    let result: Option<i32> =
        VecLender::new(vec![1, 2, 3])
            .intersperse(&10)
            .try_fold(
                0,
                |acc, x| {
                    if acc + *x > 15 { None } else { Some(acc + *x) }
                },
            );
    // 1 + 10 + 2 = 13, then next is 10 → 23 > 15, so None
    assert_eq!(result, None);
}

#[test]
fn test_intersperse_try_fold_empty() {
    let result: Option<i32> = VecLender::new(vec![])
        .intersperse(&10)
        .try_fold(0, |acc, x| Some(acc + *x));
    assert_eq!(result, Some(0));
}

#[test]
fn test_intersperse_try_fold_single() {
    let result: Option<i32> = VecLender::new(vec![42])
        .intersperse(&10)
        .try_fold(0, |acc, x| Some(acc + *x));
    assert_eq!(result, Some(42));
}

#[test]
fn test_intersperse_fold_empty() {
    let sum = VecLender::new(vec![])
        .intersperse(&10)
        .fold(0, |acc, x: &i32| acc + *x);
    assert_eq!(sum, 0);
}

#[test]
fn test_intersperse_fold_single() {
    let sum = VecLender::new(vec![42])
        .intersperse(&10)
        .fold(0, |acc, x| acc + *x);
    assert_eq!(sum, 42);
}

#[test]
fn test_intersperse_with_try_fold() {
    let result: Option<i32> = VecLender::new(vec![1, 2, 3])
        .intersperse_with(|| &10)
        .try_fold(0, |acc, x| Some(acc + *x));
    // 1 + 10 + 2 + 10 + 3 = 26
    assert_eq!(result, Some(26));
}

#[test]
fn test_intersperse_with_try_fold_early_exit() {
    let result: Option<i32> = VecLender::new(vec![1, 2, 3])
        .intersperse_with(|| &10)
        .try_fold(
            0,
            |acc, x| {
                if acc + *x > 15 { None } else { Some(acc + *x) }
            },
        );
    assert_eq!(result, None);
}

#[test]
fn test_intersperse_with_try_fold_empty() {
    let result: Option<i32> = VecLender::new(vec![])
        .intersperse_with(|| &10)
        .try_fold(0, |acc, x| Some(acc + *x));
    assert_eq!(result, Some(0));
}

#[test]
fn test_intersperse_with_try_fold_single() {
    let result: Option<i32> = VecLender::new(vec![42])
        .intersperse_with(|| &10)
        .try_fold(0, |acc, x| Some(acc + *x));
    assert_eq!(result, Some(42));
}

#[test]
fn test_intersperse_with_fold() {
    let sum = VecLender::new(vec![1, 2, 3])
        .intersperse_with(|| &10)
        .fold(0, |acc, x| acc + *x);
    // 1 + 10 + 2 + 10 + 3 = 26
    assert_eq!(sum, 26);
}

#[test]
fn test_intersperse_with_fold_empty() {
    let sum = VecLender::new(vec![])
        .intersperse_with(|| &10)
        .fold(0, |acc, x: &i32| acc + *x);
    assert_eq!(sum, 0);
}

#[test]
fn test_intersperse_with_fold_single() {
    let sum = VecLender::new(vec![42])
        .intersperse_with(|| &10)
        .fold(0, |acc, x| acc + *x);
    assert_eq!(sum, 42);
}

#[test]
fn test_intersperse_for_each() {
    let mut items = Vec::new();
    VecLender::new(vec![1, 2, 3])
        .intersperse(&0)
        .for_each(|x| items.push(*x));
    assert_eq!(items, vec![1, 0, 2, 0, 3]);
}

#[test]
fn test_intersperse_with_for_each() {
    let mut items = Vec::new();
    VecLender::new(vec![1, 2, 3])
        .intersperse_with(|| &0)
        .for_each(|x| items.push(*x));
    assert_eq!(items, vec![1, 0, 2, 0, 3]);
}

#[test]
fn test_intersperse_count() {
    let count = VecLender::new(vec![1, 2, 3]).intersperse(&0).count();
    assert_eq!(count, 5); // 3 elements + 2 separators
}

#[test]
fn test_intersperse_with_count() {
    let count = VecLender::new(vec![1, 2, 3])
        .intersperse_with(|| &0)
        .count();
    assert_eq!(count, 5);
}

// ============================================================================
// Flatten adapter tests
// ============================================================================

#[test]
fn test_flatten_basic() {
    let mut flattened = VecOfVecLender::new(vec![vec![1, 2], vec![3, 4], vec![5]]).flatten();

    assert_eq!(flattened.next(), Some(&1));
    assert_eq!(flattened.next(), Some(&2));
    assert_eq!(flattened.next(), Some(&3));
    assert_eq!(flattened.next(), Some(&4));
    assert_eq!(flattened.next(), Some(&5));
    assert_eq!(flattened.next(), None);
}

#[test]
fn test_flatten_empty_inner() {
    let mut flattened = VecOfVecLender::new(vec![vec![1], vec![], vec![2, 3]]).flatten();

    assert_eq!(flattened.next(), Some(&1));
    assert_eq!(flattened.next(), Some(&2));
    assert_eq!(flattened.next(), Some(&3));
    assert_eq!(flattened.next(), None);
}

#[test]
fn test_flatten_empty_outer() {
    let mut flattened = VecOfVecLender::new(vec![]).flatten();
    assert_eq!(flattened.next(), None);
}

// ============================================================================
// FlatMap adapter tests (infallible)
// ============================================================================

#[test]
fn test_flat_map_basic() {
    // flat_map: for each element n, produce n copies of n
    let mut l =
        [1, 2, 3]
            .into_iter()
            .into_lender()
            .flat_map(covar_mut!(for<'lend> |n: i32| -> FromIter<
                std::ops::Range<i32>,
            > { (0..n).into_lender() }));
    assert_eq!(l.next(), Some(0)); // from n=1: [0]
    assert_eq!(l.next(), Some(0)); // from n=2: [0, 1]
    assert_eq!(l.next(), Some(1));
    assert_eq!(l.next(), Some(0)); // from n=3: [0, 1, 2]
    assert_eq!(l.next(), Some(1));
    assert_eq!(l.next(), Some(2));
    assert_eq!(l.next(), None);
}

#[test]
fn test_flat_map_empty_outer() {
    let mut l = std::iter::empty::<i32>().into_lender().flat_map(covar_mut!(
        for<'lend> |n: i32| -> FromIter<std::ops::Range<i32>> { (0..n).into_lender() }
    ));
    assert_eq!(l.next(), None);
}

#[test]
fn test_flat_map_empty_inner() {
    // All inner lenders are empty
    let mut l =
        [0, 0, 0]
            .into_iter()
            .into_lender()
            .flat_map(covar_mut!(for<'lend> |n: i32| -> FromIter<
                std::ops::Range<i32>,
            > { (0..n).into_lender() }));
    assert_eq!(l.next(), None);
}

#[test]
fn test_flat_map_mixed_empty_nonempty() {
    let mut l =
        [1, 0, 2]
            .into_iter()
            .into_lender()
            .flat_map(covar_mut!(for<'lend> |n: i32| -> FromIter<
                std::ops::Range<i32>,
            > { (0..n).into_lender() }));
    assert_eq!(l.next(), Some(0)); // from n=1
    // n=0 produces empty
    assert_eq!(l.next(), Some(0)); // from n=2
    assert_eq!(l.next(), Some(1));
    assert_eq!(l.next(), None);
}

// ============================================================================
// Flatten fold/try_fold/count tests
// ============================================================================

#[test]
fn test_flatten_fold() {
    let lender = VecOfVecLender::new(vec![vec![1, 2], vec![3], vec![4, 5]]);
    let result = lender.flatten().fold(0, |acc, x| acc + x);
    assert_eq!(result, 15); // 1+2+3+4+5
}

#[test]
fn test_flatten_count() {
    let lender = VecOfVecLender::new(vec![vec![1, 2], vec![], vec![3, 4, 5]]);
    assert_eq!(lender.flatten().count(), 5);
}

#[test]
fn test_flatten_try_fold() {
    let lender = VecOfVecLender::new(vec![vec![1, 2], vec![3, 4], vec![5]]);
    let result: Result<i32, i32> = lender.flatten().try_fold(0, |acc, x| {
        let new = acc + x;
        if new > 6 { Err(new) } else { Ok(new) }
    });
    assert_eq!(result, Err(10)); // 1+2+3+4 = 10 > 6
}

#[test]
fn test_flatten_fold_empty() {
    let lender = VecOfVecLender::new(vec![]);
    let result = lender.flatten().fold(0, |acc, x: &i32| acc + x);
    assert_eq!(result, 0);
}

#[test]
fn test_flatten_count_empty() {
    let lender = VecOfVecLender::new(vec![]);
    assert_eq!(lender.flatten().count(), 0);
}
