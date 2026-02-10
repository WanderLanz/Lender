//! Tests for adapter composition, boundary conditions, and new methods

#![allow(clippy::unnecessary_fold)]

mod common;
use ::lender::prelude::*;
use common::*;

// ============================================================================
// New method tests
// ============================================================================

#[test]
fn zip_nth_back_equal_length() {
    let mut zipped =
        VecLender::new(vec![1, 2, 3, 4, 5]).zip(VecLender::new(vec![10, 20, 30, 40, 50]));
    // nth_back(0) yields the last pair
    assert_eq!(zipped.nth_back(0), Some((&5, &50)));
    // nth_back(1) skips (4,40) and yields (3,30)
    assert_eq!(zipped.nth_back(1), Some((&3, &30)));
    // Only (1,10) and (2,20) remain; nth_back(2) is past the end
    assert_eq!(zipped.nth_back(2), None);
}

#[test]
fn zip_nth_back_unequal_length() {
    let mut zipped = VecLender::new(vec![1, 2, 3, 4, 5]).zip(VecLender::new(vec![10, 20, 30]));
    // Zip length is min(5, 3) = 3, so effective pairs are (1,10),(2,20),(3,30).
    // nth_back(0) should yield (3,30)
    assert_eq!(zipped.nth_back(0), Some((&3, &30)));
    assert_eq!(zipped.nth_back(0), Some((&2, &20)));
    assert_eq!(zipped.nth_back(0), Some((&1, &10)));
    assert_eq!(zipped.nth_back(0), None);
}

#[test]
fn zip_nth_back_empty() {
    let mut zipped = VecLender::new(vec![]).zip(VecLender::new(vec![1, 2]));
    assert_eq!(zipped.nth_back(0), None);
}

#[test]
fn zip_nth_back_first_shorter() {
    // First lender shorter than second — tests the a_sz < b_sz branch
    // in Zip::nth_back where b.advance_back_by() trims the excess.
    let mut zipped = VecLender::new(vec![10, 20, 30]).zip(VecLender::new(vec![1, 2, 3, 4, 5]));
    // Zip length is min(3, 5) = 3, effective pairs: (10,1),(20,2),(30,3).
    // nth_back(0) yields (30, 3)
    assert_eq!(zipped.nth_back(0), Some((&30, &3)));
    assert_eq!(zipped.nth_back(0), Some((&20, &2)));
    assert_eq!(zipped.nth_back(0), Some((&10, &1)));
    assert_eq!(zipped.nth_back(0), None);
}

#[test]
fn step_by_count_basic() {
    let lender = VecLender::new(vec![1, 2, 3, 4, 5, 6, 7]);
    // step=2 yields [1, 3, 5, 7] → count = 4
    assert_eq!(lender.step_by(2).count(), 4);
}

#[test]
fn step_by_count_step_one() {
    let lender = VecLender::new(vec![1, 2, 3]);
    assert_eq!(lender.step_by(1).count(), 3);
}

#[test]
fn step_by_count_empty() {
    let lender = VecLender::new(vec![]);
    assert_eq!(lender.step_by(3).count(), 0);
}

#[test]
fn step_by_count_larger_step() {
    let lender = VecLender::new(vec![1, 2, 3]);
    // step=10 yields only the first element
    assert_eq!(lender.step_by(10).count(), 1);
}

#[test]
fn chunk_count() {
    let mut lender = VecLender::new(vec![1, 2, 3, 4, 5]);
    let chunk = lender.next_chunk(3);
    assert_eq!(chunk.count(), 3);
}

#[test]
fn chunk_count_larger_than_remaining() {
    let mut lender = VecLender::new(vec![1, 2]);
    let chunk = lender.next_chunk(5);
    // Underlying only has 2 elements even though chunk_size is 5
    assert_eq!(chunk.count(), 2);
}

#[test]
fn chunk_count_empty() {
    let mut lender = VecLender::new(vec![]);
    let chunk = lender.next_chunk(3);
    assert_eq!(chunk.count(), 0);
}

#[test]
fn chunk_nth_within_range() {
    let mut lender = VecLender::new(vec![10, 20, 30, 40, 50]);
    let mut chunk = lender.next_chunk(4);
    // nth(2) should skip first 2 elements and return the 3rd (30)
    assert_eq!(chunk.nth(2), Some(&30));
    // Only one element left in chunk (40)
    assert_eq!(chunk.next(), Some(&40));
    assert_eq!(chunk.next(), None);
}

#[test]
fn chunk_nth_past_end() {
    let mut lender = VecLender::new(vec![10, 20, 30]);
    let mut chunk = lender.next_chunk(3);
    // nth(5) is past the chunk boundary
    assert_eq!(chunk.nth(5), None);
    // Chunk is exhausted now
    assert_eq!(chunk.next(), None);
}

#[test]
fn chunk_nth_zero() {
    let mut lender = VecLender::new(vec![10, 20, 30]);
    let mut chunk = lender.next_chunk(3);
    assert_eq!(chunk.nth(0), Some(&10));
    assert_eq!(chunk.nth(0), Some(&20));
    assert_eq!(chunk.nth(0), Some(&30));
    assert_eq!(chunk.nth(0), None);
}

#[test]
fn chunk_try_fold_full() {
    let mut lender = VecLender::new(vec![1, 2, 3, 4, 5]);
    let mut chunk = lender.next_chunk(4);
    let result: Result<i32, ()> = chunk.try_fold(0, |acc, x| Ok(acc + *x));
    assert_eq!(result, Ok(10)); // 1+2+3+4
}

#[test]
fn chunk_try_fold_early_exit() {
    let mut lender = VecLender::new(vec![1, 2, 3, 4, 5]);
    let mut chunk = lender.next_chunk(5);
    // Stop when accumulator exceeds 5
    let result: Result<i32, i32> = chunk.try_fold(0, |acc, x| {
        let new = acc + *x;
        if new > 5 { Err(new) } else { Ok(new) }
    });
    assert_eq!(result, Err(6)); // 1+2+3 = 6 > 5
}

#[test]
fn chunk_fold_full() {
    let mut lender = VecLender::new(vec![1, 2, 3, 4, 5]);
    let chunk = lender.next_chunk(4);
    let result = chunk.fold(0, |acc, x| acc + *x);
    assert_eq!(result, 10); // 1+2+3+4
}

#[test]
fn chunk_fold_partial_underlying() {
    let mut lender = VecLender::new(vec![1, 2]);
    let chunk = lender.next_chunk(5);
    let result = chunk.fold(0, |acc, x| acc + *x);
    assert_eq!(result, 3); // 1+2, underlying runs out before chunk limit
}

// ============================================================================
// Multi-adapter composition tests (infallible)
// ============================================================================

#[test]
fn compose_filter_map_fold() {
    // filter even numbers, map to doubled, then fold sum
    let result = VecLender::new(vec![1, 2, 3, 4, 5, 6])
        .filter(|x| *x % 2 == 0)
        .map(covar_mut!(for<'all> |x: &'all i32| -> i32 { *x * 10 }))
        .fold(0, |acc, x| acc + x);
    // 2*10 + 4*10 + 6*10 = 120
    assert_eq!(result, 120);
}

#[test]
fn compose_skip_take() {
    let mut lender = VecLender::new(vec![1, 2, 3, 4, 5, 6, 7, 8]).skip(2).take(3);
    assert_eq!(lender.next(), Some(&3));
    assert_eq!(lender.next(), Some(&4));
    assert_eq!(lender.next(), Some(&5));
    assert_eq!(lender.next(), None);
}

#[test]
fn compose_chain_filter() {
    let a = VecLender::new(vec![1, 2, 3]);
    let b = VecLender::new(vec![4, 5, 6]);
    let result = a.chain(b).filter(|x| **x > 2).count();
    // 3, 4, 5, 6 pass the filter
    assert_eq!(result, 4);
}

#[test]
fn compose_rev_take() {
    let mut lender = VecLender::new(vec![1, 2, 3, 4, 5]).rev().take(3);
    assert_eq!(lender.next(), Some(&5));
    assert_eq!(lender.next(), Some(&4));
    assert_eq!(lender.next(), Some(&3));
    assert_eq!(lender.next(), None);
}

#[test]
fn compose_flatten_filter_map() {
    let result = VecOfVecLender::new(vec![vec![1, 2, 3], vec![4, 5, 6]])
        .flatten()
        .filter(|x| **x % 2 == 1)
        .map(covar_mut!(for<'all> |x: &'all i32| -> i32 { *x * 100 }))
        .fold(0, |acc, x| acc + x);
    // Odd: 1, 3, 5 → 100 + 300 + 500 = 900
    assert_eq!(result, 900);
}

#[test]
fn compose_enumerate_skip_take() {
    let mut lender = VecLender::new(vec![10, 20, 30, 40, 50])
        .enumerate()
        .skip(1)
        .take(2);
    assert_eq!(lender.next(), Some((1, &20)));
    assert_eq!(lender.next(), Some((2, &30)));
    assert_eq!(lender.next(), None);
}

#[test]
fn compose_filter_inspect_count() {
    let mut seen = Vec::new();
    let count = VecLender::new(vec![1, 2, 3, 4, 5])
        .filter(|x| **x > 2)
        .inspect(|x| seen.push(**x))
        .count();
    assert_eq!(count, 3);
    assert_eq!(seen, vec![3, 4, 5]);
}

#[test]
fn compose_step_by_chain() {
    let a = VecLender::new(vec![1, 2, 3, 4, 5, 6]).step_by(2);
    let b = VecLender::new(vec![10, 20]);
    let mut lender = a.chain(b);
    // step_by(2) yields: 1, 3, 5; then chain: 10, 20
    assert_eq!(lender.next(), Some(&1));
    assert_eq!(lender.next(), Some(&3));
    assert_eq!(lender.next(), Some(&5));
    assert_eq!(lender.next(), Some(&10));
    assert_eq!(lender.next(), Some(&20));
    assert_eq!(lender.next(), None);
}

#[test]
fn compose_zip_map_fold() {
    let a = VecLender::new(vec![1, 2, 3]);
    let b = VecLender::new(vec![10, 20, 30]);
    let result = a
        .zip(b)
        .map(covar_mut!(
            for<'all> |pair: (&'all i32, &'all i32)| -> i32 { *pair.0 + *pair.1 }
        ))
        .fold(0, |acc, x| acc + x);
    // (1+10) + (2+20) + (3+30) = 66
    assert_eq!(result, 66);
}

#[test]
fn compose_take_while_skip_while() {
    let mut lender = VecLender::new(vec![1, 2, 3, 4, 5, 6, 7])
        .skip_while(|x| **x < 3)
        .take_while(|x| **x <= 5);
    assert_eq!(lender.next(), Some(&3));
    assert_eq!(lender.next(), Some(&4));
    assert_eq!(lender.next(), Some(&5));
    assert_eq!(lender.next(), None);
}

// ============================================================================
// Boundary and overflow tests
// ============================================================================

#[test]
fn chain_size_hint_no_overflow() {
    // Chain two lenders with size hints that would overflow if added naively

    let a = std::iter::repeat_n(&1, usize::MAX / 2 + 1).into_lender();
    let b = std::iter::repeat_n(&2, usize::MAX / 2 + 1).into_lender();
    let chained = a.chain(b);
    let (lower, upper) = chained.size_hint();
    // Should saturate rather than overflow
    assert_eq!(lower, usize::MAX);
    assert!(upper.is_none() || upper == Some(usize::MAX));
}

#[test]
fn intersperse_size_hint_no_overflow() {
    // Intersperse doubles size minus 1; test with large size hints
    // that would overflow if not handled with saturating arithmetic

    let lender = std::iter::repeat_n(&1, usize::MAX / 2 + 10).into_lender();
    let interspersed = lender.intersperse(&0);
    let (lower, _upper) = interspersed.size_hint();
    // Should saturate to usize::MAX rather than overflow
    assert_eq!(lower, usize::MAX);
}

#[test]
fn step_by_large_step() {
    // Test step_by with very large step values
    let mut lender = VecLender::new(vec![1, 2, 3, 4, 5]).step_by(1000);
    assert_eq!(lender.next(), Some(&1));
    assert_eq!(lender.next(), None);
}

#[test]
fn step_by_step_equals_len() {
    // Step size equals lender length
    let mut lender = VecLender::new(vec![1, 2, 3]).step_by(3);
    assert_eq!(lender.next(), Some(&1));
    assert_eq!(lender.next(), None);
}

#[test]
fn advance_by_zero() {
    // Advance by 0 should be a no-op
    let mut lender = VecLender::new(vec![1, 2, 3]);
    assert_eq!(lender.advance_by(0), Ok(()));
    assert_eq!(lender.next(), Some(&1));
}

#[test]
fn advance_by_exact_length() {
    // Advance by exact remaining length should succeed
    let mut lender = VecLender::new(vec![1, 2, 3]);
    assert_eq!(lender.advance_by(3), Ok(()));
    assert_eq!(lender.next(), None);
}

#[test]
fn advance_by_more_than_length() {
    use core::num::NonZeroUsize;
    let mut lender = VecLender::new(vec![1, 2]);
    // Trying to advance by 5 when only 2 elements remain
    assert_eq!(lender.advance_by(5), Err(NonZeroUsize::new(3).unwrap()));
    // Lender should be exhausted after failed advance
    assert_eq!(lender.next(), None);
}

#[test]
fn nth_zero() {
    // nth(0) should be equivalent to next()
    let mut lender = VecLender::new(vec![1, 2, 3]);
    assert_eq!(lender.nth(0), Some(&1));
    assert_eq!(lender.nth(0), Some(&2));
}

#[test]
fn nth_beyond_length() {
    // nth beyond length returns None
    let mut lender = VecLender::new(vec![1, 2, 3]);
    assert_eq!(lender.nth(10), None);
    assert_eq!(lender.next(), None);
}

#[test]
fn take_zero() {
    // take(0) should yield nothing
    let mut lender = VecLender::new(vec![1, 2, 3]).take(0);
    assert_eq!(lender.next(), None);
}

#[test]
fn boundary_skip_zero() {
    // skip(0) should be a no-op
    let mut lender = VecLender::new(vec![1, 2, 3]).skip(0);
    assert_eq!(lender.next(), Some(&1));
}
