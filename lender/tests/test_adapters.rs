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
// Peekable adapter tests (Lender)
// Semantics: peek() returns a reference to the next element without consuming it.
// ============================================================================

#[test]
fn peekable_basic() {
    let mut peekable = VecLender::new(vec![1, 2, 3]).peekable();

    // peek() returns reference to next element
    assert_eq!(peekable.peek(), Some(&&1));
    // peek() again returns same element (not consumed)
    assert_eq!(peekable.peek(), Some(&&1));
    // next() consumes it
    assert_eq!(peekable.next(), Some(&1));

    // Now peek sees 2
    assert_eq!(peekable.peek(), Some(&&2));
    assert_eq!(peekable.next(), Some(&2));

    // Continue
    assert_eq!(peekable.peek(), Some(&&3));
    assert_eq!(peekable.next(), Some(&3));

    // Exhausted
    assert_eq!(peekable.peek(), None);
    assert_eq!(peekable.next(), None);
}

#[test]
fn peekable_peek_mut() {
    // Note: VecLender now yields &i32, so peek_mut() returns &mut &i32.
    // We can't modify the underlying value, but can still test peek_mut exists
    let mut peekable = VecLender::new(vec![1, 2, 3]).peekable();

    // peek_mut returns a mutable reference to the lend
    assert!(peekable.peek_mut().is_some());
    assert_eq!(peekable.next(), Some(&1));
    assert_eq!(peekable.next(), Some(&2));
}

#[test]
fn peekable_next_if() {
    let mut peekable = VecLender::new(vec![1, 2, 3, 4]).peekable();

    // next_if returns Some if predicate matches
    assert_eq!(peekable.next_if(|x| **x == 1), Some(&1));
    // next_if returns None if predicate doesn't match, element is put back
    assert_eq!(peekable.next_if(|x| **x == 10), None);
    // Element wasn't consumed
    assert_eq!(peekable.peek(), Some(&&2));
    assert_eq!(peekable.next(), Some(&2));
}

#[test]
fn peekable_next_if_eq() {
    let mut peekable = VecLender::new(vec![1, 2, 3]).peekable();

    // next_if_eq returns Some if element equals given value
    assert_eq!(peekable.next_if_eq(&&1), Some(&1));
    // next_if_eq returns None if not equal
    assert_eq!(peekable.next_if_eq(&&10), None);
    assert_eq!(peekable.next(), Some(&2));
}

#[test]
fn peekable_empty() {
    let mut peekable = VecLender::new(vec![]).peekable();
    assert_eq!(peekable.peek(), None);
    assert_eq!(peekable.next(), None);
}

#[test]
fn peekable_count() {
    // count() should include peeked element
    let mut peekable = VecLender::new(vec![1, 2, 3, 4, 5]).peekable();
    peekable.peek(); // peek first element
    assert_eq!(peekable.count(), 5);
}

#[test]
fn peekable_nth() {
    let mut peekable = VecLender::new(vec![1, 2, 3, 4, 5]).peekable();

    // nth(2) should skip 1, 2 and return 3
    assert_eq!(peekable.nth(2), Some(&3));
    assert_eq!(peekable.next(), Some(&4));
}

#[test]
fn peekable_last() {
    let mut peekable = VecLender::new(vec![1, 2, 3]).peekable();
    assert_eq!(peekable.last(), Some(&3));
}

#[test]
fn peekable_fold() {
    let sum = VecLender::new(vec![1, 2, 3, 4])
        .peekable()
        .fold(0, |acc, x| acc + *x);
    assert_eq!(sum, 10);
}

#[test]
fn peekable_size_hint() {
    let mut peekable = VecLender::new(vec![1, 2, 3]).peekable();
    assert_eq!(peekable.size_hint(), (3, Some(3)));

    peekable.peek();
    // After peek, size_hint should still be correct
    assert_eq!(peekable.size_hint(), (3, Some(3)));

    peekable.next();
    assert_eq!(peekable.size_hint(), (2, Some(2)));
}

#[test]
fn peekable_into_inner() {
    let peekable = VecLender::new(vec![1, 2, 3]).peekable();
    let inner = peekable.into_inner();
    assert_eq!(inner.data, vec![1, 2, 3]);
}

#[test]
fn peekable_peek_multiple() {
    let mut peekable = VecLender::new(vec![1, 2, 3]).peekable();
    assert_eq!(peekable.peek(), Some(&&1));
    assert_eq!(peekable.peek(), Some(&&1)); // Peeking again returns same value
    assert_eq!(peekable.next(), Some(&1));
    assert_eq!(peekable.peek(), Some(&&2));
}

#[test]
fn peekable_peek_mut_additional() {
    let mut peekable = VecLender::new(vec![1, 2, 3]).peekable();
    // VecLender yields &i32, so peek_mut returns &mut &i32 - can't modify underlying value
    assert!(peekable.peek_mut().is_some());
    assert_eq!(peekable.next(), Some(&1));
    assert_eq!(peekable.next(), Some(&2)); // Original unchanged
}

#[test]
fn peekable_next_if_additional() {
    let mut peekable = VecLender::new(vec![1, 2, 3]).peekable();
    assert_eq!(peekable.next_if(|x| **x < 2), Some(&1));
    assert_eq!(peekable.next_if(|x| **x < 2), None); // 2 is not < 2
    assert_eq!(peekable.next(), Some(&2));
}

#[test]
fn peekable_next_if_eq_additional() {
    let mut peekable = VecLender::new(vec![1, 2, 3]).peekable();
    assert_eq!(peekable.next_if_eq(&&1), Some(&1));
    assert_eq!(peekable.next_if_eq(&&1), None); // Next is 2, not 1
    assert_eq!(peekable.next_if_eq(&&2), Some(&2));
}

#[test]
fn peekable_fold_additional() {
    let sum = VecLender::new(vec![1, 2, 3])
        .peekable()
        .fold(0, |acc, x| acc + *x);
    assert_eq!(sum, 6);
}

#[test]
fn peekable_try_fold_additional() {
    let result: Option<i32> = VecLender::new(vec![1, 2, 3])
        .peekable()
        .try_fold(0, |acc, x| Some(acc + *x));
    assert_eq!(result, Some(6));
}

#[test]
fn peekable_size_hint_after_peek() {
    let mut peekable = VecLender::new(vec![1, 2, 3]).peekable();
    assert_eq!(peekable.size_hint(), (3, Some(3)));
    peekable.peek();
    // After peeking, size_hint should still be accurate
    assert_eq!(peekable.size_hint(), (3, Some(3)));
    peekable.next();
    assert_eq!(peekable.size_hint(), (2, Some(2)));
}

// Peekable::nth with peeked value when n == 0 (covers unsafe transmute at line 138-139)
#[test]
fn peekable_nth_zero_with_peeked() {
    let mut peekable = VecLender::new(vec![1, 2, 3]).peekable();
    // Peek to store a value
    assert_eq!(peekable.peek(), Some(&&1));
    // nth(0) should return the peeked value through the unsafe transmute path
    assert_eq!(peekable.nth(0), Some(&1));
    assert_eq!(peekable.next(), Some(&2));
}

// Peekable::last with peeked value (covers unsafe transmute at line 153)
#[test]
fn peekable_last_with_peeked_only() {
    let mut peekable = VecLender::new(vec![1]).peekable();
    // Peek the only value
    assert_eq!(peekable.peek(), Some(&&1));
    // last() should return the peeked value through the unsafe transmute path
    // when the underlying lender returns None
    assert_eq!(peekable.last(), Some(&1));
}

// Peekable::next_back with peeked value when underlying lender is empty
// (covers unsafe transmute at line 208)
#[test]
fn peekable_next_back_with_peeked_exhausted() {
    use lender::DoubleEndedLender;

    let mut peekable = VecLender::new(vec![1]).peekable();
    // Peek the only value
    assert_eq!(peekable.peek(), Some(&&1));
    // next_back should return the peeked value through the unsafe transmute path
    assert_eq!(peekable.next_back(), Some(&1));
    assert_eq!(peekable.next(), None);
}

#[test]
fn peekable_rfold_with_peeked() {
    let mut peekable = VecLender::new(vec![1, 2, 3]).peekable();
    assert_eq!(peekable.peek(), Some(&&1));
    // rfold processes back-to-front: 3, 2, then peeked 1
    let result = peekable.rfold(Vec::new(), |mut acc, &x| {
        acc.push(x);
        acc
    });
    assert_eq!(result, vec![3, 2, 1]);
}

#[test]
fn peekable_try_rfold_with_peeked_complete() {
    use lender::DoubleEndedLender;

    let mut peekable = VecLender::new(vec![1, 2, 3]).peekable();
    assert_eq!(peekable.peek(), Some(&&1));
    // try_rfold processes back-to-front: 3, 2, then peeked 1
    let result: Option<Vec<i32>> = peekable.try_rfold(Vec::new(), |mut acc, &x| {
        acc.push(x);
        Some(acc)
    });
    assert_eq!(result, Some(vec![3, 2, 1]));
}

// Covers the ControlFlow::Break path in try_rfold where the peeked value
// is stored back (peekable.rs lines 266-269).
#[test]
fn peekable_try_rfold_with_peeked_break() {
    use lender::DoubleEndedLender;

    let mut peekable = VecLender::new(vec![1, 2, 3]).peekable();
    assert_eq!(peekable.peek(), Some(&&1));
    // Inner lender has [2, 3]. try_rfold processes back-to-front:
    // 3 (continue, acc=3), then 2 (break via None).
    let result: Option<i32> =
        peekable.try_rfold(0, |acc, &x| if x == 2 { None } else { Some(acc + x) });
    assert_eq!(result, None);
    // The peeked value should have been stored back
    assert_eq!(peekable.next(), Some(&1));
    // Inner lender was fully consumed by try_rfold
    assert_eq!(peekable.next(), None);
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

// ============================================================================
// Zip adapter tests
// ============================================================================

#[test]
fn zip_basic() {
    let mut zipped = VecLender::new(vec![1, 2, 3]).zip(VecLender::new(vec![10, 20, 30]));

    assert_eq!(zipped.next(), Some((&1, &10)));
    assert_eq!(zipped.next(), Some((&2, &20)));
    assert_eq!(zipped.next(), Some((&3, &30)));
    assert_eq!(zipped.next(), None);
}

#[test]
fn zip_different_lengths() {
    // Stops at shorter lender
    let mut zipped = VecLender::new(vec![1, 2]).zip(VecLender::new(vec![10, 20, 30]));

    assert_eq!(zipped.next(), Some((&1, &10)));
    assert_eq!(zipped.next(), Some((&2, &20)));
    assert_eq!(zipped.next(), None);
}

#[test]
fn zip_double_ended() {
    let mut zipped = VecLender::new(vec![1, 2, 3]).zip(VecLender::new(vec![10, 20, 30]));

    assert_eq!(zipped.next_back(), Some((&3, &30)));
    assert_eq!(zipped.next_back(), Some((&2, &20)));
    assert_eq!(zipped.next_back(), Some((&1, &10)));
    assert_eq!(zipped.next_back(), None);
}

#[test]
fn zip_fold() {
    let sum = VecLender::new(vec![1, 2, 3])
        .zip(VecLender::new(vec![10, 20, 30]))
        .fold(0, |acc, (a, b)| acc + *a + *b);
    // (1+10) + (2+20) + (3+30) = 66
    assert_eq!(sum, 66);
}

#[test]
fn zip_nth() {
    let mut zipped = VecLender::new(vec![1, 2, 3, 4]).zip(VecLender::new(vec![10, 20, 30, 40]));
    assert_eq!(zipped.nth(2), Some((&3, &30)));
    assert_eq!(zipped.next(), Some((&4, &40)));
}

#[test]
fn zip_size_hint_additional() {
    let zipped = VecLender::new(vec![1, 2, 3]).zip(VecLender::new(vec![4, 5, 6, 7]));
    // Zip takes the minimum
    assert_eq!(zipped.size_hint(), (3, Some(3)));
}

#[test]
fn zip_try_fold_additional() {
    let result: Option<i32> = VecLender::new(vec![1, 2, 3])
        .zip(VecLender::new(vec![4, 5, 6]))
        .try_fold(0, |acc, (a, b)| Some(acc + *a + *b));
    assert_eq!(result, Some(21)); // (1+4) + (2+5) + (3+6) = 5 + 7 + 9 = 21
}

#[test]
fn zip_try_rfold_additional() {
    let result: Option<i32> = VecLender::new(vec![1, 2, 3])
        .zip(VecLender::new(vec![4, 5, 6]))
        .try_rfold(0, |acc, (a, b)| Some(acc + *a + *b));
    assert_eq!(result, Some(21));
}

#[test]
fn zip_into_inner() {
    let zip = VecLender::new(vec![1, 2, 3]).zip(VecLender::new(vec![4, 5, 6]));
    let (a, b) = zip.into_inner();
    assert_eq!(a.count(), 3);
    assert_eq!(b.count(), 3);
}

#[test]
fn zip_rfold() {
    let mut values = Vec::new();
    VecLender::new(vec![1, 2, 3])
        .zip(VecLender::new(vec![10, 20, 30]))
        .rfold((), |(), (a, b)| {
            values.push((*a, *b));
        });
    assert_eq!(values, vec![(3, 30), (2, 20), (1, 10)]);
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
// Mutate adapter tests
// ============================================================================

#[test]
fn mutate_basic() {
    let mut data = vec![1, 2, 3];
    let mut lender = lender::from_iter(data.iter_mut()).mutate(|x| **x *= 10);

    assert_eq!(lender.next().map(|x| *x), Some(10));
    assert_eq!(lender.next().map(|x| *x), Some(20));
    assert_eq!(lender.next().map(|x| *x), Some(30));
}

#[test]
fn mutate_fold() {
    let mut data = vec![1, 2, 3];
    let sum = lender::from_iter(data.iter_mut())
        .mutate(|x| **x *= 10)
        .fold(0, |acc, x| acc + *x);
    assert_eq!(sum, 60);
}

#[test]
fn mutate_double_ended() {
    let mut data = vec![1, 2, 3];
    let mut mutated = lender::from_iter(data.iter_mut()).mutate(|x| **x += 100);

    assert_eq!(mutated.next_back().map(|x| *x), Some(103));
    assert_eq!(mutated.next().map(|x| *x), Some(101));
    assert_eq!(mutated.next_back().map(|x| *x), Some(102));
}

#[test]
fn mutate_size_hint_additional() {
    let data = vec![1, 2, 3];
    let lender = lender::from_iter(data.iter()).mutate(|_| {});
    assert_eq!(lender.size_hint(), (3, Some(3)));
}

#[test]
fn mutate_try_fold_additional() {
    let mut data = vec![1, 2, 3];
    let result: Option<i32> = lender::from_iter(data.iter_mut())
        .mutate(|x| **x *= 2)
        .try_fold(0, |acc, x| Some(acc + *x));
    assert_eq!(result, Some(12));
}

#[test]
fn mutate_into_inner_additional() {
    let data = vec![1, 2, 3];
    let mutate = lender::from_iter(data.iter()).mutate(|_| {});
    let lender = mutate.into_inner();
    assert_eq!(lender.count(), 3);
}

#[test]
fn mutate_into_parts_additional() {
    let data = vec![1, 2, 3];
    let mutate = lender::from_iter(data.iter()).mutate(|_| {});
    let (lender, _f) = mutate.into_parts();
    assert_eq!(lender.count(), 3);
}

// ============================================================================
// Intersperse adapter tests (Lender)
// Semantics: insert separator between elements
// ============================================================================

#[test]
fn intersperse_basic() {
    let mut interspersed = VecLender::new(vec![1, 2, 3]).intersperse(&0);

    assert_eq!(interspersed.next(), Some(&1));
    assert_eq!(interspersed.next(), Some(&0)); // separator
    assert_eq!(interspersed.next(), Some(&2));
    assert_eq!(interspersed.next(), Some(&0)); // separator
    assert_eq!(interspersed.next(), Some(&3));
    assert_eq!(interspersed.next(), None);
}

#[test]
fn intersperse_single_element() {
    let mut interspersed = VecLender::new(vec![42]).intersperse(&0);

    assert_eq!(interspersed.next(), Some(&42));
    assert_eq!(interspersed.next(), None);
}

#[test]
fn intersperse_empty() {
    let mut interspersed = VecLender::new(vec![]).intersperse(&0);
    assert_eq!(interspersed.next(), None);
}

#[test]
fn intersperse_fold() {
    let sum = VecLender::new(vec![1, 2, 3])
        .intersperse(&10)
        .fold(0, |acc, x| acc + *x);
    // 1 + 10 + 2 + 10 + 3 = 26
    assert_eq!(sum, 26);
}

#[test]
fn intersperse_with_basic() {
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
fn intersperse_single_element_additional() {
    let mut lender = VecLender::new(vec![42]).intersperse(&0);
    assert_eq!(lender.next(), Some(&42));
    assert_eq!(lender.next(), None);
}

#[test]
fn intersperse_empty_additional() {
    let mut lender = VecLender::new(vec![]).intersperse(&0);
    assert_eq!(lender.next(), None);
}

#[test]
fn intersperse_into_inner() {
    let intersperse = VecLender::new(vec![1, 2, 3]).intersperse(&0);
    let lender = intersperse.into_inner();
    assert_eq!(lender.count(), 3);
}

#[test]
fn intersperse_try_fold_additional() {
    let result: Option<i32> = VecLender::new(vec![1, 2, 3])
        .intersperse(&10)
        .try_fold(0, |acc, x| Some(acc + *x));
    // 1 + 10 + 2 + 10 + 3 = 26
    assert_eq!(result, Some(26));
}

// Intersperse with separator clone (covers unsafe at line 62)
#[test]
fn intersperse_separator_coverage() {
    let mut intersperse = VecLender::new(vec![1, 2, 3]).intersperse(&0);
    // Consume all to exercise the separator clone path
    let mut results = Vec::new();
    while let Some(x) = intersperse.next() {
        results.push(*x);
    }
    assert_eq!(results, vec![1, 0, 2, 0, 3]);
}

// IntersperseWith (covers unsafe at line 142)
#[test]
fn intersperse_with_coverage() {
    let mut intersperse = VecLender::new(vec![1, 2, 3]).intersperse_with(|| &0);
    let mut results = Vec::new();
    while let Some(x) = intersperse.next() {
        results.push(*x);
    }
    assert_eq!(results, vec![1, 0, 2, 0, 3]);
}

#[test]
fn intersperse_try_fold_early_exit() {
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
fn intersperse_try_fold_empty() {
    let result: Option<i32> = VecLender::new(vec![])
        .intersperse(&10)
        .try_fold(0, |acc, x| Some(acc + *x));
    assert_eq!(result, Some(0));
}

#[test]
fn intersperse_try_fold_single() {
    let result: Option<i32> = VecLender::new(vec![42])
        .intersperse(&10)
        .try_fold(0, |acc, x| Some(acc + *x));
    assert_eq!(result, Some(42));
}

#[test]
fn intersperse_fold_empty() {
    let sum = VecLender::new(vec![])
        .intersperse(&10)
        .fold(0, |acc, x: &i32| acc + *x);
    assert_eq!(sum, 0);
}

#[test]
fn intersperse_fold_single() {
    let sum = VecLender::new(vec![42])
        .intersperse(&10)
        .fold(0, |acc, x| acc + *x);
    assert_eq!(sum, 42);
}

#[test]
fn intersperse_with_try_fold() {
    let result: Option<i32> = VecLender::new(vec![1, 2, 3])
        .intersperse_with(|| &10)
        .try_fold(0, |acc, x| Some(acc + *x));
    // 1 + 10 + 2 + 10 + 3 = 26
    assert_eq!(result, Some(26));
}

#[test]
fn intersperse_with_try_fold_early_exit() {
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
fn intersperse_with_try_fold_empty() {
    let result: Option<i32> = VecLender::new(vec![])
        .intersperse_with(|| &10)
        .try_fold(0, |acc, x| Some(acc + *x));
    assert_eq!(result, Some(0));
}

#[test]
fn intersperse_with_try_fold_single() {
    let result: Option<i32> = VecLender::new(vec![42])
        .intersperse_with(|| &10)
        .try_fold(0, |acc, x| Some(acc + *x));
    assert_eq!(result, Some(42));
}

#[test]
fn intersperse_with_fold() {
    let sum = VecLender::new(vec![1, 2, 3])
        .intersperse_with(|| &10)
        .fold(0, |acc, x| acc + *x);
    // 1 + 10 + 2 + 10 + 3 = 26
    assert_eq!(sum, 26);
}

#[test]
fn intersperse_with_fold_empty() {
    let sum = VecLender::new(vec![])
        .intersperse_with(|| &10)
        .fold(0, |acc, x: &i32| acc + *x);
    assert_eq!(sum, 0);
}

#[test]
fn intersperse_with_fold_single() {
    let sum = VecLender::new(vec![42])
        .intersperse_with(|| &10)
        .fold(0, |acc, x| acc + *x);
    assert_eq!(sum, 42);
}

#[test]
fn intersperse_for_each() {
    let mut items = Vec::new();
    VecLender::new(vec![1, 2, 3])
        .intersperse(&0)
        .for_each(|x| items.push(*x));
    assert_eq!(items, vec![1, 0, 2, 0, 3]);
}

#[test]
fn intersperse_with_for_each() {
    let mut items = Vec::new();
    VecLender::new(vec![1, 2, 3])
        .intersperse_with(|| &0)
        .for_each(|x| items.push(*x));
    assert_eq!(items, vec![1, 0, 2, 0, 3]);
}

#[test]
fn intersperse_count() {
    let count = VecLender::new(vec![1, 2, 3]).intersperse(&0).count();
    assert_eq!(count, 5); // 3 elements + 2 separators
}

#[test]
fn intersperse_with_count() {
    let count = VecLender::new(vec![1, 2, 3])
        .intersperse_with(|| &0)
        .count();
    assert_eq!(count, 5);
}

// ============================================================================
// Chunk adapter tests
// ============================================================================

#[test]
fn chunk_basic() {
    let mut lender = VecLender::new(vec![1, 2, 3, 4, 5]);
    let mut chunk = lender.next_chunk(3);

    assert_eq!(chunk.next(), Some(&1));
    assert_eq!(chunk.next(), Some(&2));
    assert_eq!(chunk.next(), Some(&3));
    assert_eq!(chunk.next(), None);

    // Remaining elements
    assert_eq!(lender.next(), Some(&4));
    assert_eq!(lender.next(), Some(&5));
}

#[test]
fn chunk_larger_than_remaining() {
    let mut lender = VecLender::new(vec![1, 2]);
    let mut chunk = lender.next_chunk(5);

    assert_eq!(chunk.next(), Some(&1));
    assert_eq!(chunk.next(), Some(&2));
    assert_eq!(chunk.next(), None);
}

#[test]
fn chunk_empty_lender() {
    let mut lender = VecLender::new(vec![]);
    let mut chunk = lender.next_chunk(3);
    assert_eq!(chunk.next(), None);
}

#[test]
fn chunk_size_hint() {
    let mut chunky = VecLender::new(vec![1, 2, 3, 4, 5]).chunky(3);
    let chunk = chunky.next().unwrap();
    // Chunk has max 3 elements, underlying has 5
    assert_eq!(chunk.size_hint(), (3, Some(3)));
}

#[test]
fn chunk_into_parts() {
    let mut chunky = VecLender::new(vec![1, 2, 3]).chunky(2);
    let chunk = chunky.next().unwrap();
    let (lender, remaining) = chunk.into_parts();
    assert_eq!(remaining, 2);
    assert_eq!(lender.count(), 3); // Original lender still has elements
}

// ============================================================================
// Chunky adapter tests
// Semantics: yields lenders that each return chunk_size elements
// ============================================================================

#[test]
fn chunky_basic() {
    let mut chunky = VecLender::new(vec![1, 2, 3, 4, 5, 6]).chunky(2);

    // First chunk: 1, 2
    let mut chunk1 = chunky.next().unwrap();
    assert_eq!(chunk1.next(), Some(&1));
    assert_eq!(chunk1.next(), Some(&2));
    assert_eq!(chunk1.next(), None);

    // Second chunk: 3, 4
    let mut chunk2 = chunky.next().unwrap();
    assert_eq!(chunk2.next(), Some(&3));
    assert_eq!(chunk2.next(), Some(&4));
    assert_eq!(chunk2.next(), None);

    // Third chunk: 5, 6
    let mut chunk3 = chunky.next().unwrap();
    assert_eq!(chunk3.next(), Some(&5));
    assert_eq!(chunk3.next(), Some(&6));
    assert_eq!(chunk3.next(), None);

    // No more chunks
    assert!(chunky.next().is_none());
}

#[test]
fn chunky_uneven() {
    // 5 elements with chunk size 2 = 3 chunks (2, 2, 1)
    let mut chunky = VecLender::new(vec![1, 2, 3, 4, 5]).chunky(2);

    let mut chunk1 = chunky.next().unwrap();
    assert_eq!(chunk1.next(), Some(&1));
    assert_eq!(chunk1.next(), Some(&2));
    assert_eq!(chunk1.next(), None);

    let mut chunk2 = chunky.next().unwrap();
    assert_eq!(chunk2.next(), Some(&3));
    assert_eq!(chunk2.next(), Some(&4));
    assert_eq!(chunk2.next(), None);

    // Last chunk has only 1 element
    let mut chunk3 = chunky.next().unwrap();
    assert_eq!(chunk3.next(), Some(&5));
    assert_eq!(chunk3.next(), None);

    assert!(chunky.next().is_none());
}

#[test]
fn chunky_size_hint() {
    let chunky = VecLender::new(vec![1, 2, 3, 4, 5, 6]).chunky(2);
    assert_eq!(chunky.size_hint(), (3, Some(3)));

    let chunky2 = VecLender::new(vec![1, 2, 3, 4, 5]).chunky(2);
    assert_eq!(chunky2.size_hint(), (3, Some(3))); // ceil(5/2) = 3
}

#[test]
fn chunky_count() {
    let chunky = VecLender::new(vec![1, 2, 3, 4, 5, 6, 7]).chunky(3);
    assert_eq!(chunky.count(), 3); // ceil(7/3) = 3
}

#[test]
fn chunky_fold() {
    // Count number of chunks
    let num_chunks = VecLender::new(vec![1, 2, 3, 4, 5])
        .chunky(2)
        .fold(0, |acc, _chunk| acc + 1);
    assert_eq!(num_chunks, 3);
}

#[test]
fn chunky_size_hint_decreases() {
    let mut chunky = VecLender::new(vec![1, 2, 3, 4, 5, 6]).chunky(2);
    // 6 elements, chunk_size 2 -> 3 chunks
    assert_eq!(chunky.size_hint(), (3, Some(3)));
    chunky.next();
    assert_eq!(chunky.size_hint(), (2, Some(2)));
    chunky.next();
    assert_eq!(chunky.size_hint(), (1, Some(1)));
    chunky.next();
    assert_eq!(chunky.size_hint(), (0, Some(0)));

    // With uneven division: 5 elements, chunk_size 2 -> ceil(5/2) = 3 chunks
    let chunky2 = VecLender::new(vec![1, 2, 3, 4, 5]).chunky(2);
    assert_eq!(chunky2.size_hint(), (3, Some(3)));
}

#[test]
fn chunky_try_fold() {
    // Test try_fold - if we only consume part of each chunk, the unconsumed
    // elements are lost and the next chunk starts from the current position.
    // This tests the "partial consumption" case.
    let result: Option<i32> =
        VecLender::new(vec![1, 2, 3, 4, 5, 6])
            .chunky(2)
            .try_fold(0, |acc, mut chunk| {
                // Only consume one element from each chunk
                if let Some(first) = chunk.next() {
                    Some(acc + first)
                } else {
                    Some(acc)
                }
            });
    // Since we only consume 1 element per chunk iteration:
    // - Iteration 1: chunk.next() gets 1
    // - Iteration 2: chunk.next() gets 2 (not 3, because we didn't consume element 2)
    // - Iteration 3: chunk.next() gets 3
    // This is expected behavior - Chunky tracks the number of chunks to yield,
    // but doesn't force consumption of all elements in each chunk.
    assert_eq!(result, Some(6)); // 1 + 2 + 3 = 6
}

#[test]
fn chunky_try_fold_full_consumption() {
    // When each chunk is fully consumed, we get the expected chunked results
    let result: Option<i32> =
        VecLender::new(vec![1, 2, 3, 4, 5, 6])
            .chunky(2)
            .try_fold(0, |acc, chunk| {
                // Consume all elements in the chunk
                let chunk_sum = chunk.fold(0, |a, x| a + x);
                Some(acc + chunk_sum)
            });
    // Chunks: [1,2], [3,4], [5,6] with full consumption
    // Sums: 3, 7, 11 -> total: 21
    assert_eq!(result, Some(21));
}

#[test]
fn chunky_into_parts() {
    let chunky = VecLender::new(vec![1, 2, 3]).chunky(2);
    let (lender, chunk_size) = chunky.into_parts();
    assert_eq!(chunk_size, 2);
    assert_eq!(lender.count(), 3);
}

#[test]
fn chunky_into_inner() {
    let chunky = VecLender::new(vec![1, 2, 3]).chunky(2);
    let lender = chunky.into_inner();
    assert_eq!(lender.count(), 3);
}

#[test]
#[should_panic(expected = "chunk size must be non-zero")]
fn chunky_zero_panics() {
    let _ = VecLender::new(vec![1, 2, 3]).chunky(0);
}

// ============================================================================
// Map adapter tests
// ============================================================================

#[test]
fn map_basic() {
    let mut mapped = VecLender::new(vec![1, 2, 3]).map(|x: &i32| *x * 2);

    assert_eq!(mapped.next(), Some(2));
    assert_eq!(mapped.next(), Some(4));
    assert_eq!(mapped.next(), Some(6));
    assert_eq!(mapped.next(), None);
}

#[test]
fn map_fold() {
    let sum = VecLender::new(vec![1, 2, 3])
        .map(|x: &i32| *x * 10)
        .fold(0, |acc, x| acc + x);
    assert_eq!(sum, 60);
}

#[test]
fn map_double_ended() {
    let mut mapped = VecLender::new(vec![1, 2, 3]).map(|x: &i32| *x * 10);

    assert_eq!(mapped.next_back(), Some(30));
    assert_eq!(mapped.next(), Some(10));
    assert_eq!(mapped.next_back(), Some(20));
    assert_eq!(mapped.next(), None);
}

#[test]
fn map_fold_additional() {
    let sum = VecLender::new(vec![1, 2, 3])
        .map(|x: &i32| *x * 2)
        .fold(0, |acc, x| acc + x);
    assert_eq!(sum, 12); // 2 + 4 + 6
}

#[test]
fn map_try_fold_additional() {
    let result: Option<i32> = VecLender::new(vec![1, 2, 3])
        .map(|x: &i32| *x * 2)
        .try_fold(0, |acc, x| Some(acc + x));
    assert_eq!(result, Some(12));
}

#[test]
fn map_rfold_additional() {
    let values: Vec<i32> =
        VecLender::new(vec![1, 2, 3])
            .map(|x: &i32| *x * 2)
            .rfold(Vec::new(), |mut acc, x| {
                acc.push(x);
                acc
            });
    assert_eq!(values, vec![6, 4, 2]);
}

#[test]
fn map_into_inner() {
    let map = VecLender::new(vec![1, 2, 3]).map(|x: &i32| *x * 2);
    let lender = map.into_inner();
    assert_eq!(lender.count(), 3);
}

#[test]
fn map_into_parts_additional() {
    let map = VecLender::new(vec![1, 2, 3]).map(|x: &i32| *x * 2);
    let (lender, _f) = map.into_parts();
    assert_eq!(lender.count(), 3);
}

#[test]
fn map_try_rfold_additional() {
    let result: Option<i32> = VecLender::new(vec![1, 2, 3])
        .map(|x: &i32| *x * 2)
        .try_rfold(0, |acc, x| Some(acc + x));
    assert_eq!(result, Some(12)); // 6 + 4 + 2 = 12
}

// ============================================================================
// Scan adapter tests
// Semantics: like fold but yields intermediate states
// Note: scan requires hrc_mut! macro for the closure
// ============================================================================

#[test]
fn scan_basic() {
    let mut scanned = VecLender::new(vec![1, 2, 3]).scan(
        0,
        hrc_mut!(for<'all> |args: (&'all mut i32, &i32)| -> Option<i32> {
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
        hrc_mut!(for<'all> |args: (&'all mut i32, &i32)| -> Option<i32> {
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
        hrc_mut!(for<'all> |args: (&'all mut i32, &i32)| -> Option<i32> {
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
        hrc_mut!(for<'all> |args: (&'all mut i32, &i32)| -> Option<i32> {
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
        hrc_mut!(for<'all> |args: (&'all mut i32, &i32)| -> Option<i32> { Some(*args.1) }),
    );
    // Scan can terminate early, so lower bound is 0
    let (lower, upper) = scan.size_hint();
    assert_eq!(lower, 0);
    assert_eq!(upper, Some(5));
}

// ============================================================================
// MapWhile adapter tests
// Semantics: like map but stops when function returns None
// Note: map_while requires hrc_mut! macro for the closure
// ============================================================================

#[test]
fn map_while_basic() {
    let mut mw = VecLender::new(vec![1, 2, 3, 4, 5]).map_while(hrc_mut!(
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
        VecLender::new(vec![1, 2, 3]).map_while(hrc_mut!(for<'all> |x: &i32| -> Option<i32> {
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
        VecLender::new(vec![5, 4, 3]).map_while(hrc_mut!(for<'all> |x: &i32| -> Option<i32> {
            if *x < 4 { Some(*x) } else { None }
        }));
    assert_eq!(mw.next(), None);
}

#[test]
fn map_while_into_inner() {
    let map_while =
        VecLender::new(vec![1, 2, 3]).map_while(hrc_mut!(for<'all> |x: &i32| -> Option<i32> {
            Some(*x * 2)
        }));
    let lender = map_while.into_inner();
    assert_eq!(lender.count(), 3);
}

#[test]
fn map_while_into_parts() {
    let map_while =
        VecLender::new(vec![1, 2, 3]).map_while(hrc_mut!(for<'all> |x: &i32| -> Option<i32> {
            Some(*x * 2)
        }));
    let (lender, _predicate) = map_while.into_parts();
    assert_eq!(lender.count(), 3);
}

#[test]
fn map_while_size_hint_additional() {
    let map_while = VecLender::new(vec![1, 2, 3, 4, 5])
        .map_while(hrc_mut!(for<'all> |x: &i32| -> Option<i32> { Some(*x) }));
    // MapWhile can terminate early, so lower bound is 0
    let (lower, upper) = map_while.size_hint();
    assert_eq!(lower, 0);
    assert_eq!(upper, Some(5));
}

#[test]
fn map_while_all_some() {
    // When all return Some, all elements are yielded
    let values: Vec<i32> = VecLender::new(vec![1, 2, 3])
        .map_while(hrc_mut!(for<'all> |x: &i32| -> Option<i32> {
            Some(*x * 10)
        }))
        .fold(Vec::new(), |mut acc, x| {
            acc.push(x);
            acc
        });
    assert_eq!(values, vec![10, 20, 30]);
}

// ============================================================================
// Flatten adapter tests
// ============================================================================

#[test]
fn flatten_basic() {
    let mut flattened = VecOfVecLender::new(vec![vec![1, 2], vec![3, 4], vec![5]]).flatten();

    assert_eq!(flattened.next(), Some(&1));
    assert_eq!(flattened.next(), Some(&2));
    assert_eq!(flattened.next(), Some(&3));
    assert_eq!(flattened.next(), Some(&4));
    assert_eq!(flattened.next(), Some(&5));
    assert_eq!(flattened.next(), None);
}

#[test]
fn flatten_empty_inner() {
    let mut flattened = VecOfVecLender::new(vec![vec![1], vec![], vec![2, 3]]).flatten();

    assert_eq!(flattened.next(), Some(&1));
    assert_eq!(flattened.next(), Some(&2));
    assert_eq!(flattened.next(), Some(&3));
    assert_eq!(flattened.next(), None);
}

#[test]
fn flatten_empty_outer() {
    let mut flattened = VecOfVecLender::new(vec![]).flatten();
    assert_eq!(flattened.next(), None);
}

// ============================================================================
// New method tests (M5–M7)
// ============================================================================

// M5: Zip nth_back — equal-length lenders
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

// M5: Zip nth_back — unequal-length lenders
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

// M5: Zip nth_back — empty
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

// M6: StepBy count
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

// M7: Chunk count
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

// M7: Chunk nth
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

// M7: Chunk try_fold
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

// M7: Chunk fold
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
    let sum: i32 = lender.copied().fold(0, |acc, x| acc + x);
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

// ============================================================================
// FlatMap adapter tests (infallible)
// ============================================================================

#[test]
fn flat_map_basic() {
    // flat_map: for each element n, produce n copies of n
    let mut l = [1, 2, 3]
        .into_iter()
        .into_lender()
        .flat_map(|n| (0..n).into_lender());
    assert_eq!(l.next(), Some(0)); // from n=1: [0]
    assert_eq!(l.next(), Some(0)); // from n=2: [0, 1]
    assert_eq!(l.next(), Some(1));
    assert_eq!(l.next(), Some(0)); // from n=3: [0, 1, 2]
    assert_eq!(l.next(), Some(1));
    assert_eq!(l.next(), Some(2));
    assert_eq!(l.next(), None);
}

#[test]
fn flat_map_empty_outer() {
    let mut l = std::iter::empty::<i32>()
        .into_lender()
        .flat_map(|n| (0..n).into_lender());
    assert_eq!(l.next(), None);
}

#[test]
fn flat_map_empty_inner() {
    // All inner lenders are empty
    let mut l = [0, 0, 0]
        .into_iter()
        .into_lender()
        .flat_map(|n| (0..n).into_lender());
    assert_eq!(l.next(), None);
}

#[test]
fn flat_map_mixed_empty_nonempty() {
    let mut l = [1, 0, 2]
        .into_iter()
        .into_lender()
        .flat_map(|n| (0..n).into_lender());
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
fn flatten_fold() {
    let lender = VecOfVecLender::new(vec![vec![1, 2], vec![3], vec![4, 5]]);
    let result = lender.flatten().fold(0, |acc, x| acc + x);
    assert_eq!(result, 15); // 1+2+3+4+5
}

#[test]
fn flatten_count() {
    let lender = VecOfVecLender::new(vec![vec![1, 2], vec![], vec![3, 4, 5]]);
    assert_eq!(lender.flatten().count(), 5);
}

#[test]
fn flatten_try_fold() {
    let lender = VecOfVecLender::new(vec![vec![1, 2], vec![3, 4], vec![5]]);
    let result: Result<i32, i32> = lender.flatten().try_fold(0, |acc, x| {
        let new = acc + x;
        if new > 6 { Err(new) } else { Ok(new) }
    });
    assert_eq!(result, Err(10)); // 1+2+3+4 = 10 > 6
}

#[test]
fn flatten_fold_empty() {
    let lender = VecOfVecLender::new(vec![]);
    let result = lender.flatten().fold(0, |acc, x: &i32| acc + x);
    assert_eq!(result, 0);
}

#[test]
fn flatten_count_empty() {
    let lender = VecOfVecLender::new(vec![]);
    assert_eq!(lender.flatten().count(), 0);
}

// ============================================================================
// T4: Multi-adapter composition tests (infallible)
// ============================================================================

#[test]
fn compose_filter_map_fold() {
    // filter even numbers, map to doubled, then fold sum
    let result = VecLender::new(vec![1, 2, 3, 4, 5, 6])
        .filter(|x| *x % 2 == 0)
        .map(hrc_mut!(for<'all> |x: &'all i32| -> i32 { *x * 10 }))
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
        .map(hrc_mut!(for<'all> |x: &'all i32| -> i32 { *x * 100 }))
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
        .map(hrc_mut!(for<'all> |pair: (&'all i32, &'all i32)| -> i32 {
            *pair.0 + *pair.1
        }))
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
