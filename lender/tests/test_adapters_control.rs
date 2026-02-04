//! Tests for control flow adapters: Chain, Fuse, Cycle, Rev

#![allow(clippy::unnecessary_fold)]

mod common;
use ::lender::prelude::*;
use common::*;

// ============================================================================
// Chain adapter tests (Lender)
// ============================================================================
#[test]
fn chain_basic_forward_iteration() {
    // Documented semantics: "first yield all lends from self, then all lends from other"
    let mut chained = VecLender::new(vec![1, 2]).chain(VecLender::new(vec![3, 4]));

    // First lender's elements come first
    assert_eq!(chained.next(), Some(&1));
    assert_eq!(chained.next(), Some(&2));
    // Then second lender's elements
    assert_eq!(chained.next(), Some(&3));
    assert_eq!(chained.next(), Some(&4));
    // Then exhausted
    assert_eq!(chained.next(), None);
    // Should remain exhausted (fused behavior from inner Fuse wrappers)
    assert_eq!(chained.next(), None);
}

#[test]
fn chain_double_ended_iteration() {
    // DoubleEndedLender: next_back should yield from the *second* lender first
    let mut chained = VecLender::new(vec![1, 2]).chain(VecLender::new(vec![3, 4]));

    // next_back starts from the end of the second lender
    assert_eq!(chained.next_back(), Some(&4));
    assert_eq!(chained.next_back(), Some(&3));
    // Then from the end of the first lender
    assert_eq!(chained.next_back(), Some(&2));
    assert_eq!(chained.next_back(), Some(&1));
    // Exhausted
    assert_eq!(chained.next_back(), None);
}

#[test]
fn chain_mixed_forward_backward() {
    // Mixed iteration: front and back should not interfere incorrectly
    let mut chained = VecLender::new(vec![1, 2, 3]).chain(VecLender::new(vec![4, 5, 6]));

    // Take from front
    assert_eq!(chained.next(), Some(&1));
    // Take from back
    assert_eq!(chained.next_back(), Some(&6));
    // Continue from front
    assert_eq!(chained.next(), Some(&2));
    // Continue from back
    assert_eq!(chained.next_back(), Some(&5));
    // Should meet in the middle
    assert_eq!(chained.next(), Some(&3));
    assert_eq!(chained.next_back(), Some(&4));
    // Now exhausted
    assert_eq!(chained.next(), None);
    assert_eq!(chained.next_back(), None);
}

#[test]
fn chain_empty_first_lender() {
    // When first lender is empty, should immediately yield from second
    let mut chained = VecLender::new(vec![]).chain(VecLender::new(vec![1, 2]));

    assert_eq!(chained.next(), Some(&1));
    assert_eq!(chained.next(), Some(&2));
    assert_eq!(chained.next(), None);
}

#[test]
fn chain_empty_second_lender() {
    // When second lender is empty, should yield from first then be exhausted
    let mut chained = VecLender::new(vec![1, 2]).chain(VecLender::new(vec![]));

    assert_eq!(chained.next(), Some(&1));
    assert_eq!(chained.next(), Some(&2));
    assert_eq!(chained.next(), None);
}

#[test]
fn chain_both_empty() {
    let mut chained = VecLender::new(vec![]).chain(VecLender::new(vec![]));
    assert_eq!(chained.next(), None);
    assert_eq!(chained.next_back(), None);
}

#[test]
fn chain_count() {
    // count() should return total elements from both lenders
    let chained = VecLender::new(vec![1, 2, 3]).chain(VecLender::new(vec![4, 5]));
    assert_eq!(chained.count(), 5);

    // Empty case
    let chained_empty = VecLender::new(vec![]).chain(VecLender::new(vec![]));
    assert_eq!(chained_empty.count(), 0);
}

#[test]
fn chain_nth() {
    // nth(n) should skip n elements and return the (n+1)th
    let mut chained = VecLender::new(vec![1, 2, 3]).chain(VecLender::new(vec![4, 5, 6]));

    // nth(0) is same as next()
    assert_eq!(chained.nth(0), Some(&1));
    // nth(1) skips one and returns the next
    assert_eq!(chained.nth(1), Some(&3));
    // Now we should be at element 4 (index 3 originally, but we've consumed 0, 1, 2)
    // nth(1) skips 4 and returns 5
    assert_eq!(chained.nth(1), Some(&5));
    // Only 6 left, nth(0) returns it
    assert_eq!(chained.nth(0), Some(&6));
    // Exhausted
    assert_eq!(chained.nth(0), None);
}

#[test]
fn chain_nth_crossing_boundary() {
    // nth that crosses the boundary between first and second lender
    let mut chained = VecLender::new(vec![1, 2]).chain(VecLender::new(vec![3, 4, 5]));

    // Skip past first lender entirely and into second
    assert_eq!(chained.nth(3), Some(&4)); // skip 1,2,3 -> return 4
    assert_eq!(chained.next(), Some(&5));
    assert_eq!(chained.next(), None);
}

#[test]
fn chain_nth_back() {
    // nth_back(n) should skip n elements from the back
    let mut chained = VecLender::new(vec![1, 2, 3]).chain(VecLender::new(vec![4, 5, 6]));

    // nth_back(0) is same as next_back()
    assert_eq!(chained.nth_back(0), Some(&6));
    // nth_back(1) skips 5 and returns 4
    assert_eq!(chained.nth_back(1), Some(&4));
    // Now crossing into first lender: nth_back(1) skips 3 and returns 2
    assert_eq!(chained.nth_back(1), Some(&2));
    // Only 1 left
    assert_eq!(chained.nth_back(0), Some(&1));
    assert_eq!(chained.nth_back(0), None);
}

#[test]
fn chain_last() {
    // last() should return the last element of the second lender (if non-empty)
    let mut chained = VecLender::new(vec![1, 2]).chain(VecLender::new(vec![3, 4]));
    assert_eq!(chained.last(), Some(&4));

    // If second lender is empty, should return last of first
    let mut chained2 = VecLender::new(vec![1, 2]).chain(VecLender::new(vec![]));
    assert_eq!(chained2.last(), Some(&2));

    // If both empty, should return None
    let mut chained3 = VecLender::new(vec![]).chain(VecLender::new(vec![]));
    assert_eq!(chained3.last(), None);
}

#[test]
fn chain_find() {
    // find() should search first lender, then second
    let mut chained = VecLender::new(vec![1, 2, 3]).chain(VecLender::new(vec![4, 5, 6]));

    // Find in first lender
    assert_eq!(chained.find(|x| **x == 2), Some(&2));
    // Now first lender is partially consumed (1 was skipped), continue search
    // find in second lender
    assert_eq!(chained.find(|x| **x > 4), Some(&5));
}

#[test]
fn chain_rfind() {
    // rfind() should search second lender first (from back), then first
    let mut chained = VecLender::new(vec![1, 2, 3]).chain(VecLender::new(vec![4, 5, 6]));

    // rfind in second lender
    assert_eq!(chained.rfind(|x| **x == 5), Some(&5));
    // rfind crosses to first lender
    assert_eq!(chained.rfind(|x| **x < 3), Some(&2));
}

#[test]
fn chain_size_hint() {
    // size_hint should be sum of both lenders' hints
    let chained = VecLender::new(vec![1, 2, 3]).chain(VecLender::new(vec![4, 5]));
    assert_eq!(chained.size_hint(), (5, Some(5)));

    let chained_empty = VecLender::new(vec![]).chain(VecLender::new(vec![]));
    assert_eq!(chained_empty.size_hint(), (0, Some(0)));
}

#[test]
fn chain_fold() {
    // fold should process all elements from both lenders
    let sum = VecLender::new(vec![1, 2, 3])
        .chain(VecLender::new(vec![4, 5, 6]))
        .fold(0, |acc, x| acc + *x);
    assert_eq!(sum, 21); // 1+2+3+4+5+6
}

#[test]
fn chain_rfold() {
    // rfold should process elements in reverse order (second lender first, from back)
    let mut order = Vec::new();
    VecLender::new(vec![1, 2, 3])
        .chain(VecLender::new(vec![4, 5, 6]))
        .rfold((), |(), x| order.push(*x));
    // Should be: 6, 5, 4, 3, 2, 1
    assert_eq!(order, vec![6, 5, 4, 3, 2, 1]);
}

#[test]
fn chain_into_inner() {
    // into_inner should return the original lenders
    let chained = VecLender::new(vec![1, 2]).chain(VecLender::new(vec![3, 4]));
    let (a, b) = chained.into_inner();
    // The returned lenders are the original ones (unwrapped from Fuse)
    assert_eq!(a.data, vec![1, 2]);
    assert_eq!(b.data, vec![3, 4]);
}

#[test]
fn advance_by_chain_additional() {
    let mut chained = VecLender::new(vec![1, 2]).chain(VecLender::new(vec![3, 4]));
    assert_eq!(chained.advance_by(3), Ok(())); // Skip 1, 2, 3
    assert_eq!(chained.next(), Some(&4));
}

#[test]
fn advance_back_by_chain_additional() {
    use lender::DoubleEndedLender;

    let mut chained = VecLender::new(vec![1, 2]).chain(VecLender::new(vec![3, 4]));
    assert_eq!(chained.advance_back_by(3), Ok(())); // Skip 4, 3, 2
    assert_eq!(chained.next(), Some(&1));
}

// ============================================================================
// Fuse adapter tests (Lender)
// Semantics: Once a lender returns None, fuse() ensures it continues to
// return None forever, even if the underlying lender would return Some again.
// ============================================================================

/// A lender that alternates between returning values and None (non-fused behavior)
struct UnfusedLender {
    values: Vec<i32>,
    index: usize,
    skip_next: bool,
}

impl UnfusedLender {
    fn new(values: Vec<i32>) -> Self {
        Self {
            values,
            index: 0,
            skip_next: false,
        }
    }
}

impl<'lend> Lending<'lend> for UnfusedLender {
    type Lend = i32;
}

impl Lender for UnfusedLender {
    check_covariance!();

    fn next(&mut self) -> Option<Lend<'_, Self>> {
        // Simulate non-fused behavior: return None sometimes, then Some again
        if self.skip_next {
            self.skip_next = false;
            return None; // Return None but don't advance
        }
        if self.index < self.values.len() {
            let val = self.values[self.index];
            self.index += 1;
            // After returning a value, toggle to skip next call
            self.skip_next = true;
            Some(val)
        } else {
            None
        }
    }
}

#[test]
fn fuse_basic_iteration() {
    // Basic case: fuse should not change behavior for normal lenders
    let mut fused = VecLender::new(vec![1, 2, 3]).fuse();

    assert_eq!(fused.next(), Some(&1));
    assert_eq!(fused.next(), Some(&2));
    assert_eq!(fused.next(), Some(&3));
    assert_eq!(fused.next(), None);
    // Key semantics: continues to return None
    assert_eq!(fused.next(), None);
    assert_eq!(fused.next(), None);
}

#[test]
fn fuse_guarantees_none_after_exhaustion() {
    // With a non-fused lender that would return Some after None,
    // fuse() should ensure it keeps returning None
    let mut fused = UnfusedLender::new(vec![1, 2, 3]).fuse();

    // First call returns Some(1)
    assert_eq!(fused.next(), Some(1));
    // Underlying lender would return None here, then Some(2)
    // But fuse sets the flag when None is returned
    assert_eq!(fused.next(), None);
    // Now fuse should keep returning None even though underlying would return Some(2)
    assert_eq!(fused.next(), None);
    assert_eq!(fused.next(), None);
}

#[test]
fn fuse_double_ended() {
    let mut fused = VecLender::new(vec![1, 2, 3, 4]).fuse();

    assert_eq!(fused.next(), Some(&1));
    assert_eq!(fused.next_back(), Some(&4));
    assert_eq!(fused.next(), Some(&2));
    assert_eq!(fused.next_back(), Some(&3));
    // Now exhausted
    assert_eq!(fused.next(), None);
    assert_eq!(fused.next_back(), None);
    // Should stay exhausted
    assert_eq!(fused.next(), None);
    assert_eq!(fused.next_back(), None);
}

#[test]
fn fuse_nth() {
    let mut fused = VecLender::new(vec![1, 2, 3, 4, 5]).fuse();

    // nth(2) skips 1, 2 and returns 3
    assert_eq!(fused.nth(2), Some(&3));
    // nth(0) returns next element (4)
    assert_eq!(fused.nth(0), Some(&4));
    // nth(0) returns 5
    assert_eq!(fused.nth(0), Some(&5));
    // Now exhausted
    assert_eq!(fused.nth(0), None);
    // Fused - stays None
    assert_eq!(fused.nth(0), None);
}

#[test]
fn fuse_nth_back() {
    let mut fused = VecLender::new(vec![1, 2, 3, 4, 5]).fuse();

    // nth_back(1) skips 5 and returns 4
    assert_eq!(fused.nth_back(1), Some(&4));
    // nth_back(0) returns 3
    assert_eq!(fused.nth_back(0), Some(&3));
    // Continue
    assert_eq!(fused.nth_back(0), Some(&2));
    assert_eq!(fused.nth_back(0), Some(&1));
    // Exhausted
    assert_eq!(fused.nth_back(0), None);
    // Fused - stays None
    assert_eq!(fused.nth_back(0), None);
}

#[test]
fn fuse_last() {
    // last() should return the last element
    let mut fused = VecLender::new(vec![1, 2, 3]).fuse();
    assert_eq!(fused.last(), Some(&3));

    // After last(), lender is exhausted, should return None
    // But last() consumes, so we need a new fused lender
    let mut fused2 = VecLender::new(vec![]).fuse();
    assert_eq!(fused2.last(), None);
    // Should stay None
    assert_eq!(fused2.last(), None);
}

#[test]
fn fuse_count() {
    // count() should return total elements
    let fused = VecLender::new(vec![1, 2, 3, 4, 5]).fuse();
    assert_eq!(fused.count(), 5);

    let fused_empty = VecLender::new(vec![]).fuse();
    assert_eq!(fused_empty.count(), 0);
}

#[test]
fn fuse_size_hint() {
    let fused = VecLender::new(vec![1, 2, 3]).fuse();
    assert_eq!(fused.size_hint(), (3, Some(3)));

    let fused_empty = VecLender::new(vec![]).fuse();
    assert_eq!(fused_empty.size_hint(), (0, Some(0)));
}

#[test]
fn fuse_size_hint_after_exhaustion() {
    let mut fused = VecLender::new(vec![1, 2]).fuse();

    assert_eq!(fused.size_hint(), (2, Some(2)));
    fused.next();
    assert_eq!(fused.size_hint(), (1, Some(1)));
    fused.next();
    assert_eq!(fused.size_hint(), (0, Some(0)));
    fused.next(); // Returns None, sets flag
    // After exhaustion, size_hint should be (0, Some(0))
    assert_eq!(fused.size_hint(), (0, Some(0)));
}

#[test]
fn fuse_fold() {
    let sum = VecLender::new(vec![1, 2, 3, 4])
        .fuse()
        .fold(0, |acc, x| acc + *x);
    assert_eq!(sum, 10);
}

#[test]
fn fuse_rfold() {
    let mut order = Vec::new();
    VecLender::new(vec![1, 2, 3])
        .fuse()
        .rfold((), |(), x| order.push(*x));
    assert_eq!(order, vec![3, 2, 1]);
}

#[test]
fn fuse_find() {
    let mut fused = VecLender::new(vec![1, 2, 3, 4, 5]).fuse();

    assert_eq!(fused.find(|x| **x > 2), Some(&3));
    assert_eq!(fused.find(|x| **x > 4), Some(&5));
    assert_eq!(fused.find(|x| **x > 10), None);
    // After returning None from find, should stay fused
    assert_eq!(fused.next(), None);
}

#[test]
fn fuse_rfind() {
    let mut fused = VecLender::new(vec![1, 2, 3, 4, 5]).fuse();

    assert_eq!(fused.rfind(|x| **x < 4), Some(&3));
    assert_eq!(fused.rfind(|x| **x < 2), Some(&1));
    assert_eq!(fused.rfind(|x| **x < 0), None);
    // After returning None from rfind, should stay fused
    assert_eq!(fused.next_back(), None);
}

#[test]
fn fuse_exact_size() {
    use lender::ExactSizeLender;

    let mut fused = VecLender::new(vec![1, 2, 3]).fuse();
    assert_eq!(fused.len(), 3);
    assert!(!fused.is_empty());

    fused.next();
    assert_eq!(fused.len(), 2);

    fused.next();
    fused.next();
    assert_eq!(fused.len(), 0);
    assert!(fused.is_empty());

    // After exhaustion, len should remain 0
    fused.next();
    assert_eq!(fused.len(), 0);
    assert!(fused.is_empty());
}

#[test]
fn fuse_into_inner() {
    let fused = VecLender::new(vec![1, 2, 3]).fuse();
    let inner = fused.into_inner();
    assert_eq!(inner.data, vec![1, 2, 3]);
}

#[test]
fn fuse_after_none() {
    // Create a lender that returns None then Some (normally not allowed behavior)
    struct FlickeringLender {
        count: i32,
    }
    impl<'lend> Lending<'lend> for FlickeringLender {
        type Lend = i32;
    }
    impl Lender for FlickeringLender {
        lender::check_covariance!();
        fn next(&mut self) -> Option<Lend<'_, Self>> {
            self.count += 1;
            if self.count == 2 {
                None // Returns None in the middle
            } else if self.count <= 4 {
                Some(self.count)
            } else {
                None
            }
        }
    }

    let mut fused = FlickeringLender { count: 0 }.fuse();
    assert_eq!(fused.next(), Some(1));
    assert_eq!(fused.next(), None); // Fuse returns None
    assert_eq!(fused.next(), None); // And stays None
    assert_eq!(fused.next(), None); // Even though underlying would return Some
}

// ============================================================================
// Cycle adapter tests (Lender)
// Semantics: cycle() repeats the lender infinitely
// ============================================================================

#[test]
fn cycle_basic() {
    let mut cycled = VecLender::new(vec![1, 2, 3]).cycle();

    // First cycle
    assert_eq!(cycled.next(), Some(&1));
    assert_eq!(cycled.next(), Some(&2));
    assert_eq!(cycled.next(), Some(&3));
    // Second cycle
    assert_eq!(cycled.next(), Some(&1));
    assert_eq!(cycled.next(), Some(&2));
    assert_eq!(cycled.next(), Some(&3));
    // Third cycle
    assert_eq!(cycled.next(), Some(&1));
}

#[test]
fn cycle_single_element() {
    let mut cycled = VecLender::new(vec![42]).cycle();

    for _ in 0..10 {
        assert_eq!(cycled.next(), Some(&42));
    }
}

#[test]
fn cycle_empty() {
    // cycle() on empty lender should return None forever
    let mut cycled = VecLender::new(vec![]).cycle();

    assert_eq!(cycled.next(), None);
    assert_eq!(cycled.next(), None);
    assert_eq!(cycled.next(), None);
}

#[test]
fn cycle_size_hint() {
    // For non-empty lender, size_hint is (usize::MAX, None) - infinite
    let cycled = VecLender::new(vec![1, 2, 3]).cycle();
    let (lower, upper) = cycled.size_hint();
    assert_eq!(lower, usize::MAX);
    assert_eq!(upper, None);

    // For empty lender, size_hint is (0, Some(0))
    let cycled_empty = VecLender::new(vec![]).cycle();
    assert_eq!(cycled_empty.size_hint(), (0, Some(0)));
}

#[test]
fn cycle_multiple_rounds() {
    let mut cycle = VecLender::new(vec![1, 2]).cycle();
    assert_eq!(cycle.next(), Some(&1));
    assert_eq!(cycle.next(), Some(&2));
    assert_eq!(cycle.next(), Some(&1)); // Start of second cycle
    assert_eq!(cycle.next(), Some(&2));
    assert_eq!(cycle.next(), Some(&1)); // Third cycle
}

#[test]
fn cycle_size_hint_additional() {
    let empty_cycle = VecLender::new(Vec::<i32>::new()).cycle();
    assert_eq!(empty_cycle.size_hint(), (0, Some(0)));

    let cycle = VecLender::new(vec![1, 2]).cycle();
    assert_eq!(cycle.size_hint(), (usize::MAX, None)); // Infinite
}

#[test]
fn cycle_try_fold_additional() {
    // Take first 5 elements from cycling [1, 2]
    let result: Option<i32> = VecLender::new(vec![1, 2])
        .cycle()
        .take(5)
        .try_fold(0, |acc, x| Some(acc + *x));
    // 1 + 2 + 1 + 2 + 1 = 7
    assert_eq!(result, Some(7));
}

#[test]
fn cycle_advance_by_within_first_cycle() {
    let mut cycled = VecLender::new(vec![1, 2, 3]).cycle();
    // Advance 2 within the first cycle
    assert_eq!(cycled.advance_by(2), Ok(()));
    // Next element should be 3 (remaining in first cycle)
    assert_eq!(cycled.next(), Some(&3));
}

#[test]
fn cycle_advance_by_across_cycles() {
    let mut cycled = VecLender::new(vec![1, 2, 3]).cycle();
    // Advance 5: skips [1,2,3] (first cycle) + [1,2] (second cycle)
    assert_eq!(cycled.advance_by(5), Ok(()));
    // Next element should be 3 (remaining in second cycle)
    assert_eq!(cycled.next(), Some(&3));
}

#[test]
fn cycle_advance_by_exact_cycle_boundary() {
    let mut cycled = VecLender::new(vec![1, 2, 3]).cycle();
    // Advance exactly one full cycle
    assert_eq!(cycled.advance_by(3), Ok(()));
    // Next element should be 1 (start of second cycle)
    assert_eq!(cycled.next(), Some(&1));
}

#[test]
fn cycle_advance_by_empty() {
    use core::num::NonZeroUsize;

    let mut cycled = VecLender::new(vec![]).cycle();
    assert_eq!(cycled.advance_by(0), Ok(()));
    assert_eq!(cycled.advance_by(1), Err(NonZeroUsize::new(1).unwrap()));
}

// ============================================================================
// Rev adapter tests
// ============================================================================

#[test]
fn rev_basic() {
    let mut reversed = VecLender::new(vec![1, 2, 3]).rev();

    assert_eq!(reversed.next(), Some(&3));
    assert_eq!(reversed.next(), Some(&2));
    assert_eq!(reversed.next(), Some(&1));
    assert_eq!(reversed.next(), None);
}

#[test]
fn rev_double_ended() {
    let mut reversed = VecLender::new(vec![1, 2, 3]).rev();

    // next_back on Rev is next on original
    assert_eq!(reversed.next_back(), Some(&1));
    assert_eq!(reversed.next_back(), Some(&2));
}

#[test]
fn rev_fold() {
    let mut order = Vec::new();
    VecLender::new(vec![1, 2, 3])
        .rev()
        .fold((), |(), x| order.push(*x));
    assert_eq!(order, vec![3, 2, 1]);
}

#[test]
fn rev_rfold() {
    let mut order = Vec::new();
    VecLender::new(vec![1, 2, 3])
        .rev()
        .rfold((), |(), x| order.push(*x));
    // rfold on Rev goes forward on original
    assert_eq!(order, vec![1, 2, 3]);
}

#[test]
fn rev_nth() {
    let mut reversed = VecLender::new(vec![1, 2, 3, 4, 5]).rev();
    // nth(2) on reversed: skips 5, 4 and returns 3
    assert_eq!(reversed.nth(2), Some(&3));
}

#[test]
fn rev_nth_back() {
    let mut reversed = VecLender::new(vec![1, 2, 3, 4, 5]).rev();
    // nth_back(1) on reversed: skips 1 and returns 2
    assert_eq!(reversed.nth_back(1), Some(&2));
}

#[test]
fn rev_double_rev() {
    // Reversing twice should give original order
    let mut lender = VecLender::new(vec![1, 2, 3]).rev().rev();
    assert_eq!(lender.next(), Some(&1));
    assert_eq!(lender.next(), Some(&2));
    assert_eq!(lender.next(), Some(&3));
}

#[test]
fn rev_try_fold_additional() {
    let result: Option<i32> = VecLender::new(vec![1, 2, 3])
        .rev()
        .try_fold(0, |acc, x| Some(acc + *x));
    assert_eq!(result, Some(6));
}

#[test]
fn rev_try_rfold_additional() {
    let result: Option<i32> = VecLender::new(vec![1, 2, 3])
        .rev()
        .try_rfold(0, |acc, x| Some(acc + *x));
    assert_eq!(result, Some(6));
}

