//! Tests for FallibleLender trait methods, DoubleEnded, peekable unsafe paths

mod common;
use ::lender::prelude::*;
use common::*;

// Comprehensive FallibleLender tests
// ============================================================================

#[test]
fn fallible_lender_next_chunk() {
    use lender::FallibleLender;

    let mut fallible: lender::IntoFallible<_> =
        VecLender::new(vec![1, 2, 3, 4, 5]).into_fallible();
    let mut chunk = fallible.next_chunk(2);
    assert_eq!(chunk.next(), Ok(Some(&1)));
    assert_eq!(chunk.next(), Ok(Some(&2)));
    assert_eq!(chunk.next(), Ok(None));
}

#[test]
fn fallible_lender_count() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<_> = VecLender::new(vec![1, 2, 3, 4, 5]).into_fallible();
    assert_eq!(fallible.count(), Ok(5));
}

#[test]
fn fallible_lender_last() {
    use lender::FallibleLender;

    let mut fallible: lender::IntoFallible<_> = VecLender::new(vec![1, 2, 3]).into_fallible();
    assert_eq!(fallible.last(), Ok(Some(&3)));
}

#[test]
fn fallible_lender_advance_by() {
    use lender::FallibleLender;

    let mut fallible: lender::IntoFallible<_> =
        VecLender::new(vec![1, 2, 3, 4, 5]).into_fallible();
    assert_eq!(fallible.advance_by(2), Ok(Ok(())));
    assert_eq!(fallible.next(), Ok(Some(&3)));
}

#[test]
fn fallible_lender_nth() {
    use lender::FallibleLender;

    let mut fallible: lender::IntoFallible<_> =
        VecLender::new(vec![1, 2, 3, 4, 5]).into_fallible();
    assert_eq!(fallible.nth(2), Ok(Some(&3)));
}

#[test]
fn fallible_lender_step_by() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<_> = VecLender::new(vec![1, 2, 3, 4, 5]).into_fallible();
    let mut stepped = fallible.step_by(2);
    assert_eq!(stepped.next(), Ok(Some(&1)));
    assert_eq!(stepped.next(), Ok(Some(&3)));
    assert_eq!(stepped.next(), Ok(Some(&5)));
    assert_eq!(stepped.next(), Ok(None));
}

#[test]
fn fallible_lender_chain() {
    use lender::FallibleLender;

    let fallible1: lender::IntoFallible<_> = VecLender::new(vec![1, 2]).into_fallible();
    let fallible2: lender::IntoFallible<_> = VecLender::new(vec![3, 4]).into_fallible();
    let mut chained = fallible1.chain(fallible2);
    assert_eq!(chained.next(), Ok(Some(&1)));
    assert_eq!(chained.next(), Ok(Some(&2)));
    assert_eq!(chained.next(), Ok(Some(&3)));
    assert_eq!(chained.next(), Ok(Some(&4)));
    assert_eq!(chained.next(), Ok(None));
}

#[test]
fn fallible_lender_zip() {
    use lender::FallibleLender;

    let fallible1: lender::IntoFallible<_> = VecLender::new(vec![1, 2, 3]).into_fallible();
    let fallible2: lender::IntoFallible<_> = VecLender::new(vec![4, 5, 6]).into_fallible();
    let mut zipped = fallible1.zip(fallible2);
    assert_eq!(zipped.next(), Ok(Some((&1, &4))));
    assert_eq!(zipped.next(), Ok(Some((&2, &5))));
    assert_eq!(zipped.next(), Ok(Some((&3, &6))));
    assert_eq!(zipped.next(), Ok(None));
}

#[test]
fn fallible_lender_map() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<_> = VecLender::new(vec![1, 2, 3]).into_fallible();
    let mut mapped = fallible.map(|x: &i32| Ok(*x * 2));
    assert_eq!(mapped.next(), Ok(Some(2)));
    assert_eq!(mapped.next(), Ok(Some(4)));
    assert_eq!(mapped.next(), Ok(Some(6)));
    assert_eq!(mapped.next(), Ok(None));
}

#[test]
fn fallible_lender_filter() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<_> =
        VecLender::new(vec![1, 2, 3, 4, 5, 6]).into_fallible();
    let mut filtered = fallible.filter(|&&x| Ok(x % 2 == 0));
    assert_eq!(filtered.next(), Ok(Some(&2)));
    assert_eq!(filtered.next(), Ok(Some(&4)));
    assert_eq!(filtered.next(), Ok(Some(&6)));
    assert_eq!(filtered.next(), Ok(None));
}

#[test]
fn fallible_lender_enumerate() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<_> = VecLender::new(vec![10, 20, 30]).into_fallible();
    let mut enumerated = fallible.enumerate();
    assert_eq!(enumerated.next(), Ok(Some((0, &10))));
    assert_eq!(enumerated.next(), Ok(Some((1, &20))));
    assert_eq!(enumerated.next(), Ok(Some((2, &30))));
    assert_eq!(enumerated.next(), Ok(None));
}

#[test]
fn fallible_lender_skip() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<_> = VecLender::new(vec![1, 2, 3, 4, 5]).into_fallible();
    let mut skipped = fallible.skip(2);
    assert_eq!(skipped.next(), Ok(Some(&3)));
    assert_eq!(skipped.next(), Ok(Some(&4)));
    assert_eq!(skipped.next(), Ok(Some(&5)));
    assert_eq!(skipped.next(), Ok(None));
}

#[test]
fn fallible_lender_take() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<_> = VecLender::new(vec![1, 2, 3, 4, 5]).into_fallible();
    let mut taken = fallible.take(3);
    assert_eq!(taken.next(), Ok(Some(&1)));
    assert_eq!(taken.next(), Ok(Some(&2)));
    assert_eq!(taken.next(), Ok(Some(&3)));
    assert_eq!(taken.next(), Ok(None));
}

#[test]
fn fallible_lender_skip_while() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<_> = VecLender::new(vec![1, 2, 3, 4, 5]).into_fallible();
    let mut skipped = fallible.skip_while(|&&x| Ok(x < 3));
    assert_eq!(skipped.next(), Ok(Some(&3)));
    assert_eq!(skipped.next(), Ok(Some(&4)));
    assert_eq!(skipped.next(), Ok(Some(&5)));
    assert_eq!(skipped.next(), Ok(None));
}

#[test]
fn fallible_lender_take_while() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<_> = VecLender::new(vec![1, 2, 3, 4, 5]).into_fallible();
    let mut taken = fallible.take_while(|&&x| Ok(x < 4));
    assert_eq!(taken.next(), Ok(Some(&1)));
    assert_eq!(taken.next(), Ok(Some(&2)));
    assert_eq!(taken.next(), Ok(Some(&3)));
    assert_eq!(taken.next(), Ok(None));
}

#[test]
fn fallible_lender_inspect() {
    use lender::FallibleLender;

    let mut inspected = Vec::new();
    let fallible: lender::IntoFallible<_> = VecLender::new(vec![1, 2, 3]).into_fallible();
    let mut lender = fallible.inspect(|&&x| {
        inspected.push(x);
        Ok(())
    });
    assert_eq!(lender.next(), Ok(Some(&1)));
    assert_eq!(lender.next(), Ok(Some(&2)));
    assert_eq!(lender.next(), Ok(Some(&3)));
    assert_eq!(inspected, vec![1, 2, 3]);
}

#[test]
fn fallible_lender_fuse() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<_> = VecLender::new(vec![1, 2]).into_fallible();
    let mut fused = fallible.fuse();
    assert_eq!(fused.next(), Ok(Some(&1)));
    assert_eq!(fused.next(), Ok(Some(&2)));
    assert_eq!(fused.next(), Ok(None));
    assert_eq!(fused.next(), Ok(None)); // Fused stays None
}

#[test]
fn fallible_lender_fold() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<_> = VecLender::new(vec![1, 2, 3, 4, 5]).into_fallible();
    let sum = fallible.fold(0, |acc, x| Ok(acc + *x));
    assert_eq!(sum, Ok(15));
}

#[test]
fn fallible_lender_for_each() {
    use lender::FallibleLender;

    let mut collected = Vec::new();
    let fallible: lender::IntoFallible<_> = VecLender::new(vec![1, 2, 3]).into_fallible();
    let result = fallible.for_each(|x| {
        collected.push(*x);
        Ok(())
    });
    assert_eq!(result, Ok(()));
    assert_eq!(collected, vec![1, 2, 3]);
}

#[test]
fn fallible_lender_all() {
    use lender::FallibleLender;

    let mut fallible: lender::IntoFallible<_> = VecLender::new(vec![2, 4, 6]).into_fallible();
    assert_eq!(fallible.all(|x| Ok(*x % 2 == 0)), Ok(true));

    let mut fallible2: lender::IntoFallible<_> = VecLender::new(vec![2, 3, 6]).into_fallible();
    assert_eq!(fallible2.all(|x| Ok(*x % 2 == 0)), Ok(false));
}

#[test]
fn fallible_lender_any() {
    use lender::FallibleLender;

    let mut fallible: lender::IntoFallible<_> = VecLender::new(vec![1, 3, 5]).into_fallible();
    assert_eq!(fallible.any(|x| Ok(*x % 2 == 0)), Ok(false));

    let mut fallible2: lender::IntoFallible<_> = VecLender::new(vec![1, 2, 3]).into_fallible();
    assert_eq!(fallible2.any(|x| Ok(*x % 2 == 0)), Ok(true));
}

#[test]
fn fallible_lender_find() {
    use lender::FallibleLender;

    let mut fallible: lender::IntoFallible<_> =
        VecLender::new(vec![1, 2, 3, 4, 5]).into_fallible();
    assert_eq!(fallible.find(|&&x| Ok(x > 3)), Ok(Some(&4)));
}

#[test]
fn fallible_lender_position() {
    use lender::FallibleLender;

    let mut fallible: lender::IntoFallible<_> =
        VecLender::new(vec![1, 2, 3, 4, 5]).into_fallible();
    assert_eq!(fallible.position(|x| Ok(*x == 3)), Ok(Some(2)));
}

#[test]
fn fallible_lender_rposition() {
    use lender::FallibleLender;

    let mut fallible: lender::IntoFallible<_> =
        VecLender::new(vec![1, 2, 3, 4, 5]).into_fallible();
    assert_eq!(fallible.rposition(|x| Ok(*x == 3)), Ok(Some(2)));
}

#[test]
fn lender_convert() {
    use lender::FallibleLender;

    let data = vec![Ok(1), Ok(2), Err("oops")];
    let mut lender = data.into_iter().into_lender().convert::<&str>();
    assert_eq!(lender.next(), Ok(Some(1)));
    assert_eq!(lender.next(), Ok(Some(2)));
    assert!(lender.next().is_err());
}

#[test]
fn fallible_lender_chunky() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<_> =
        VecLender::new(vec![1, 2, 3, 4, 5, 6]).into_fallible();
    let mut chunky = fallible.chunky(2);

    let mut chunk1 = chunky.next().unwrap().unwrap();
    assert_eq!(chunk1.next(), Ok(Some(&1)));
    assert_eq!(chunk1.next(), Ok(Some(&2)));
    assert_eq!(chunk1.next(), Ok(None));
}

#[test]
fn fallible_lender_rev() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<_> = VecLender::new(vec![1, 2, 3]).into_fallible();
    let mut rev = fallible.rev();
    assert_eq!(rev.next(), Ok(Some(&3)));
    assert_eq!(rev.next(), Ok(Some(&2)));
    assert_eq!(rev.next(), Ok(Some(&1)));
    assert_eq!(rev.next(), Ok(None));
}

// ============================================================================
// DoubleEndedFallibleLender tests
// ============================================================================

#[test]
fn double_ended_fallible_advance_back_by() {
    use lender::DoubleEndedFallibleLender;

    let mut fallible: lender::IntoFallible<_> =
        VecLender::new(vec![1, 2, 3, 4, 5]).into_fallible();
    assert_eq!(fallible.advance_back_by(2), Ok(Ok(())));
    assert_eq!(fallible.next_back(), Ok(Some(&3)));
}

#[test]
fn double_ended_fallible_nth_back() {
    use lender::DoubleEndedFallibleLender;

    let mut fallible: lender::IntoFallible<_> =
        VecLender::new(vec![1, 2, 3, 4, 5]).into_fallible();
    assert_eq!(fallible.nth_back(2), Ok(Some(&3)));
}

#[test]
fn double_ended_fallible_try_rfold() {
    use lender::DoubleEndedFallibleLender;

    let mut fallible: lender::IntoFallible<_> = VecLender::new(vec![1, 2, 3]).into_fallible();
    let result: Result<Option<i32>, core::convert::Infallible> = fallible.try_rfold(0, |acc, x| Ok(Some(acc + *x)));
    assert_eq!(result, Ok(Some(6)));
}

#[test]
fn double_ended_fallible_rfold() {
    use lender::DoubleEndedFallibleLender;

    let fallible: lender::IntoFallible<_> = VecLender::new(vec![1, 2, 3]).into_fallible();
    let values: Result<Vec<i32>, core::convert::Infallible> = fallible.rfold(Vec::new(), |mut acc, x| {
        acc.push(*x);
        Ok(acc)
    });
    assert_eq!(values, Ok(vec![3, 2, 1]));
}

// ============================================================================
// Fallible peekable unsafe paths
// ============================================================================

#[test]
fn fallible_peekable_nth_zero_with_peeked() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<_> = VecLender::new(vec![1, 2, 3]).into_fallible();
    let mut peekable = fallible.peekable();
    // Peek to store a value
    assert_eq!(peekable.peek(), Ok(Some(&&1)));
    // nth(0) should return the peeked value through the unsafe transmute path
    assert_eq!(peekable.nth(0), Ok(Some(&1)));
    assert_eq!(peekable.next(), Ok(Some(&2)));
}

// FalliblePeekable::last with peeked value
#[test]
fn fallible_peekable_last_with_peeked_only() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<_> = VecLender::new(vec![1]).into_fallible();
    let mut peekable = fallible.peekable();
    // Peek the only value
    assert_eq!(peekable.peek(), Ok(Some(&&1)));
    // last() should return the peeked value through the unsafe transmute path
    assert_eq!(peekable.last(), Ok(Some(&1)));
}

// FalliblePeekable::next_back with peeked value when underlying lender is empty
#[test]
fn fallible_peekable_next_back_with_peeked_exhausted() {
    use lender::DoubleEndedFallibleLender;

    let fallible: lender::IntoFallible<_> = VecLender::new(vec![1]).into_fallible();
    let mut peekable = fallible.peekable();
    // Peek the only value
    let _ = peekable.peek();
    // next_back should return the peeked value through the unsafe transmute path
    assert_eq!(peekable.next_back(), Ok(Some(&1)));
}

// FalliblePeekable::peek_mut (covers unsafe transmute in peek_mut)
#[test]
fn fallible_peekable_peek_mut() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<_> = VecLender::new(vec![1, 2, 3]).into_fallible();
    let mut peekable = fallible.peekable();
    // peek_mut to store a value and get mutable reference
    let peeked = peekable.peek_mut().unwrap();
    assert_eq!(peeked, Some(&mut &1));
}

// FalliblePeekable::next_if (covers unsafe transmute in next_if)
#[test]
fn fallible_peekable_next_if_match() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<_> = VecLender::new(vec![1, 2, 3]).into_fallible();
    let mut peekable = fallible.peekable();
    // next_if should return Some when predicate matches
    assert_eq!(peekable.next_if(|&&x| x == 1), Ok(Some(&1)));
    // Should have advanced
    assert_eq!(peekable.next(), Ok(Some(&2)));
}

#[test]
fn fallible_peekable_next_if_no_match() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<_> = VecLender::new(vec![1, 2, 3]).into_fallible();
    let mut peekable = fallible.peekable();
    // next_if should return None when predicate doesn't match (and store in peeked)
    assert_eq!(peekable.next_if(|&&x| x == 5), Ok(None));
    // Value should still be available
    assert_eq!(peekable.next(), Ok(Some(&1)));
}

#[test]
fn fallible_peekable_rfold_with_peeked() {
    use lender::DoubleEndedFallibleLender;
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<_> = VecLender::new(vec![1, 2, 3]).into_fallible();
    let mut peekable = fallible.peekable();
    assert_eq!(peekable.peek(), Ok(Some(&&1)));
    // rfold processes back-to-front: 3, 2, then peeked 1
    let result = peekable.rfold(Vec::new(), |mut acc, &x| {
        acc.push(x);
        Ok(acc)
    });
    assert_eq!(result, Ok(vec![3, 2, 1]));
}

#[test]
fn fallible_peekable_try_rfold_with_peeked_complete() {
    use lender::DoubleEndedFallibleLender;
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<_> = VecLender::new(vec![1, 2, 3]).into_fallible();
    let mut peekable = fallible.peekable();
    assert_eq!(peekable.peek(), Ok(Some(&&1)));
    // try_rfold processes back-to-front: 3, 2, then peeked 1
    let result: Result<Option<Vec<i32>>, core::convert::Infallible> = peekable.try_rfold(Vec::new(), |mut acc, &x| {
        acc.push(x);
        Ok(Some(acc))
    });
    assert_eq!(result, Ok(Some(vec![3, 2, 1])));
}

// Covers the ControlFlow::Break path in fallible Peekable::try_rfold
// where the peeked value is stored back.
#[test]
fn fallible_peekable_try_rfold_with_peeked_break() {
    use lender::DoubleEndedFallibleLender;
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<_> = VecLender::new(vec![1, 2, 3]).into_fallible();
    let mut peekable = fallible.peekable();
    assert_eq!(peekable.peek(), Ok(Some(&&1)));
    // Inner lender has [2, 3]. try_rfold processes back-to-front:
    // 3 (continue, acc=3), then 2 (break via None).
    let result: Result<Option<i32>, core::convert::Infallible> =
        peekable.try_rfold(
            0,
            |acc, &x| {
                if x == 2 { Ok(None) } else { Ok(Some(acc + x)) }
            },
        );
    assert_eq!(result, Ok(None));
    // The peeked value should have been stored back
    assert_eq!(peekable.next(), Ok(Some(&1)));
    // Inner lender was fully consumed by try_rfold
    assert_eq!(peekable.next(), Ok(None));
}

// ============================================================================
// Iter fallible iterator
// ============================================================================

// Iter adapter FallibleIterator next (covers unsafe transmute in next)
// Note: .iter() requires the Lend type to satisfy complex higher-ranked trait bounds.
// With VecFallibleLender yielding &'lend i32, there are lifetime issues that prevent
// it from working with .iter(). We test with owned values via into_iter().into_lender().into_fallible()
// which yields i32 (Copy type with no lifetime issues).
#[test]
fn iter_fallible_iterator_next() {
    use fallible_iterator::FallibleIterator;

    let fallible = vec![1, 2, 3]
        .into_iter()
        .into_lender()
        .into_fallible();
    let mut iter = fallible.iter();
    assert_eq!(FallibleIterator::next(&mut iter), Ok(Some(1)));
    assert_eq!(FallibleIterator::next(&mut iter), Ok(Some(2)));
    assert_eq!(FallibleIterator::next(&mut iter), Ok(Some(3)));
    assert_eq!(FallibleIterator::next(&mut iter), Ok(None));
}

// Iter adapter DoubleEndedFallibleIterator next_back (covers unsafe transmute in next_back)
#[test]
fn iter_double_ended_fallible_iterator_next_back() {
    use fallible_iterator::DoubleEndedFallibleIterator;

    let fallible = vec![1, 2, 3]
        .into_iter()
        .into_lender()
        .into_fallible();
    let mut iter = fallible.iter();
    assert_eq!(
        DoubleEndedFallibleIterator::next_back(&mut iter),
        Ok(Some(3))
    );
    assert_eq!(
        DoubleEndedFallibleIterator::next_back(&mut iter),
        Ok(Some(2))
    );
}

// ============================================================================
// Cycle fallible coverage
// ============================================================================

// Cycle fallible next (covers unsafe reborrow in next)
#[test]
fn cycle_fallible_next_coverage() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<_> = VecLender::new(vec![1, 2]).into_fallible();
    let mut cycle = fallible.cycle();
    // Call next() multiple times to exercise the unsafe reborrow and cycling
    assert_eq!(cycle.next(), Ok(Some(&1)));
    assert_eq!(cycle.next(), Ok(Some(&2)));
    // This should cycle back to the beginning
    assert_eq!(cycle.next(), Ok(Some(&1)));
    assert_eq!(cycle.next(), Ok(Some(&2)));
    assert_eq!(cycle.next(), Ok(Some(&1)));
}

// ============================================================================
// Fallible nth past end
// ============================================================================

#[test]
fn fallible_lender_nth_past_end() {
    use core::num::NonZeroUsize;
    use lender::{FallibleLend, FallibleLender, FallibleLending, FusedFallibleLender};

    /// A fallible lender that always has elements but whose advance_by
    /// always reports failure without consuming anything.
    struct StubbyAdvance(i32);

    impl<'lend> FallibleLending<'lend> for StubbyAdvance {
        type Lend = i32;
    }

    impl FallibleLender for StubbyAdvance {
        type Error = ();
        lender::check_covariance_fallible!();

        fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
            self.0 += 1;
            Ok(Some(self.0))
        }

        fn advance_by(&mut self, n: usize) -> Result<Result<(), NonZeroUsize>, Self::Error> {
            // Report complete failure: didn't advance at all.
            Ok(NonZeroUsize::new(n).map_or(Ok(()), Err))
        }
    }

    impl FusedFallibleLender for StubbyAdvance {}

    let mut lender = StubbyAdvance(0);
    // advance_by(3) will return Ok(Err(3)) — full failure.
    // nth(3) must therefore return Ok(None), not call next().
    assert_eq!(lender.nth(3), Ok(None));
}

#[test]
fn double_ended_fallible_nth_back_past_end() {
    use core::num::NonZeroUsize;
    use lender::{
        DoubleEndedFallibleLender, FallibleLend, FallibleLender, FallibleLending,
        FusedFallibleLender,
    };

    /// A fallible lender whose advance_back_by always reports failure.
    struct StubbyAdvanceBack(i32);

    impl<'lend> FallibleLending<'lend> for StubbyAdvanceBack {
        type Lend = i32;
    }

    impl FallibleLender for StubbyAdvanceBack {
        type Error = ();
        lender::check_covariance_fallible!();

        fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
            self.0 += 1;
            Ok(Some(self.0))
        }
    }

    impl DoubleEndedFallibleLender for StubbyAdvanceBack {
        fn next_back(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
            self.0 += 1;
            Ok(Some(self.0))
        }

        fn advance_back_by(&mut self, n: usize) -> Result<Result<(), NonZeroUsize>, Self::Error> {
            // Report complete failure: didn't advance at all.
            Ok(NonZeroUsize::new(n).map_or(Ok(()), Err))
        }
    }

    impl FusedFallibleLender for StubbyAdvanceBack {}

    let mut lender = StubbyAdvanceBack(0);
    // advance_back_by(3) will return Ok(Err(3)) — full failure.
    // nth_back(3) must therefore return Ok(None), not call next_back().
    assert_eq!(lender.nth_back(3), Ok(None));
}

