mod common;
use common::*;
use ::lender::prelude::*;

// ============================================================================
// Fallible sources
// ============================================================================

#[test]
fn fallible_empty() {
    use lender::{fallible_empty, fallible_lend};

    // Test basic fallible_empty lender
    let mut empty = fallible_empty::<String, fallible_lend!(u32)>();
    assert!(empty.next().unwrap().is_none());
    assert!(empty.next().unwrap().is_none()); // Should continue returning None

    // Test that it's fused
    let mut empty_fused = fallible_empty::<String, fallible_lend!(i32)>();
    for _ in 0..10 {
        assert!(empty_fused.next().unwrap().is_none());
    }

    // Test fold operation
    let sum: Result<i32, String> =
        fallible_empty::<String, fallible_lend!(i32)>().fold(0, |acc, _x: i32| Ok(acc + 1));
    assert_eq!(sum, Ok(0)); // Should never iterate so result is 0

    // Test count
    let count: Result<usize, String> = fallible_empty::<String, fallible_lend!(i32)>().count();
    assert_eq!(count, Ok(0));

    // Test with reference type
    let mut empty_ref = fallible_empty::<String, fallible_lend!(&'lend str)>();
    assert!(empty_ref.next().unwrap().is_none());

    // FallibleEmpty should implement ExactSizeFallibleLender
    let empty_exact = fallible_empty::<String, fallible_lend!(i32)>();
    assert_eq!(lender::ExactSizeFallibleLender::len(&empty_exact), 0);
    assert!(lender::ExactSizeFallibleLender::is_empty(&empty_exact));
}

#[test]
fn fallible_once() {
    use lender::{fallible_lend, fallible_once};

    // Test with Ok value
    let mut once = fallible_once::<String, fallible_lend!(i32)>(Ok(42));
    assert_eq!(once.next().unwrap(), Some(42));
    assert!(once.next().unwrap().is_none());
    assert!(once.next().unwrap().is_none()); // Should continue returning None (fused)

    // Test with Err value
    let mut once_err = fallible_once::<String, fallible_lend!(i32)>(Err("error".to_string()));
    match once_err.next() {
        Err(e) => assert_eq!(e, "error"),
        Ok(_) => panic!("Expected error"),
    }
    // After an error, should return None
    assert!(once_err.next().unwrap().is_none());

    // Test fold with Ok
    let sum: Result<i32, String> =
        fallible_once::<String, fallible_lend!(i32)>(Ok(10)).fold(0, |acc, x| Ok(acc + x));
    assert_eq!(sum, Ok(10));

    // Test fold with Err
    let sum_err: Result<i32, String> =
        fallible_once::<String, fallible_lend!(i32)>(Err("error".to_string()))
            .fold(0, |acc, x: i32| Ok(acc + x));
    assert!(sum_err.is_err());

    // Test count with Ok
    let count: Result<usize, String> = fallible_once::<String, fallible_lend!(i32)>(Ok(42)).count();
    assert_eq!(count, Ok(1));

    // Test count with Err
    let count_err: Result<usize, String> =
        fallible_once::<String, fallible_lend!(i32)>(Err("error".to_string())).count();
    assert!(count_err.is_err());

    // FallibleOnce should implement ExactSizeFallibleLender
    let once_exact = fallible_once::<String, fallible_lend!(i32)>(Ok(42));
    assert_eq!(lender::ExactSizeFallibleLender::len(&once_exact), 1);
    assert!(!lender::ExactSizeFallibleLender::is_empty(&once_exact));
}

#[test]
fn fallible_repeat() {
    use lender::{fallible_lend, fallible_repeat};

    // Test with Ok value
    let mut repeat = fallible_repeat::<String, fallible_lend!(i32)>(Ok(42));
    assert_eq!(repeat.next().unwrap(), Some(42));
    assert_eq!(repeat.next().unwrap(), Some(42));
    assert_eq!(repeat.next().unwrap(), Some(42));
    // Should continue repeating
    for _ in 0..100 {
        assert_eq!(repeat.next().unwrap(), Some(42));
    }

    // Test with Err value
    let mut repeat_err = fallible_repeat::<String, fallible_lend!(i32)>(Err("error".to_string()));
    match repeat_err.next() {
        Err(e) => assert_eq!(e, "error"),
        Ok(_) => panic!("Expected error"),
    }
    // Should continue to return the same error
    match repeat_err.next() {
        Err(e) => assert_eq!(e, "error"),
        Ok(_) => panic!("Expected error"),
    }

    // Test take with Ok - manually collect
    let mut collected = Vec::new();
    let result = fallible_repeat::<String, fallible_lend!(i32)>(Ok(5))
        .take(3)
        .for_each(|x| {
            collected.push(x);
            Ok(())
        });
    assert!(result.is_ok());
    assert_eq!(collected, vec![5, 5, 5]);

    // Test take with Err - should fail on first item
    let mut collected_err = Vec::new();
    let result_err = fallible_repeat::<String, fallible_lend!(i32)>(Err("error".to_string()))
        .take(3)
        .for_each(|x| {
            collected_err.push(x);
            Ok(())
        });
    assert!(result_err.is_err());
    assert!(collected_err.is_empty()); // Should not have collected anything

    // size_hint should indicate infinite iterator
    let repeat_hint = fallible_repeat::<String, fallible_lend!(i32)>(Ok(42));
    assert_eq!(repeat_hint.size_hint(), (usize::MAX, None));

    // FallibleRepeat should be double-ended (infinite both ways)
    let mut repeat_de = fallible_repeat::<String, fallible_lend!(i32)>(Ok(7));
    assert_eq!(repeat_de.next_back().unwrap(), Some(7));
    assert_eq!(repeat_de.next_back().unwrap(), Some(7));
    assert_eq!(repeat_de.next().unwrap(), Some(7));
}

#[test]
fn fallible_once_with() {
    use lender::{fallible_once_with, hrc_once};

    // Test with Ok value from closure
    let mut once_with = fallible_once_with(
        42,
        hrc_once!(move |x: &mut i32| -> Result<i32, String> { Ok(*x) }),
    );
    assert_eq!(once_with.next().unwrap(), Some(42));
    assert!(once_with.next().unwrap().is_none());
    assert!(once_with.next().unwrap().is_none()); // Should be fused

    // Test with Err value from closure
    let mut once_with_err = fallible_once_with(
        42,
        hrc_once!(move |_x: &mut i32| -> Result<i32, String> { Err("error".to_string()) }),
    );
    match once_with_err.next() {
        Err(e) => assert_eq!(e, "error"),
        Ok(_) => panic!("Expected error"),
    }
    assert!(once_with_err.next().unwrap().is_none());
}

#[test]
fn fallible_repeat_with() {
    use lender::{fallible_lend, fallible_repeat_with};

    // Test with closure that returns Ok
    let mut counter = 0;
    let mut repeat_with = fallible_repeat_with::<'_, fallible_lend!(i32), String, _>(move || {
        counter += 1;
        Ok(counter)
    });
    assert_eq!(repeat_with.next().unwrap(), Some(1));
    assert_eq!(repeat_with.next().unwrap(), Some(2));
    assert_eq!(repeat_with.next().unwrap(), Some(3));

    // Test with closure that returns Err
    let mut repeat_with_err =
        fallible_repeat_with::<'_, fallible_lend!(i32), String, _>(|| Err("error".to_string()));
    match repeat_with_err.next() {
        Err(e) => assert_eq!(e, "error"),
        Ok(_) => panic!("Expected error"),
    }
    // Should continue to return errors
    match repeat_with_err.next() {
        Err(e) => assert_eq!(e, "error"),
        Ok(_) => panic!("Expected error"),
    }

    // size_hint should indicate infinite iterator
    let repeat_with_hint = fallible_repeat_with::<'_, fallible_lend!(i32), String, _>(|| Ok(1));
    assert_eq!(repeat_with_hint.size_hint(), (usize::MAX, None));

    // FallibleRepeatWith should be double-ended (infinite both ways)
    let mut repeat_with_de = fallible_repeat_with::<'_, fallible_lend!(i32), String, _>(|| Ok(99));
    assert_eq!(repeat_with_de.next_back().unwrap(), Some(99));
    assert_eq!(repeat_with_de.next_back().unwrap(), Some(99));
    assert_eq!(repeat_with_de.next().unwrap(), Some(99));
}

#[test]
fn from_fallible_fn() {
    use lender::from_fallible_fn;

    // Test with stateful closure that counts up
    let mut from_fn = from_fallible_fn(0, |state: &mut i32| -> Result<Option<i32>, String> {
        *state += 1;
        if *state <= 3 {
            Ok(Some(*state))
        } else {
            Ok(None)
        }
    });
    assert_eq!(from_fn.next().unwrap(), Some(1));
    assert_eq!(from_fn.next().unwrap(), Some(2));
    assert_eq!(from_fn.next().unwrap(), Some(3));
    assert!(from_fn.next().unwrap().is_none());
    assert!(from_fn.next().unwrap().is_none()); // Should continue returning None

    // Test with closure that returns error
    let mut from_fn_err = from_fallible_fn(0, |state: &mut i32| -> Result<Option<i32>, String> {
        *state += 1;
        if *state == 2 {
            Err("error".to_string())
        } else if *state < 4 {
            Ok(Some(*state))
        } else {
            Ok(None)
        }
    });
    assert_eq!(from_fn_err.next().unwrap(), Some(1));
    match from_fn_err.next() {
        Err(e) => assert_eq!(e, "error"),
        Ok(_) => panic!("Expected error"),
    }
}

// ============================================================================
// Fallible adapters
// ============================================================================

#[test]
fn into_fallible_adapter() {
    use lender::prelude::*;

    // Test converting a normal lender to fallible
    let data = vec![1, 2, 3];
    let mut fallible = data.into_iter().into_lender().into_fallible::<String>();
    assert_eq!(fallible.next().unwrap(), Some(1));
    assert_eq!(fallible.next().unwrap(), Some(2));
    assert_eq!(fallible.next().unwrap(), Some(3));
    assert!(fallible.next().unwrap().is_none());

    // Test with fold
    let data2 = vec![10, 20, 30];
    let sum: Result<i32, String> = data2
        .into_iter()
        .into_lender()
        .into_fallible::<String>()
        .fold(0, |acc, x| Ok(acc + x));
    assert_eq!(sum, Ok(60));
}

#[test]
fn map_err_adapter() {
    use lender::{fallible_lend, fallible_once};

    // Test mapping error type
    let mut mapped = fallible_once::<i32, fallible_lend!(u32)>(Err(42))
        .map_err(|e: i32| format!("Error: {}", e));
    match mapped.next() {
        Err(e) => assert_eq!(e, "Error: 42"),
        Ok(_) => panic!("Expected error"),
    }

    // Test with Ok value (error mapper shouldn't be called)
    let mut mapped_ok = fallible_once::<String, fallible_lend!(i32)>(Ok(100))
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
        .into_fallible::<std::convert::Infallible>()
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
    let data = vec![
        Wrapper(vec![1, 2]),
        Wrapper(vec![3]),
        Wrapper(vec![4, 5]),
    ];
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
    let data = vec![
        Wrapper(vec![1, 2]),
        Wrapper(vec![]),
        Wrapper(vec![3, 4, 5]),
    ];
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
    let data = vec![
        Wrapper(vec![1, 2]),
        Wrapper(vec![3, 4]),
        Wrapper(vec![5]),
    ];
    let mut iter = data.into_iter().into_lender().into_fallible().flatten();
    let result: Result<i32, i32> = iter.try_fold(0, |acc, x| {
        let new = acc + x;
        if new > 6 {
            Ok(Err(new))
        } else {
            Ok(Ok(new))
        }
    }).unwrap();
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
    let fallible: lender::IntoFallible<(), _> = vec![1, 5, 3].into_iter().into_lender().into_fallible();
    assert_eq!(fallible.max_by(|a, b| Ok(a.cmp(b))), Ok(Some(5)));

    // Per Iterator::max_by docs: "If several elements are equally maximum, the last element is returned."
    // Use abs() comparison so that -3 and 3 are equal; last should win.
    let fallible2: lender::IntoFallible<(), _> = vec![-3i32, 1, 3].into_iter().into_lender().into_fallible();
    assert_eq!(
        fallible2.max_by(|a, b| Ok(a.abs().cmp(&b.abs()))),
        Ok(Some(3))
    );
}

#[test]
fn fallible_lender_min_by() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<(), _> = vec![3, 1, 5].into_iter().into_lender().into_fallible();
    assert_eq!(fallible.min_by(|a, b| Ok(a.cmp(b))), Ok(Some(1)));

    // Per Iterator::min_by docs: "If several elements are equally minimum, the first element is returned."
    // Use abs() comparison so that -1 and 1 are equal; first should win.
    let fallible2: lender::IntoFallible<(), _> = vec![3i32, -1, 1].into_iter().into_lender().into_fallible();
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

    let mut fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2, 3]).into_fallible();

    assert_eq!(fallible.next(), Ok(Some(&1)));
    assert_eq!(fallible.next(), Ok(Some(&2)));
    assert_eq!(fallible.next(), Ok(Some(&3)));
    assert_eq!(fallible.next(), Ok(None));
}

#[test]
fn fallible_into_fallible_size_hint() {
    let fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2, 3]).into_fallible();
    assert_eq!(fallible.size_hint(), (3, Some(3)));
}

#[test]
fn fallible_into_fallible_double_ended() {
    use lender::DoubleEndedFallibleLender;

    let mut fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2, 3]).into_fallible();

    assert_eq!(fallible.next_back(), Ok(Some(&3)));
    assert_eq!(fallible.next(), Ok(Some(&1)));
    assert_eq!(fallible.next_back(), Ok(Some(&2)));
    assert_eq!(fallible.next(), Ok(None));
}

#[test]
fn fallible_into_fallible_exact_size() {
    use lender::ExactSizeFallibleLender;

    let fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2, 3]).into_fallible();
    assert_eq!(fallible.len(), 3);
}

#[test]
fn fallible_into_fallible_try_fold() {
    use lender::FallibleLender;

    let mut fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2, 3]).into_fallible();

    let result: Result<Option<i32>, ()> = fallible.try_fold(0, |acc, x| Ok(Some(acc + *x)));
    assert_eq!(result, Ok(Some(6)));
}

#[test]
fn fallible_into_fallible_try_rfold() {
    use lender::DoubleEndedFallibleLender;

    let mut fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2, 3]).into_fallible();

    let result: Result<Option<i32>, ()> = fallible.try_rfold(0, |acc, x| Ok(Some(acc + *x)));
    assert_eq!(result, Ok(Some(6)));
}

#[test]
fn fallible_into_inner() {
    let fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2, 3]).into_fallible();
    let inner = fallible.into_inner();
    assert_eq!(inner.count(), 3);
}

// ============================================================================
// Comprehensive FallibleLender tests
// ============================================================================

#[test]
fn fallible_lender_next_chunk() {
    use lender::FallibleLender;

    let mut fallible: lender::IntoFallible<(), _> =
        VecLender::new(vec![1, 2, 3, 4, 5]).into_fallible();
    let mut chunk = fallible.next_chunk(2);
    assert_eq!(chunk.next(), Ok(Some(&1)));
    assert_eq!(chunk.next(), Ok(Some(&2)));
    assert_eq!(chunk.next(), Ok(None));
}

#[test]
fn fallible_lender_count() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2, 3, 4, 5]).into_fallible();
    assert_eq!(fallible.count(), Ok(5));
}

#[test]
fn fallible_lender_last() {
    use lender::FallibleLender;

    let mut fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2, 3]).into_fallible();
    assert_eq!(fallible.last(), Ok(Some(&3)));
}

#[test]
fn fallible_lender_advance_by() {
    use lender::FallibleLender;

    let mut fallible: lender::IntoFallible<(), _> =
        VecLender::new(vec![1, 2, 3, 4, 5]).into_fallible();
    assert_eq!(fallible.advance_by(2), Ok(Ok(())));
    assert_eq!(fallible.next(), Ok(Some(&3)));
}

#[test]
fn fallible_lender_nth() {
    use lender::FallibleLender;

    let mut fallible: lender::IntoFallible<(), _> =
        VecLender::new(vec![1, 2, 3, 4, 5]).into_fallible();
    assert_eq!(fallible.nth(2), Ok(Some(&3)));
}

#[test]
fn fallible_lender_step_by() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2, 3, 4, 5]).into_fallible();
    let mut stepped = fallible.step_by(2);
    assert_eq!(stepped.next(), Ok(Some(&1)));
    assert_eq!(stepped.next(), Ok(Some(&3)));
    assert_eq!(stepped.next(), Ok(Some(&5)));
    assert_eq!(stepped.next(), Ok(None));
}

#[test]
fn fallible_lender_chain() {
    use lender::FallibleLender;

    let fallible1: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2]).into_fallible();
    let fallible2: lender::IntoFallible<(), _> = VecLender::new(vec![3, 4]).into_fallible();
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

    let fallible1: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2, 3]).into_fallible();
    let fallible2: lender::IntoFallible<(), _> = VecLender::new(vec![4, 5, 6]).into_fallible();
    let mut zipped = fallible1.zip(fallible2);
    assert_eq!(zipped.next(), Ok(Some((&1, &4))));
    assert_eq!(zipped.next(), Ok(Some((&2, &5))));
    assert_eq!(zipped.next(), Ok(Some((&3, &6))));
    assert_eq!(zipped.next(), Ok(None));
}

#[test]
fn fallible_lender_map() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2, 3]).into_fallible();
    let mut mapped = fallible.map(|x: &i32| Ok(*x * 2));
    assert_eq!(mapped.next(), Ok(Some(2)));
    assert_eq!(mapped.next(), Ok(Some(4)));
    assert_eq!(mapped.next(), Ok(Some(6)));
    assert_eq!(mapped.next(), Ok(None));
}

#[test]
fn fallible_lender_filter() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<(), _> =
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

    let fallible: lender::IntoFallible<(), _> = VecLender::new(vec![10, 20, 30]).into_fallible();
    let mut enumerated = fallible.enumerate();
    assert_eq!(enumerated.next(), Ok(Some((0, &10))));
    assert_eq!(enumerated.next(), Ok(Some((1, &20))));
    assert_eq!(enumerated.next(), Ok(Some((2, &30))));
    assert_eq!(enumerated.next(), Ok(None));
}

#[test]
fn fallible_lender_skip() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2, 3, 4, 5]).into_fallible();
    let mut skipped = fallible.skip(2);
    assert_eq!(skipped.next(), Ok(Some(&3)));
    assert_eq!(skipped.next(), Ok(Some(&4)));
    assert_eq!(skipped.next(), Ok(Some(&5)));
    assert_eq!(skipped.next(), Ok(None));
}

#[test]
fn fallible_lender_take() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2, 3, 4, 5]).into_fallible();
    let mut taken = fallible.take(3);
    assert_eq!(taken.next(), Ok(Some(&1)));
    assert_eq!(taken.next(), Ok(Some(&2)));
    assert_eq!(taken.next(), Ok(Some(&3)));
    assert_eq!(taken.next(), Ok(None));
}

#[test]
fn fallible_lender_skip_while() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2, 3, 4, 5]).into_fallible();
    let mut skipped = fallible.skip_while(|&&x| Ok(x < 3));
    assert_eq!(skipped.next(), Ok(Some(&3)));
    assert_eq!(skipped.next(), Ok(Some(&4)));
    assert_eq!(skipped.next(), Ok(Some(&5)));
    assert_eq!(skipped.next(), Ok(None));
}

#[test]
fn fallible_lender_take_while() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2, 3, 4, 5]).into_fallible();
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
    let fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2, 3]).into_fallible();
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

    let fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2]).into_fallible();
    let mut fused = fallible.fuse();
    assert_eq!(fused.next(), Ok(Some(&1)));
    assert_eq!(fused.next(), Ok(Some(&2)));
    assert_eq!(fused.next(), Ok(None));
    assert_eq!(fused.next(), Ok(None)); // Fused stays None
}

#[test]
fn fallible_lender_fold() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2, 3, 4, 5]).into_fallible();
    let sum = fallible.fold(0, |acc, x| Ok(acc + *x));
    assert_eq!(sum, Ok(15));
}

#[test]
fn fallible_lender_for_each() {
    use lender::FallibleLender;

    let mut collected = Vec::new();
    let fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2, 3]).into_fallible();
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

    let mut fallible: lender::IntoFallible<(), _> = VecLender::new(vec![2, 4, 6]).into_fallible();
    assert_eq!(fallible.all(|x| Ok(*x % 2 == 0)), Ok(true));

    let mut fallible2: lender::IntoFallible<(), _> = VecLender::new(vec![2, 3, 6]).into_fallible();
    assert_eq!(fallible2.all(|x| Ok(*x % 2 == 0)), Ok(false));
}

#[test]
fn fallible_lender_any() {
    use lender::FallibleLender;

    let mut fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 3, 5]).into_fallible();
    assert_eq!(fallible.any(|x| Ok(*x % 2 == 0)), Ok(false));

    let mut fallible2: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2, 3]).into_fallible();
    assert_eq!(fallible2.any(|x| Ok(*x % 2 == 0)), Ok(true));
}

#[test]
fn fallible_lender_find() {
    use lender::FallibleLender;

    let mut fallible: lender::IntoFallible<(), _> =
        VecLender::new(vec![1, 2, 3, 4, 5]).into_fallible();
    assert_eq!(fallible.find(|&&x| Ok(x > 3)), Ok(Some(&4)));
}

#[test]
fn fallible_lender_position() {
    use lender::FallibleLender;

    let mut fallible: lender::IntoFallible<(), _> =
        VecLender::new(vec![1, 2, 3, 4, 5]).into_fallible();
    assert_eq!(fallible.position(|x| Ok(*x == 3)), Ok(Some(2)));
}

#[test]
fn fallible_lender_rposition() {
    use lender::FallibleLender;

    let mut fallible: lender::IntoFallible<(), _> =
        VecLender::new(vec![1, 2, 3, 4, 5]).into_fallible();
    assert_eq!(fallible.rposition(|x| Ok(*x == 3)), Ok(Some(2)));
}

#[test]
fn lender_convert() {
    use lender::FallibleLender;

    let data = vec![Ok(1), Ok(2), Err("oops")];
    let mut lender = lender::from_iter(data.into_iter()).convert::<&str>();
    assert_eq!(lender.next(), Ok(Some(1)));
    assert_eq!(lender.next(), Ok(Some(2)));
    assert!(lender.next().is_err());
}

#[test]
fn fallible_lender_chunky() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<(), _> =
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

    let fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2, 3]).into_fallible();
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

    let mut fallible: lender::IntoFallible<(), _> =
        VecLender::new(vec![1, 2, 3, 4, 5]).into_fallible();
    assert_eq!(fallible.advance_back_by(2), Ok(Ok(())));
    assert_eq!(fallible.next_back(), Ok(Some(&3)));
}

#[test]
fn double_ended_fallible_nth_back() {
    use lender::DoubleEndedFallibleLender;

    let mut fallible: lender::IntoFallible<(), _> =
        VecLender::new(vec![1, 2, 3, 4, 5]).into_fallible();
    assert_eq!(fallible.nth_back(2), Ok(Some(&3)));
}

#[test]
fn double_ended_fallible_try_rfold() {
    use lender::DoubleEndedFallibleLender;

    let mut fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2, 3]).into_fallible();
    let result: Result<Option<i32>, ()> = fallible.try_rfold(0, |acc, x| Ok(Some(acc + *x)));
    assert_eq!(result, Ok(Some(6)));
}

#[test]
fn double_ended_fallible_rfold() {
    use lender::DoubleEndedFallibleLender;

    let fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2, 3]).into_fallible();
    let values: Result<Vec<i32>, ()> = fallible.rfold(Vec::new(), |mut acc, x| {
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

    let fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2, 3]).into_fallible();
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

    let fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1]).into_fallible();
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

    let fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1]).into_fallible();
    let mut peekable = fallible.peekable();
    // Peek the only value
    let _ = peekable.peek();
    // next_back should return the peeked value through the unsafe transmute path
    assert_eq!(peekable.next_back(), Ok(Some(&1)));
}

// FalliblePeekable::peek_mut (covers unsafe at line 57, 65)
#[test]
fn fallible_peekable_peek_mut() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2, 3]).into_fallible();
    let mut peekable = fallible.peekable();
    // peek_mut to store a value and get mutable reference
    let peeked = peekable.peek_mut().unwrap();
    assert_eq!(peeked, Some(&mut &1));
}

// FalliblePeekable::next_if (covers unsafe at lines 76, 85)
#[test]
fn fallible_peekable_next_if_match() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2, 3]).into_fallible();
    let mut peekable = fallible.peekable();
    // next_if should return Some when predicate matches
    assert_eq!(peekable.next_if(|&&x| x == 1), Ok(Some(&1)));
    // Should have advanced
    assert_eq!(peekable.next(), Ok(Some(&2)));
}

#[test]
fn fallible_peekable_next_if_no_match() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2, 3]).into_fallible();
    let mut peekable = fallible.peekable();
    // next_if should return None when predicate doesn't match (and store in peeked)
    assert_eq!(peekable.next_if(|&&x| x == 5), Ok(None));
    // Value should still be available
    assert_eq!(peekable.next(), Ok(Some(&1)));
}

// ============================================================================
// Iter fallible iterator
// ============================================================================

// Iter adapter FallibleIterator next (covers unsafe at line 101-102)
// Note: .iter() requires the Lend type to satisfy complex higher-ranked trait bounds.
// With VecFallibleLender yielding &'lend i32, there are lifetime issues that prevent
// it from working with .iter(). We test with owned values via into_iter().into_lender().into_fallible()
// which yields i32 (Copy type with no lifetime issues).
#[test]
fn iter_fallible_iterator_next() {
    use fallible_iterator::FallibleIterator;

    let fallible = vec![1, 2, 3].into_iter().into_lender().into_fallible::<()>();
    let mut iter = fallible.iter();
    assert_eq!(FallibleIterator::next(&mut iter), Ok(Some(1)));
    assert_eq!(FallibleIterator::next(&mut iter), Ok(Some(2)));
    assert_eq!(FallibleIterator::next(&mut iter), Ok(Some(3)));
    assert_eq!(FallibleIterator::next(&mut iter), Ok(None));
}

// Iter adapter DoubleEndedFallibleIterator next_back (covers unsafe at line 120-121)
#[test]
fn iter_double_ended_fallible_iterator_next_back() {
    use fallible_iterator::DoubleEndedFallibleIterator;

    let fallible = vec![1, 2, 3].into_iter().into_lender().into_fallible::<()>();
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

// Cycle fallible next (covers unsafe reborrow at line 129)
#[test]
fn cycle_fallible_next_coverage() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2]).into_fallible();
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
    use lender::{
        FallibleLend, FallibleLender, FallibleLending, FusedFallibleLender,
    };

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

        fn advance_back_by(
            &mut self,
            n: usize,
        ) -> Result<Result<(), NonZeroUsize>, Self::Error> {
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

// ============================================================================
// New method tests (M5–M7, M11)
// ============================================================================

// M5: Fallible Zip nth_back — equal-length lenders
#[test]
fn fallible_zip_nth_back_equal_length() {
    let mut zipped = VecFallibleLender::new(vec![1, 2, 3, 4, 5])
        .zip(VecFallibleLender::new(vec![10, 20, 30, 40, 50]));
    assert_eq!(zipped.nth_back(0), Ok(Some((&5, &50))));
    assert_eq!(zipped.nth_back(1), Ok(Some((&3, &30))));
    assert_eq!(zipped.nth_back(2), Ok(None));
}

// M5: Fallible Zip nth_back — unequal-length lenders
#[test]
fn fallible_zip_nth_back_unequal_length() {
    let mut zipped = VecFallibleLender::new(vec![1, 2, 3, 4, 5])
        .zip(VecFallibleLender::new(vec![10, 20, 30]));
    assert_eq!(zipped.nth_back(0), Ok(Some((&3, &30))));
    assert_eq!(zipped.nth_back(0), Ok(Some((&2, &20))));
    assert_eq!(zipped.nth_back(0), Ok(Some((&1, &10))));
    assert_eq!(zipped.nth_back(0), Ok(None));
}

// M5: Fallible Zip nth_back — empty
#[test]
fn fallible_zip_nth_back_empty() {
    let mut zipped = VecFallibleLender::new(vec![])
        .zip(VecFallibleLender::new(vec![1, 2]));
    assert_eq!(zipped.nth_back(0), Ok(None));
}

// M6: Fallible StepBy count
#[test]
fn fallible_step_by_count() {
    let lender = VecFallibleLender::new(vec![1, 2, 3, 4, 5, 6, 7]);
    // step=2 yields [1, 3, 5, 7] → count = 4
    assert_eq!(lender.step_by(2).count(), Ok(4));
}

#[test]
fn fallible_step_by_count_step_one() {
    let lender = VecFallibleLender::new(vec![1, 2, 3]);
    assert_eq!(lender.step_by(1).count(), Ok(3));
}

#[test]
fn fallible_step_by_count_empty() {
    let lender = VecFallibleLender::new(vec![]);
    assert_eq!(lender.step_by(3).count(), Ok(0));
}

// M7: Fallible Chunk count
#[test]
fn fallible_chunk_count() {
    let mut lender = VecFallibleLender::new(vec![1, 2, 3, 4, 5]);
    let chunk = lender.next_chunk(3);
    assert_eq!(chunk.count(), Ok(3));
}

#[test]
fn fallible_chunk_count_larger_than_remaining() {
    let mut lender = VecFallibleLender::new(vec![1, 2]);
    let chunk = lender.next_chunk(5);
    assert_eq!(chunk.count(), Ok(2));
}

#[test]
fn fallible_chunk_count_empty() {
    let mut lender = VecFallibleLender::new(vec![]);
    let chunk = lender.next_chunk(3);
    assert_eq!(chunk.count(), Ok(0));
}

// M7: Fallible Chunk nth
#[test]
fn fallible_chunk_nth_within_range() {
    let mut lender = VecFallibleLender::new(vec![10, 20, 30, 40, 50]);
    let mut chunk = lender.next_chunk(4);
    assert_eq!(chunk.nth(2), Ok(Some(&30)));
    assert_eq!(chunk.next(), Ok(Some(&40)));
    assert_eq!(chunk.next(), Ok(None));
}

#[test]
fn fallible_chunk_nth_past_end() {
    let mut lender = VecFallibleLender::new(vec![10, 20, 30]);
    let mut chunk = lender.next_chunk(3);
    assert_eq!(chunk.nth(5), Ok(None));
    assert_eq!(chunk.next(), Ok(None));
}

// M7: Fallible Chunk try_fold
#[test]
fn fallible_chunk_try_fold() {
    let mut lender = VecFallibleLender::new(vec![1, 2, 3, 4, 5]);
    let mut chunk = lender.next_chunk(4);
    let result: Result<Result<i32, ()>, _> = chunk.try_fold(0, |acc, x| Ok(Ok(acc + *x)));
    assert_eq!(result, Ok(Ok(10)));
}

// M7: Fallible Chunk fold
#[test]
fn fallible_chunk_fold() {
    let mut lender = VecFallibleLender::new(vec![1, 2, 3, 4, 5]);
    let chunk = lender.next_chunk(4);
    let result = chunk.fold(0, |acc, x| Ok(acc + *x));
    assert_eq!(result, Ok(10));
}

// M11: Fallible Intersperse try_fold
#[test]
fn fallible_intersperse_try_fold() {
    use lender::from_fallible_fn;

    let interspersed = from_fallible_fn(0, |state: &mut i32| -> Result<Option<i32>, String> {
        *state += 1;
        if *state <= 3 { Ok(Some(*state)) } else { Ok(None) }
    })
    .intersperse(0);

    // try_fold to collect into a Vec via for_each (which calls try_fold internally)
    let mut collected = Vec::new();
    interspersed
        .for_each(|x| {
            collected.push(x);
            Ok(())
        })
        .unwrap();
    assert_eq!(collected, vec![1, 0, 2, 0, 3]);
}

// M11: Fallible Intersperse try_fold with fold (full consumption)
#[test]
fn fallible_intersperse_fold() {
    use lender::from_fallible_fn;

    let interspersed = from_fallible_fn(0, |state: &mut i32| -> Result<Option<i32>, String> {
        *state += 1;
        if *state <= 4 { Ok(Some(*state)) } else { Ok(None) }
    })
    .intersperse(0);

    // fold sums all elements: 1 + 0 + 2 + 0 + 3 + 0 + 4 = 10
    let sum = interspersed.fold(0, |acc, x| Ok(acc + x)).unwrap();
    assert_eq!(sum, 10);
}

// M11: Fallible IntersperseWith try_fold
#[test]
fn fallible_intersperse_with_try_fold() {
    use lender::from_fallible_fn;

    let mut sep_counter = 100;
    let interspersed = from_fallible_fn(0, |state: &mut i32| -> Result<Option<i32>, String> {
        *state += 1;
        if *state <= 3 { Ok(Some(*state)) } else { Ok(None) }
    })
    .intersperse_with(move || {
        sep_counter += 1;
        Ok(sep_counter)
    });

    let mut collected = Vec::new();
    interspersed
        .for_each(|x| {
            collected.push(x);
            Ok(())
        })
        .unwrap();
    assert_eq!(collected, vec![1, 101, 2, 102, 3]);
}

// M11: Fallible IntersperseWith fold
#[test]
fn fallible_intersperse_with_fold() {
    use lender::from_fallible_fn;

    let interspersed = from_fallible_fn(0, |state: &mut i32| -> Result<Option<i32>, String> {
        *state += 1;
        if *state <= 3 { Ok(Some(*state)) } else { Ok(None) }
    })
    .intersperse_with(|| Ok(0));

    let sum = interspersed.fold(0, |acc, x| Ok(acc + x)).unwrap();
    assert_eq!(sum, 6); // 1 + 0 + 2 + 0 + 3 = 6
}
