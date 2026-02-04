//! Tests for fallible adapters: basic adapters, trait adapters, max_by/min_by, into_fallible

mod common;
use ::lender::prelude::*;
use common::*;

// ============================================================================
// Fallible adapters
// ============================================================================

#[test]
fn into_fallible_adapter() {
    use lender::prelude::*;

    // Test converting a normal lender to fallible
    let data = vec![1, 2, 3];
    let mut fallible = data.into_iter().into_lender().into_fallible();
    assert_eq!(fallible.next().unwrap(), Some(1));
    assert_eq!(fallible.next().unwrap(), Some(2));
    assert_eq!(fallible.next().unwrap(), Some(3));
    assert!(fallible.next().unwrap().is_none());

    // Test with fold
    let data2 = vec![10, 20, 30];
    let sum: Result<i32, core::convert::Infallible> = data2
        .into_iter()
        .into_lender()
        .into_fallible()
        .fold(0, |acc, x| Ok(acc + x));
    assert_eq!(sum, Ok(60));
}

#[test]
fn map_err_adapter() {
    use lender::{fallible_lend, fallible_once, fallible_once_err};

    // Test mapping error type
    let mut mapped =
        fallible_once_err::<fallible_lend!(u32), _>(42).map_err(|e: i32| format!("Error: {}", e));
    match mapped.next() {
        Err(e) => assert_eq!(e, "Error: 42"),
        Ok(_) => panic!("Expected error"),
    }

    // Test with value (error mapper shouldn't be called)
    let mut mapped_ok = fallible_once::<fallible_lend!(i32), String>(100)
        .map_err(|_e: String| panic!("Should not be called"));
    assert_eq!(mapped_ok.next().unwrap(), Some(100));
}

#[test]
fn fallible_peekable_adapter() {
    use lender::{FalliblePeekable, from_fallible_fn};

    // Test peeking functionality
    let mut peekable: FalliblePeekable<_> =
        from_fallible_fn(0, |state: &mut i32| -> Result<Option<i32>, String> {
            *state += 1;
            if *state <= 3 {
                Ok(Some(*state))
            } else {
                Ok(None)
            }
        })
        .peekable();

    // Peek multiple times - should see same value
    assert_eq!(peekable.peek().unwrap(), Some(&1));
    assert_eq!(peekable.peek().unwrap(), Some(&1));

    // Next consumes the value
    assert_eq!(peekable.next().unwrap(), Some(1));

    // Now peek sees next value
    assert_eq!(peekable.peek().unwrap(), Some(&2));
    assert_eq!(peekable.next().unwrap(), Some(2));

    // Test peek_mut
    if let Some(val) = peekable.peek_mut().unwrap() {
        *val = 100;
    }
    assert_eq!(peekable.next().unwrap(), Some(100));

    // Peek at end
    assert!(peekable.peek().unwrap().is_none());
    assert!(peekable.next().unwrap().is_none());
}

#[test]
fn intersperse_adapters() {
    use lender::from_fallible_fn;

    // Test intersperse with fixed separator
    let interspersed = from_fallible_fn(0, |state: &mut i32| -> Result<Option<i32>, String> {
        *state += 1;
        if *state <= 3 {
            Ok(Some(*state))
        } else {
            Ok(None)
        }
    })
    .intersperse(0);

    let mut collected = Vec::new();
    interspersed
        .for_each(|x| {
            collected.push(x);
            Ok(())
        })
        .unwrap();
    assert_eq!(collected, vec![1, 0, 2, 0, 3]);

    // Test intersperse_with using a closure
    let mut counter = 10;
    let interspersed_with = from_fallible_fn(0, |state: &mut i32| -> Result<Option<i32>, String> {
        *state += 1;
        if *state <= 3 {
            Ok(Some(*state))
        } else {
            Ok(None)
        }
    })
    .intersperse_with(move || {
        counter += 1;
        Ok(counter)
    });

    let mut collected_with = Vec::new();
    interspersed_with
        .for_each(|x| {
            collected_with.push(x);
            Ok(())
        })
        .unwrap();
    assert_eq!(collected_with, vec![1, 11, 2, 12, 3]);
}

#[test]
fn map_adapters() {
    let data = vec![1, 2, 3];

    let mut iter = data
        .into_iter()
        .into_lender()
        .into_fallible()
        .map(hrc_mut!(for<'lend> |x: i32| -> Result<
            i32,
            std::convert::Infallible,
        > { Ok(x * 2) }));

    assert_eq!(iter.next().unwrap(), Some(2));
    assert_eq!(iter.next().unwrap(), Some(4));
    assert_eq!(iter.next().unwrap(), Some(6));
    assert_eq!(iter.next().unwrap(), None);
}

struct Wrapper(Vec<i32>);
impl<'lend> FallibleLending<'lend> for Wrapper {
    type Lend = i32;
}
impl FallibleLender for Wrapper {
    type Error = std::convert::Infallible;
    lender::check_covariance_fallible!();
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        if self.0.is_empty() {
            Ok(None)
        } else {
            Ok(Some(self.0.remove(0)))
        }
    }
}

#[test]
fn flatten_adapters() {
    let data = vec![
        Wrapper(vec![1, 2, 3]),
        Wrapper(vec![1, 2, 3]),
        Wrapper(vec![1, 2, 3]),
    ];

    let mut iter = data.into_iter().into_lender().into_fallible().flatten();

    assert_eq!(iter.next().unwrap(), Some(1));
    assert_eq!(iter.next().unwrap(), Some(2));
    assert_eq!(iter.next().unwrap(), Some(3));
    assert_eq!(iter.next().unwrap(), Some(1));
    assert_eq!(iter.next().unwrap(), Some(2));
    assert_eq!(iter.next().unwrap(), Some(3));
    assert_eq!(iter.next().unwrap(), Some(1));
    assert_eq!(iter.next().unwrap(), Some(2));
    assert_eq!(iter.next().unwrap(), Some(3));
}

#[test]
fn flat_map_adapters() {
    let data = vec![1, 2, 3];

    let mut iter = data
        .into_iter()
        .into_lender()
        .into_fallible()
        .flat_map(hrc_mut!(for<'lend> |x: i32| -> Result<
            Wrapper,
            std::convert::Infallible,
        > { Ok(Wrapper(vec![x; 2])) }));

    assert_eq!(iter.next().unwrap(), Some(1));
    assert_eq!(iter.next().unwrap(), Some(1));
    assert_eq!(iter.next().unwrap(), Some(2));
    assert_eq!(iter.next().unwrap(), Some(2));
    assert_eq!(iter.next().unwrap(), Some(3));
    assert_eq!(iter.next().unwrap(), Some(3));
}

#[test]
fn fallible_flatten_fold() {
    let data = vec![Wrapper(vec![1, 2]), Wrapper(vec![3]), Wrapper(vec![4, 5])];
    let iter = data.into_iter().into_lender().into_fallible().flatten();
    let result = iter.fold(0, |acc, x| Ok(acc + x)).unwrap();
    assert_eq!(result, 15);
}

#[test]
fn fallible_flatten_fold_empty() {
    let data: Vec<Wrapper> = vec![];
    let iter = data.into_iter().into_lender().into_fallible().flatten();
    let result = iter.fold(0, |acc, x: i32| Ok(acc + x)).unwrap();
    assert_eq!(result, 0);
}

#[test]
fn fallible_flatten_count() {
    let data = vec![Wrapper(vec![1, 2]), Wrapper(vec![]), Wrapper(vec![3, 4, 5])];
    let iter = data.into_iter().into_lender().into_fallible().flatten();
    assert_eq!(iter.count().unwrap(), 5);
}

#[test]
fn fallible_flatten_count_empty() {
    let data: Vec<Wrapper> = vec![];
    let iter = data.into_iter().into_lender().into_fallible().flatten();
    assert_eq!(iter.count().unwrap(), 0);
}

#[test]
fn fallible_flatten_try_fold() {
    let data = vec![Wrapper(vec![1, 2]), Wrapper(vec![3, 4]), Wrapper(vec![5])];
    let mut iter = data.into_iter().into_lender().into_fallible().flatten();
    let result: Result<i32, i32> = iter
        .try_fold(0, |acc, x| {
            let new = acc + x;
            if new > 6 { Ok(Err(new)) } else { Ok(Ok(new)) }
        })
        .unwrap();
    assert_eq!(result, Err(10));
}

#[test]
fn fallible_flat_map_fold() {
    let data = vec![1, 2, 3];
    let iter = data
        .into_iter()
        .into_lender()
        .into_fallible()
        .flat_map(hrc_mut!(for<'lend> |x: i32| -> Result<
            Wrapper,
            std::convert::Infallible,
        > { Ok(Wrapper(vec![x; 2])) }));
    let result = iter.fold(0, |acc, x| Ok(acc + x)).unwrap();
    assert_eq!(result, 12); // (1+1) + (2+2) + (3+3) = 12
}

#[test]
fn fallible_flat_map_count() {
    let data = vec![1, 2, 3];
    let iter = data
        .into_iter()
        .into_lender()
        .into_fallible()
        .flat_map(hrc_mut!(for<'lend> |x: i32| -> Result<
            Wrapper,
            std::convert::Infallible,
        > { Ok(Wrapper(vec![x; 2])) }));
    assert_eq!(iter.count().unwrap(), 6);
}

// ============================================================================
// ExactSize/DoubleEnded/Fused fallible basics
// ============================================================================

#[test]
fn exact_size_fallible_lender_basic() {
    use lender::ExactSizeFallibleLender;

    let mut lender = VecFallibleLender::new(vec![1, 2, 3, 4, 5]);
    assert_eq!(lender.len(), 5);
    assert!(!lender.is_empty());

    lender.next().unwrap();
    assert_eq!(lender.len(), 4);

    lender.next().unwrap();
    lender.next().unwrap();
    lender.next().unwrap();
    lender.next().unwrap();
    assert_eq!(lender.len(), 0);
    assert!(lender.is_empty());
}

#[test]
fn double_ended_fallible_lender_basic() {
    use lender::DoubleEndedFallibleLender;

    let mut lender = VecFallibleLender::new(vec![1, 2, 3, 4, 5]);

    // Front and back iteration
    assert_eq!(lender.next().unwrap(), Some(&1));
    assert_eq!(lender.next_back().unwrap(), Some(&5));
    assert_eq!(lender.next().unwrap(), Some(&2));
    assert_eq!(lender.next_back().unwrap(), Some(&4));
    assert_eq!(lender.next().unwrap(), Some(&3));
    assert_eq!(lender.next().unwrap(), None);
    assert_eq!(lender.next_back().unwrap(), None);
}

#[test]
fn fused_fallible_lender_basic() {
    use lender::FusedFallibleLender;

    fn assert_fused<L: FusedFallibleLender>(_: &L) {}

    let lender = VecFallibleLender::new(vec![1, 2, 3]);
    assert_fused(&lender);

    // Test fused behavior - should continue returning None after exhaustion
    let mut lender = VecFallibleLender::new(vec![1]);
    assert_eq!(lender.next().unwrap(), Some(&1));
    assert_eq!(lender.next().unwrap(), None);
    assert_eq!(lender.next().unwrap(), None);
    assert_eq!(lender.next().unwrap(), None);
}

// ============================================================================
// Fallible trait adapter tests
// ============================================================================

#[test]
fn fallible_trait_adapters_map() {
    use lender::{ExactSizeFallibleLender, FusedFallibleLender};

    fn assert_exact_size<L: ExactSizeFallibleLender>(_: &L) {}
    fn assert_fused<L: FusedFallibleLender>(_: &L) {}

    let lender = VecFallibleLender::new(vec![1, 2, 3]);
    let mapped = lender.map(hrc_mut!(for<'lend> |x: &'lend i32| -> Result<
        i32,
        std::convert::Infallible,
    > { Ok(*x * 2) }));

    assert_exact_size(&mapped);
    assert_fused(&mapped);
}

#[test]
fn fallible_trait_adapters_filter() {
    use lender::FusedFallibleLender;

    fn assert_fused<L: FusedFallibleLender>(_: &L) {}

    let lender = VecFallibleLender::new(vec![1, 2, 3, 4, 5]);
    let filtered = lender.filter(|&&x| Ok(x > 2));

    assert_fused(&filtered);
}

#[test]
fn fallible_trait_adapters_enumerate() {
    use lender::{DoubleEndedFallibleLender, ExactSizeFallibleLender, FusedFallibleLender};

    fn assert_exact_size<L: ExactSizeFallibleLender>(_: &L) {}
    fn assert_fused<L: FusedFallibleLender>(_: &L) {}
    fn assert_double_ended<L: DoubleEndedFallibleLender>(_: &L) {}

    let lender = VecFallibleLender::new(vec![10, 20, 30]);
    let enumerated = lender.enumerate();

    assert_exact_size(&enumerated);
    assert_fused(&enumerated);
    assert_double_ended(&enumerated);
}

#[test]
fn fallible_trait_adapters_skip() {
    use lender::{DoubleEndedFallibleLender, ExactSizeFallibleLender, FusedFallibleLender};

    fn assert_exact_size<L: ExactSizeFallibleLender>(_: &L) {}
    fn assert_fused<L: FusedFallibleLender>(_: &L) {}
    fn assert_double_ended<L: DoubleEndedFallibleLender>(_: &L) {}

    let lender = VecFallibleLender::new(vec![1, 2, 3, 4, 5]);
    let skipped = lender.skip(2);

    assert_exact_size(&skipped);
    assert_fused(&skipped);
    assert_double_ended(&skipped);

    // Test that skip works correctly with double-ended iteration
    let mut skipped = VecFallibleLender::new(vec![1, 2, 3, 4, 5]).skip(2);
    assert_eq!(skipped.next_back().unwrap(), Some(&5));
    assert_eq!(skipped.next_back().unwrap(), Some(&4));
    assert_eq!(skipped.next_back().unwrap(), Some(&3));
    assert_eq!(skipped.next_back().unwrap(), None);
}

#[test]
fn fallible_trait_adapters_take() {
    use lender::{DoubleEndedFallibleLender, ExactSizeFallibleLender, FusedFallibleLender};

    fn assert_exact_size<L: ExactSizeFallibleLender>(_: &L) {}
    fn assert_fused<L: FusedFallibleLender>(_: &L) {}
    fn assert_double_ended<L: DoubleEndedFallibleLender>(_: &L) {}

    let lender = VecFallibleLender::new(vec![1, 2, 3, 4, 5]);
    let taken = lender.take(3);

    assert_exact_size(&taken);
    assert_fused(&taken);
    assert_double_ended(&taken);

    // Test that take works correctly with double-ended iteration
    let mut taken = VecFallibleLender::new(vec![1, 2, 3, 4, 5]).take(3);
    assert_eq!(taken.next_back().unwrap(), Some(&3));
    assert_eq!(taken.next_back().unwrap(), Some(&2));
    assert_eq!(taken.next_back().unwrap(), Some(&1));
    assert_eq!(taken.next_back().unwrap(), None);
}

#[test]
fn fallible_trait_adapters_zip() {
    use lender::{DoubleEndedFallibleLender, ExactSizeFallibleLender, FusedFallibleLender};

    fn assert_exact_size<L: ExactSizeFallibleLender>(_: &L) {}
    fn assert_fused<L: FusedFallibleLender>(_: &L) {}
    fn assert_double_ended<L: DoubleEndedFallibleLender>(_: &L) {}

    let lender1 = VecFallibleLender::new(vec![1, 2, 3]);
    let lender2 = VecFallibleLender::new(vec![10, 20, 30]);
    let zipped = lender1.zip(lender2);

    assert_exact_size(&zipped);
    assert_fused(&zipped);
    assert_double_ended(&zipped);

    // Test zip with double-ended iteration
    let mut zipped =
        VecFallibleLender::new(vec![1, 2, 3]).zip(VecFallibleLender::new(vec![10, 20, 30]));
    assert_eq!(zipped.next_back().unwrap(), Some((&3, &30)));
    assert_eq!(zipped.next_back().unwrap(), Some((&2, &20)));
    assert_eq!(zipped.next_back().unwrap(), Some((&1, &10)));
    assert_eq!(zipped.next_back().unwrap(), None);
}

#[test]
fn fallible_trait_adapters_rev() {
    use lender::{DoubleEndedFallibleLender, ExactSizeFallibleLender, FusedFallibleLender};

    fn assert_exact_size<L: ExactSizeFallibleLender>(_: &L) {}
    fn assert_fused<L: FusedFallibleLender>(_: &L) {}
    fn assert_double_ended<L: DoubleEndedFallibleLender>(_: &L) {}

    let lender = VecFallibleLender::new(vec![1, 2, 3]);
    let reversed = lender.rev();

    assert_exact_size(&reversed);
    assert_fused(&reversed);
    assert_double_ended(&reversed);

    // Test rev works correctly
    let mut reversed = VecFallibleLender::new(vec![1, 2, 3]).rev();
    assert_eq!(reversed.next().unwrap(), Some(&3));
    assert_eq!(reversed.next().unwrap(), Some(&2));
    assert_eq!(reversed.next().unwrap(), Some(&1));
    assert_eq!(reversed.next().unwrap(), None);
}

#[test]
fn fallible_trait_adapters_step_by() {
    use lender::{DoubleEndedFallibleLender, ExactSizeFallibleLender};

    fn assert_exact_size<L: ExactSizeFallibleLender>(_: &L) {}
    fn assert_double_ended<L: DoubleEndedFallibleLender>(_: &L) {}

    let lender = VecFallibleLender::new(vec![1, 2, 3, 4, 5, 6]);
    let stepped = lender.step_by(2);

    assert_exact_size(&stepped);
    assert_double_ended(&stepped);

    // Test step_by works correctly
    let mut stepped = VecFallibleLender::new(vec![1, 2, 3, 4, 5, 6]).step_by(2);
    assert_eq!(stepped.next().unwrap(), Some(&1));
    assert_eq!(stepped.next().unwrap(), Some(&3));
    assert_eq!(stepped.next().unwrap(), Some(&5));
    assert_eq!(stepped.next().unwrap(), None);

    // Test step_by with next_back
    let mut stepped = VecFallibleLender::new(vec![1, 2, 3, 4, 5, 6]).step_by(2);
    assert_eq!(stepped.next_back().unwrap(), Some(&5));
    assert_eq!(stepped.next_back().unwrap(), Some(&3));
    assert_eq!(stepped.next_back().unwrap(), Some(&1));
    assert_eq!(stepped.next_back().unwrap(), None);
}

#[test]
fn fallible_trait_adapters_chain() {
    use lender::FusedFallibleLender;

    fn assert_fused<L: FusedFallibleLender>(_: &L) {}

    let lender1 = VecFallibleLender::new(vec![1, 2, 3]);
    let lender2 = VecFallibleLender::new(vec![4, 5, 6]);
    let chained = lender1.chain(lender2);

    assert_fused(&chained);

    // Test chain works correctly
    let mut chained = VecFallibleLender::new(vec![1, 2]).chain(VecFallibleLender::new(vec![3, 4]));
    assert_eq!(chained.next().unwrap(), Some(&1));
    assert_eq!(chained.next().unwrap(), Some(&2));
    assert_eq!(chained.next().unwrap(), Some(&3));
    assert_eq!(chained.next().unwrap(), Some(&4));
    assert_eq!(chained.next().unwrap(), None);
}

#[test]
fn fallible_trait_adapters_inspect() {
    use lender::{ExactSizeFallibleLender, FusedFallibleLender};

    fn assert_exact_size<L: ExactSizeFallibleLender>(_: &L) {}
    fn assert_fused<L: FusedFallibleLender>(_: &L) {}

    let lender = VecFallibleLender::new(vec![1, 2, 3]);
    let inspected = lender.inspect(|_: &&i32| Ok(()));

    assert_exact_size(&inspected);
    assert_fused(&inspected);
}

#[test]
fn fallible_trait_adapters_fuse() {
    use lender::{ExactSizeFallibleLender, FusedFallibleLender};

    fn assert_exact_size<L: ExactSizeFallibleLender>(_: &L) {}
    fn assert_fused<L: FusedFallibleLender>(_: &L) {}

    let lender = VecFallibleLender::new(vec![1, 2, 3]);
    let fused = lender.fuse();

    assert_exact_size(&fused);
    assert_fused(&fused);
}

// ============================================================================
// Fallible max_by/min_by
// ============================================================================

#[test]
fn fallible_lender_max_by() {
    use lender::FallibleLender;

    // Use into_iter to get owned values, since max_by uses ToOwned
    let fallible: lender::IntoFallible<_> =
        vec![1, 5, 3].into_iter().into_lender().into_fallible();
    assert_eq!(fallible.max_by(|a, b| Ok(a.cmp(b))), Ok(Some(5)));

    // Per Iterator::max_by docs: "If several elements are equally maximum, the last element is returned."
    // Use abs() comparison so that -3 and 3 are equal; last should win.
    let fallible2: lender::IntoFallible<_> =
        vec![-3i32, 1, 3].into_iter().into_lender().into_fallible();
    assert_eq!(
        fallible2.max_by(|a, b| Ok(a.abs().cmp(&b.abs()))),
        Ok(Some(3))
    );
}

#[test]
fn fallible_lender_min_by() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<_> =
        vec![3, 1, 5].into_iter().into_lender().into_fallible();
    assert_eq!(fallible.min_by(|a, b| Ok(a.cmp(b))), Ok(Some(1)));

    // Per Iterator::min_by docs: "If several elements are equally minimum, the first element is returned."
    // Use abs() comparison so that -1 and 1 are equal; first should win.
    let fallible2: lender::IntoFallible<_> =
        vec![3i32, -1, 1].into_iter().into_lender().into_fallible();
    assert_eq!(
        fallible2.min_by(|a, b| Ok(a.abs().cmp(&b.abs()))),
        Ok(Some(-1))
    );
}

// ============================================================================
// Fallible into_fallible detailed
// ============================================================================

#[test]
fn fallible_into_fallible_basic() {
    use lender::FallibleLender;

    let mut fallible: lender::IntoFallible<_> = VecLender::new(vec![1, 2, 3]).into_fallible();

    assert_eq!(fallible.next(), Ok(Some(&1)));
    assert_eq!(fallible.next(), Ok(Some(&2)));
    assert_eq!(fallible.next(), Ok(Some(&3)));
    assert_eq!(fallible.next(), Ok(None));
}

#[test]
fn fallible_into_fallible_size_hint() {
    let fallible: lender::IntoFallible<_> = VecLender::new(vec![1, 2, 3]).into_fallible();
    assert_eq!(fallible.size_hint(), (3, Some(3)));
}

#[test]
fn fallible_into_fallible_double_ended() {
    use lender::DoubleEndedFallibleLender;

    let mut fallible: lender::IntoFallible<_> = VecLender::new(vec![1, 2, 3]).into_fallible();

    assert_eq!(fallible.next_back(), Ok(Some(&3)));
    assert_eq!(fallible.next(), Ok(Some(&1)));
    assert_eq!(fallible.next_back(), Ok(Some(&2)));
    assert_eq!(fallible.next(), Ok(None));
}

#[test]
fn fallible_into_fallible_exact_size() {
    use lender::ExactSizeFallibleLender;

    let fallible: lender::IntoFallible<_> = VecLender::new(vec![1, 2, 3]).into_fallible();
    assert_eq!(fallible.len(), 3);
}

#[test]
fn fallible_into_fallible_try_fold() {
    use lender::FallibleLender;

    let mut fallible: lender::IntoFallible<_> = VecLender::new(vec![1, 2, 3]).into_fallible();

    let result: Result<Option<i32>, core::convert::Infallible> = fallible.try_fold(0, |acc, x| Ok(Some(acc + *x)));
    assert_eq!(result, Ok(Some(6)));
}

#[test]
fn fallible_into_fallible_try_rfold() {
    use lender::DoubleEndedFallibleLender;

    let mut fallible: lender::IntoFallible<_> = VecLender::new(vec![1, 2, 3]).into_fallible();

    let result: Result<Option<i32>, core::convert::Infallible> = fallible.try_rfold(0, |acc, x| Ok(Some(acc + *x)));
    assert_eq!(result, Ok(Some(6)));
}

#[test]
fn fallible_into_inner() {
    let fallible: lender::IntoFallible<_> = VecLender::new(vec![1, 2, 3]).into_fallible();
    let inner = fallible.into_inner();
    assert_eq!(inner.count(), 3);
}

// ============================================================================
