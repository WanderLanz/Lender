//! Tests for fallible coverage: new methods, adapter-specific tests, error propagation, composition

mod common;
use ::lender::prelude::*;
use common::*;
use fallible_iterator::{DoubleEndedFallibleIterator, FallibleIterator};

// ============================================================================

#[test]
fn test_fallible_zip_nth_back_equal_length() {
    let mut zipped = VecFallibleLender::new(vec![1, 2, 3, 4, 5])
        .zip(VecFallibleLender::new(vec![10, 20, 30, 40, 50]));
    assert_eq!(zipped.nth_back(0), Ok(Some((&5, &50))));
    assert_eq!(zipped.nth_back(1), Ok(Some((&3, &30))));
    assert_eq!(zipped.nth_back(2), Ok(None));
}

#[test]
fn test_fallible_zip_nth_back_unequal_length() {
    let mut zipped =
        VecFallibleLender::new(vec![1, 2, 3, 4, 5]).zip(VecFallibleLender::new(vec![10, 20, 30]));
    assert_eq!(zipped.nth_back(0), Ok(Some((&3, &30))));
    assert_eq!(zipped.nth_back(0), Ok(Some((&2, &20))));
    assert_eq!(zipped.nth_back(0), Ok(Some((&1, &10))));
    assert_eq!(zipped.nth_back(0), Ok(None));
}

#[test]
fn test_fallible_zip_nth_back_empty() {
    let mut zipped = VecFallibleLender::new(vec![]).zip(VecFallibleLender::new(vec![1, 2]));
    assert_eq!(zipped.nth_back(0), Ok(None));
}

#[test]
fn test_fallible_step_by_count() {
    let lender = VecFallibleLender::new(vec![1, 2, 3, 4, 5, 6, 7]);
    // step=2 yields [1, 3, 5, 7] → count = 4
    assert_eq!(lender.step_by(2).count(), Ok(4));
}

#[test]
fn test_fallible_step_by_count_step_one() {
    let lender = VecFallibleLender::new(vec![1, 2, 3]);
    assert_eq!(lender.step_by(1).count(), Ok(3));
}

#[test]
fn test_fallible_step_by_count_empty() {
    let lender = VecFallibleLender::new(vec![]);
    assert_eq!(lender.step_by(3).count(), Ok(0));
}

#[test]
fn test_fallible_chunk_count() {
    let mut lender = VecFallibleLender::new(vec![1, 2, 3, 4, 5]);
    let chunk = lender.next_chunk(3);
    assert_eq!(chunk.count(), Ok(3));
}

#[test]
fn test_fallible_chunk_count_larger_than_remaining() {
    let mut lender = VecFallibleLender::new(vec![1, 2]);
    let chunk = lender.next_chunk(5);
    assert_eq!(chunk.count(), Ok(2));
}

#[test]
fn test_fallible_chunk_count_empty() {
    let mut lender = VecFallibleLender::new(vec![]);
    let chunk = lender.next_chunk(3);
    assert_eq!(chunk.count(), Ok(0));
}

#[test]
fn test_fallible_chunk_nth_within_range() {
    let mut lender = VecFallibleLender::new(vec![10, 20, 30, 40, 50]);
    let mut chunk = lender.next_chunk(4);
    assert_eq!(chunk.nth(2), Ok(Some(&30)));
    assert_eq!(chunk.next(), Ok(Some(&40)));
    assert_eq!(chunk.next(), Ok(None));
}

#[test]
fn test_fallible_chunk_nth_past_end() {
    let mut lender = VecFallibleLender::new(vec![10, 20, 30]);
    let mut chunk = lender.next_chunk(3);
    assert_eq!(chunk.nth(5), Ok(None));
    assert_eq!(chunk.next(), Ok(None));
}

#[test]
fn test_fallible_chunk_try_fold() {
    let mut lender = VecFallibleLender::new(vec![1, 2, 3, 4, 5]);
    let mut chunk = lender.next_chunk(4);
    let result: Result<Result<i32, ()>, _> = chunk.try_fold(0, |acc, x| Ok(Ok(acc + *x)));
    assert_eq!(result, Ok(Ok(10)));
}

#[test]
fn test_fallible_chunk_fold() {
    let mut lender = VecFallibleLender::new(vec![1, 2, 3, 4, 5]);
    let chunk = lender.next_chunk(4);
    let result = chunk.fold(0, |acc, x| Ok(acc + *x));
    assert_eq!(result, Ok(10));
}

#[test]
fn test_fallible_intersperse_try_fold() {
    use lender::from_fallible_fn;

    let interspersed = from_fallible_fn(
        0,
        covar_mut!(
            for<'lend> |state: &'lend mut i32| -> Result<Option<i32>, String> {
                *state += 1;
                if *state <= 3 {
                    Ok(Some(*state))
                } else {
                    Ok(None)
                }
            }
        ),
    )
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

#[test]
fn test_fallible_intersperse_fold() {
    use lender::from_fallible_fn;

    let interspersed = from_fallible_fn(
        0,
        covar_mut!(
            for<'lend> |state: &'lend mut i32| -> Result<Option<i32>, String> {
                *state += 1;
                if *state <= 4 {
                    Ok(Some(*state))
                } else {
                    Ok(None)
                }
            }
        ),
    )
    .intersperse(0);

    // fold sums all elements: 1 + 0 + 2 + 0 + 3 + 0 + 4 = 10
    let sum = interspersed.fold(0, |acc, x| Ok(acc + x)).unwrap();
    assert_eq!(sum, 10);
}

#[test]
fn test_fallible_intersperse_with_try_fold() {
    use lender::from_fallible_fn;

    let mut sep_counter = 100;
    let interspersed = from_fallible_fn(
        0,
        covar_mut!(
            for<'lend> |state: &'lend mut i32| -> Result<Option<i32>, String> {
                *state += 1;
                if *state <= 3 {
                    Ok(Some(*state))
                } else {
                    Ok(None)
                }
            }
        ),
    )
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

#[test]
fn test_fallible_intersperse_with_fold() {
    use lender::from_fallible_fn;

    let interspersed = from_fallible_fn(
        0,
        covar_mut!(
            for<'lend> |state: &'lend mut i32| -> Result<Option<i32>, String> {
                *state += 1;
                if *state <= 3 {
                    Ok(Some(*state))
                } else {
                    Ok(None)
                }
            }
        ),
    )
    .intersperse_with(|| Ok(0));

    let sum = interspersed.fold(0, |acc, x| Ok(acc + x)).unwrap();
    assert_eq!(sum, 6); // 1 + 0 + 2 + 0 + 3 = 6
}

// ============================================================================
// try_find tests (fallible)
// ============================================================================

#[test]
fn test_fallible_try_find_found() {
    use lender::from_fallible_fn;

    let mut lender = from_fallible_fn(
        0,
        covar_mut!(
            for<'lend> |state: &'lend mut i32| -> Result<Option<i32>, String> {
                *state += 1;
                if *state <= 5 {
                    Ok(Some(*state))
                } else {
                    Ok(None)
                }
            }
        ),
    );
    // try_find with R = Option<bool>: None short-circuits, Some(true) finds, Some(false) skips
    let result = lender.try_find(|x| Ok(if *x == 3 { Some(true) } else { Some(false) }));
    assert!(result.is_ok());
    let inner = result.unwrap();
    assert_eq!(inner, Some(Some(3)));
}

#[test]
fn test_fallible_try_find_not_found() {
    use lender::from_fallible_fn;

    let mut lender = from_fallible_fn(
        0,
        covar_mut!(
            for<'lend> |state: &'lend mut i32| -> Result<Option<i32>, String> {
                *state += 1;
                if *state <= 3 {
                    Ok(Some(*state))
                } else {
                    Ok(None)
                }
            }
        ),
    );
    let result = lender.try_find(|x| Ok(if *x == 99 { Some(true) } else { Some(false) }));
    assert!(result.is_ok());
    let inner = result.unwrap();
    assert_eq!(inner, Some(None));
}

#[test]
fn test_fallible_try_find_closure_short_circuit() {
    use lender::from_fallible_fn;

    let mut lender = from_fallible_fn(
        0,
        covar_mut!(
            for<'lend> |state: &'lend mut i32| -> Result<Option<i32>, String> {
                *state += 1;
                if *state <= 5 {
                    Ok(Some(*state))
                } else {
                    Ok(None)
                }
            }
        ),
    );
    // Short-circuit: closure returns Ok(None) which breaks the Try
    let result = lender.try_find(|x| Ok(if *x == 3 { None } else { Some(false) }));
    assert!(result.is_ok());
    let inner = result.unwrap();
    assert_eq!(inner, None); // None from the short-circuit
}

#[test]
fn test_fallible_try_find_lender_error() {
    let mut lender = ErrorAtLender::new(vec![1, 2, 3, 4, 5], 2);
    // The lender errors at index 2, so try_find should propagate that error
    let result = lender.try_find(|x| Ok(if **x == 5 { Some(true) } else { Some(false) }));
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "error at index 2");
}

// ============================================================================
// Fallible adapter-specific tests
// ============================================================================

#[test]
fn test_fallible_scan_basic() {
    use lender::from_fallible_fn;

    let mut lender = from_fallible_fn(
        0,
        covar_mut!(
            for<'all> |state: &'all mut i32| -> Result<Option<i32>, String> {
                *state += 1;
                if *state <= 5 {
                    Ok(Some(*state))
                } else {
                    Ok(None)
                }
            }
        ),
    )
    .scan(
        0,
        covar_mut!(
            for<'all> |(state, x): (&'all mut i32, i32)| -> Result<Option<i32>, String> {
                *state += x;
                Ok(if *state > 6 { None } else { Some(*state) })
            }
        ),
    );

    // Running sum: 1, 3, 6 — next would be 10 > 6, so stops
    assert_eq!(lender.next().unwrap(), Some(1));
    assert_eq!(lender.next().unwrap(), Some(3));
    assert_eq!(lender.next().unwrap(), Some(6));
    assert_eq!(lender.next().unwrap(), None);
}

#[test]
fn test_fallible_filter_map_basic() {
    use lender::from_fallible_fn;

    let lender = from_fallible_fn(
        0,
        covar_mut!(
            for<'lend> |state: &'lend mut i32| -> Result<Option<i32>, String> {
                *state += 1;
                if *state <= 6 {
                    Ok(Some(*state))
                } else {
                    Ok(None)
                }
            }
        ),
    )
    .filter_map(covar_mut!(
        for<'lend> |x: i32| -> Result<Option<i32>, String> {
            Ok(if x % 2 == 0 { Some(x * 10) } else { None })
        }
    ));

    // Even elements doubled: 2*10=20, 4*10=40, 6*10=60
    let mut result = Vec::new();
    lender
        .for_each(|x| {
            result.push(x);
            Ok(())
        })
        .unwrap();
    assert_eq!(result, vec![20, 40, 60]);
}

#[test]
fn test_fallible_map_while_basic() {
    use lender::from_fallible_fn;

    let mut lender = from_fallible_fn(
        0,
        covar_mut!(
            for<'lend> |state: &'lend mut i32| -> Result<Option<i32>, String> {
                *state += 1;
                if *state <= 5 {
                    Ok(Some(*state))
                } else {
                    Ok(None)
                }
            }
        ),
    )
    .map_while(covar_mut!(
        for<'lend> |x: i32| -> Result<Option<i32>, String> {
            Ok(if x < 4 { Some(x * 2) } else { None })
        }
    ));

    // Doubles elements while < 4: 2, 4, 6 — then 4 >= 4, stops
    assert_eq!(lender.next().unwrap(), Some(2));
    assert_eq!(lender.next().unwrap(), Some(4));
    assert_eq!(lender.next().unwrap(), Some(6));
    assert_eq!(lender.next().unwrap(), None);
}

#[test]
fn test_fallible_mutate_basic() {
    use lender::from_fallible_fn;

    let mut observed = Vec::new();
    let lender = from_fallible_fn(
        0,
        covar_mut!(
            for<'lend> |state: &'lend mut i32| -> Result<Option<i32>, String> {
                *state += 1;
                if *state <= 3 {
                    Ok(Some(*state))
                } else {
                    Ok(None)
                }
            }
        ),
    )
    .mutate(|x| {
        observed.push(*x);
        Ok(())
    });

    let mut result = Vec::new();
    lender
        .for_each(|x| {
            result.push(x);
            Ok(())
        })
        .unwrap();
    assert_eq!(result, vec![1, 2, 3]);
    assert_eq!(observed, vec![1, 2, 3]);
}

#[test]
fn test_fallible_scan_empty() {
    use lender::FallibleLender;

    let lender = lender::fallible_empty::<lender::fallible_lend!(i32), String>().scan(
        0,
        covar_mut!(
            for<'all> |(state, x): (&'all mut i32, i32)| -> Result<Option<i32>, String> {
                *state += x;
                Ok(Some(*state))
            }
        ),
    );

    assert_eq!(lender.count(), Ok(0));
}

#[test]
fn test_fallible_scan_error_in_source() {
    let mut lender = ErrorAtLender::new(vec![1, 2, 3, 4, 5], 2).scan(
        0,
        covar_mut!(
            for<'all> |(state, x): (&'all mut i32, &'all i32)| -> Result<Option<i32>, String> {
                *state += *x;
                Ok(Some(*state))
            }
        ),
    );

    assert_eq!(lender.next().unwrap(), Some(1)); // 0 + 1
    assert_eq!(lender.next().unwrap(), Some(3)); // 1 + 2
    assert!(lender.next().is_err()); // error at index 2
}

#[test]
fn test_fallible_scan_error_in_closure() {
    use lender::from_fallible_fn;

    let mut lender = from_fallible_fn(
        0,
        covar_mut!(
            for<'all> |state: &'all mut i32| -> Result<Option<i32>, String> {
                *state += 1;
                if *state <= 5 {
                    Ok(Some(*state))
                } else {
                    Ok(None)
                }
            }
        ),
    )
    .scan(
        0,
        covar_mut!(
            for<'all> |(state, x): (&'all mut i32, i32)| -> Result<Option<i32>, String> {
                *state += x;
                if *state > 5 {
                    Err("sum too large".to_string())
                } else {
                    Ok(Some(*state))
                }
            }
        ),
    );

    assert_eq!(lender.next().unwrap(), Some(1)); // sum = 1
    assert_eq!(lender.next().unwrap(), Some(3)); // sum = 3
    assert!(lender.next().is_err()); // sum = 6 > 5
}

#[test]
fn test_fallible_map_while_empty() {
    use lender::FallibleLender;

    let lender = lender::fallible_empty::<lender::fallible_lend!(i32), String>().map_while(
        covar_mut!(for<'lend> |x: i32| -> Result<Option<i32>, String> { Ok(Some(x * 2)) }),
    );

    assert_eq!(lender.count(), Ok(0));
}

#[test]
fn test_fallible_map_while_error_in_source() {
    let mut lender = ErrorAtLender::new(vec![1, 2, 3, 4, 5], 2).map_while(covar_mut!(
        for<'lend> |x: &'lend i32| -> Result<Option<i32>, String> { Ok(Some(*x * 2)) }
    ));

    assert_eq!(lender.next().unwrap(), Some(2));
    assert_eq!(lender.next().unwrap(), Some(4));
    assert!(lender.next().is_err()); // error at index 2
}

#[test]
fn test_fallible_map_while_error_in_closure() {
    use lender::from_fallible_fn;

    let mut lender = from_fallible_fn(
        0,
        covar_mut!(
            for<'lend> |state: &'lend mut i32| -> Result<Option<i32>, String> {
                *state += 1;
                if *state <= 5 {
                    Ok(Some(*state))
                } else {
                    Ok(None)
                }
            }
        ),
    )
    .map_while(covar_mut!(
        for<'lend> |x: i32| -> Result<Option<i32>, String> {
            if x > 2 {
                Err("value too large".to_string())
            } else {
                Ok(Some(x * 10))
            }
        }
    ));

    assert_eq!(lender.next().unwrap(), Some(10)); // 1 * 10
    assert_eq!(lender.next().unwrap(), Some(20)); // 2 * 10
    assert!(lender.next().is_err()); // 3 > 2, error
}

#[test]
fn test_fallible_chunky_specific() {
    // chunky() requires ExactSizeFallibleLender, so use VecFallibleLender
    let mut chunky = VecFallibleLender::new(vec![1, 2, 3, 4, 5, 6]).chunky(2);

    // First chunk: [1, 2]
    let mut chunk = chunky.next().unwrap().unwrap();
    assert_eq!(chunk.next().unwrap(), Some(&1));
    assert_eq!(chunk.next().unwrap(), Some(&2));
    assert_eq!(chunk.next().unwrap(), None);

    // Second chunk: [3, 4]
    let mut chunk = chunky.next().unwrap().unwrap();
    assert_eq!(chunk.next().unwrap(), Some(&3));
    assert_eq!(chunk.next().unwrap(), Some(&4));
    assert_eq!(chunk.next().unwrap(), None);

    // Third chunk: [5, 6]
    let mut chunk = chunky.next().unwrap().unwrap();
    assert_eq!(chunk.next().unwrap(), Some(&5));
    assert_eq!(chunk.next().unwrap(), Some(&6));
    assert_eq!(chunk.next().unwrap(), None);

    // Exhausted
    assert!(chunky.next().unwrap().is_none());
}

// ============================================================================
// Error propagation through adapter chains
// ============================================================================

#[test]
fn test_error_propagation_filter() {
    let mut lender = ErrorAtLender::new(vec![1, 2, 3, 4, 5], 2).filter(|x| Ok(**x > 0));
    assert_eq!(lender.next().unwrap(), Some(&1));
    assert_eq!(lender.next().unwrap(), Some(&2));
    // Index 2 errors
    assert_eq!(lender.next().unwrap_err(), "error at index 2");
}

#[test]
fn test_error_propagation_map() {
    let mut lender = ErrorAtLender::new(vec![10, 20, 30, 40], 1).map(covar_mut!(
        for<'lend> |x: &'lend i32| -> Result<i32, String> { Ok(*x * 2) }
    ));
    assert_eq!(lender.next().unwrap(), Some(20));
    // Index 1 errors
    assert_eq!(lender.next().unwrap_err(), "error at index 1");
}

#[test]
fn test_error_propagation_enumerate() {
    let mut lender = ErrorAtLender::new(vec![10, 20, 30], 1).enumerate();
    assert_eq!(lender.next().unwrap(), Some((0, &10)));
    // Index 1 errors
    assert_eq!(lender.next().unwrap_err(), "error at index 1");
}

#[test]
fn test_error_propagation_skip() {
    // Error at index 1, but we skip(2) — the error occurs during skip
    let mut lender = ErrorAtLender::new(vec![1, 2, 3, 4, 5], 1).skip(2);
    assert_eq!(lender.next().unwrap_err(), "error at index 1");
}

#[test]
fn test_error_propagation_take() {
    let mut lender = ErrorAtLender::new(vec![1, 2, 3, 4, 5], 2).take(4);
    assert_eq!(lender.next().unwrap(), Some(&1));
    assert_eq!(lender.next().unwrap(), Some(&2));
    // Index 2 errors
    assert_eq!(lender.next().unwrap_err(), "error at index 2");
}

#[test]
fn test_error_propagation_chain() {
    // Use two ErrorAtLenders (same error type) to test chain.
    let a = ErrorAtLender::new(vec![1, 2], 10); // no error
    let b = ErrorAtLender::new(vec![3, 4, 5], 0); // errors immediately
    let mut lender = a.chain(b);
    assert_eq!(lender.next().unwrap(), Some(&1));
    assert_eq!(lender.next().unwrap(), Some(&2));
    // Second lender errors at index 0
    assert_eq!(lender.next().unwrap_err(), "error at index 0");
}

#[test]
fn test_error_propagation_zip() {
    let a = ErrorAtLender::new(vec![1, 2, 3], 10); // no error
    let b = ErrorAtLender::new(vec![10, 20, 30], 1); // errors at index 1
    let mut lender = a.zip(b);
    let (x, y) = lender.next().unwrap().unwrap();
    assert_eq!(*x, 1);
    assert_eq!(*y, 10);
    // b errors at index 1
    assert_eq!(lender.next().unwrap_err(), "error at index 1");
}

#[test]
fn test_error_propagation_fold() {
    let lender = ErrorAtLender::new(vec![1, 2, 3, 4, 5], 3);
    let result = lender.fold(0, |acc, x| Ok(acc + *x));
    // Should get error at index 3, after accumulating 1+2+3=6
    assert_eq!(result.unwrap_err(), "error at index 3");
}

#[test]
fn test_error_propagation_count() {
    let lender = ErrorAtLender::new(vec![1, 2, 3, 4, 5], 2);
    let result = lender.count();
    assert_eq!(result.unwrap_err(), "error at index 2");
}

#[test]
fn test_error_propagation_for_each() {
    let lender = ErrorAtLender::new(vec![1, 2, 3], 1);
    let mut seen = Vec::new();
    let result = lender.for_each(|x| {
        seen.push(*x);
        Ok(())
    });
    assert_eq!(seen, vec![1]);
    assert_eq!(result.unwrap_err(), "error at index 1");
}

// ============================================================================
// Multi-adapter composition tests (fallible)
// ============================================================================

#[test]
fn test_fallible_compose_filter_map_fold() {
    use lender::from_fallible_fn;

    let result = from_fallible_fn(
        0,
        covar_mut!(
            for<'lend> |state: &'lend mut i32| -> Result<Option<i32>, String> {
                *state += 1;
                if *state <= 6 {
                    Ok(Some(*state))
                } else {
                    Ok(None)
                }
            }
        ),
    )
    .filter(|x| Ok(*x % 2 == 0))
    .map(covar_mut!(for<'lend> |x: i32| -> Result<i32, String> {
        Ok(x * 10)
    }))
    .fold(0, |acc, x| Ok(acc + x));

    // Even elements: 2, 4, 6; mapped to 20, 40, 60; sum = 120
    assert_eq!(result.unwrap(), 120);
}

#[test]
fn test_fallible_compose_skip_take() {
    use lender::from_fallible_fn;

    let mut lender = from_fallible_fn(
        0,
        covar_mut!(
            for<'lend> |state: &'lend mut i32| -> Result<Option<i32>, String> {
                *state += 1;
                if *state <= 10 {
                    Ok(Some(*state))
                } else {
                    Ok(None)
                }
            }
        ),
    )
    .skip(3)
    .take(2);

    assert_eq!(lender.next().unwrap(), Some(4));
    assert_eq!(lender.next().unwrap(), Some(5));
    assert_eq!(lender.next().unwrap(), None);
}

#[test]
fn test_fallible_compose_error_through_chain() {
    // Error in second half of a chain, through a filter
    let a = ErrorAtLender::new(vec![1, 2, 3], 10); // no error
    let b = ErrorAtLender::new(vec![4, 5, 6], 1); // errors at index 1
    let mut lender = a.chain(b).filter(|x| Ok(**x > 0));
    assert_eq!(lender.next().unwrap(), Some(&1));
    assert_eq!(lender.next().unwrap(), Some(&2));
    assert_eq!(lender.next().unwrap(), Some(&3));
    assert_eq!(lender.next().unwrap(), Some(&4));
    // b errors at index 1
    assert_eq!(lender.next().unwrap_err(), "error at index 1");
}

// ============================================================================
// is_partitioned and collect_into (fallible)
// ============================================================================

#[test]
fn test_fallible_is_partitioned_true() {
    use lender::from_fallible_fn;

    let result = from_fallible_fn(
        0,
        covar_mut!(
            for<'lend> |state: &'lend mut i32| -> Result<Option<i32>, String> {
                *state += 1;
                if *state <= 6 {
                    Ok(Some(*state))
                } else {
                    Ok(None)
                }
            }
        ),
    )
    .is_partitioned(|x| Ok(x <= 3));

    // 1,2,3 are true, then 4,5,6 are false — partitioned
    assert!(result.unwrap());
}

#[test]
fn test_fallible_is_partitioned_false() {
    use lender::from_fallible_fn;

    let result = from_fallible_fn(
        0,
        covar_mut!(
            for<'lend> |state: &'lend mut i32| -> Result<Option<i32>, String> {
                *state += 1;
                if *state <= 4 {
                    Ok(Some(*state))
                } else {
                    Ok(None)
                }
            }
        ),
    )
    .is_partitioned(|x| Ok(x % 2 == 0));

    // 1(f), 2(t), 3(f), 4(t) — not partitioned
    assert!(!result.unwrap());
}

#[test]
fn test_fallible_collect_into() {
    // collect_into requires ExtendLender for NonFallibleAdapter.
    // VecFallibleLender yields &i32, so NonFallibleAdapter yields &i32.
    let lender = VecFallibleLender::new(vec![1, 2, 3]);
    let mut result = I32Collector(Vec::new());
    let out = lender.collect_into(&mut result);
    assert!(out.is_ok());
    assert_eq!(result.0, vec![1, 2, 3]);
}

#[test]
fn test_fallible_collect_into_error() {
    // ErrorAtLender yields &i32, so NonFallibleAdapter yields &i32.
    let lender = ErrorAtLender::new(vec![1, 2, 3, 4, 5], 2);
    let mut result = I32Collector(Vec::new());
    let out = lender.collect_into(&mut result);
    assert!(out.is_err());
    let (collection, err) = out.unwrap_err();
    assert_eq!(collection.0, vec![1, 2]); // collected before error
    assert_eq!(err, "error at index 2");
}

// ============================================================================
// Fallible iterator adapters (cloned, copied, iter, map_into_iter)
// ============================================================================

#[test]
fn test_fallible_cloned_basic() {
    use fallible_iterator::FallibleIterator;

    let lender = VecFallibleLender::new(vec![1, 2, 3]);
    let mut cloned = lender.cloned();

    assert_eq!(cloned.next().unwrap(), Some(1));
    assert_eq!(cloned.next().unwrap(), Some(2));
    assert_eq!(cloned.next().unwrap(), Some(3));
    assert_eq!(cloned.next().unwrap(), None);
}

#[test]
fn test_fallible_cloned_double_ended() {
    use fallible_iterator::{DoubleEndedFallibleIterator, FallibleIterator};

    let lender = VecFallibleLender::new(vec![1, 2, 3]);
    let mut cloned = lender.cloned();

    assert_eq!(cloned.next_back().unwrap(), Some(3));
    assert_eq!(cloned.next().unwrap(), Some(1));
    assert_eq!(cloned.next_back().unwrap(), Some(2));
    assert_eq!(cloned.next().unwrap(), None);
}

#[test]
fn test_fallible_cloned_size_hint() {
    use fallible_iterator::FallibleIterator;

    let lender = VecFallibleLender::new(vec![1, 2, 3]);
    let cloned = lender.cloned();

    assert_eq!(cloned.size_hint(), (3, Some(3)));
}

#[test]
fn test_fallible_cloned_error_propagation() {
    use fallible_iterator::FallibleIterator;

    let lender = ErrorAtLender::new(vec![1, 2, 3, 4], 2);
    let mut cloned = lender.cloned();

    assert_eq!(cloned.next().unwrap(), Some(1));
    assert_eq!(cloned.next().unwrap(), Some(2));
    assert!(cloned.next().is_err());
}

#[test]
fn test_fallible_copied_basic() {
    use fallible_iterator::FallibleIterator;

    let lender = VecFallibleLender::new(vec![10, 20, 30]);
    let mut copied = lender.copied();

    assert_eq!(copied.next().unwrap(), Some(10));
    assert_eq!(copied.next().unwrap(), Some(20));
    assert_eq!(copied.next().unwrap(), Some(30));
    assert_eq!(copied.next().unwrap(), None);
}

#[test]
fn test_fallible_copied_double_ended() {
    use fallible_iterator::{DoubleEndedFallibleIterator, FallibleIterator};

    let lender = VecFallibleLender::new(vec![10, 20, 30]);
    let mut copied = lender.copied();

    assert_eq!(copied.next_back().unwrap(), Some(30));
    assert_eq!(copied.next().unwrap(), Some(10));
    assert_eq!(copied.next_back().unwrap(), Some(20));
    assert_eq!(copied.next().unwrap(), None);
}

#[test]
fn test_fallible_copied_size_hint() {
    use fallible_iterator::FallibleIterator;

    let lender = VecFallibleLender::new(vec![10, 20, 30]);
    let copied = lender.copied();

    assert_eq!(copied.size_hint(), (3, Some(3)));
}

#[test]
fn test_fallible_copied_error_propagation() {
    use fallible_iterator::FallibleIterator;

    let lender = ErrorAtLender::new(vec![10, 20, 30, 40], 1);
    let mut copied = lender.copied();

    assert_eq!(copied.next().unwrap(), Some(10));
    assert!(copied.next().is_err());
}

#[test]
fn test_fallible_map_into_iter_basic() {
    use fallible_iterator::FallibleIterator;

    let lender = VecFallibleLender::new(vec![1, 2, 3]);
    let mut iter = lender.map_into_iter(|x: &i32| Ok(*x * 2));

    assert_eq!(iter.next().unwrap(), Some(2));
    assert_eq!(iter.next().unwrap(), Some(4));
    assert_eq!(iter.next().unwrap(), Some(6));
    assert_eq!(iter.next().unwrap(), None);
}

#[test]
fn test_fallible_map_into_iter_double_ended() {
    use fallible_iterator::{DoubleEndedFallibleIterator, FallibleIterator};

    let lender = VecFallibleLender::new(vec![1, 2, 3]);
    let mut iter = lender.map_into_iter(|x: &i32| Ok(*x * 2));

    assert_eq!(iter.next_back().unwrap(), Some(6));
    assert_eq!(iter.next().unwrap(), Some(2));
    assert_eq!(iter.next_back().unwrap(), Some(4));
    assert_eq!(iter.next().unwrap(), None);
}

#[test]
fn test_fallible_map_into_iter_size_hint() {
    use fallible_iterator::FallibleIterator;

    let lender = VecFallibleLender::new(vec![1, 2, 3]);
    let iter = lender.map_into_iter(|x: &i32| Ok(*x * 2));

    assert_eq!(iter.size_hint(), (3, Some(3)));
}

#[test]
fn test_fallible_map_into_iter_closure_error() {
    use fallible_iterator::FallibleIterator;

    // Use ErrorAtLender which has String error type to allow closure errors
    let lender = ErrorAtLender::new(vec![1, 2, 3, 4, 5, 6], 100); // error_at beyond range
    let mut iter = lender.map_into_iter(|x: &i32| {
        if *x == 3 {
            Err::<i32, _>("closure error".to_string())
        } else {
            Ok(*x * 2)
        }
    });

    assert_eq!(iter.next().unwrap(), Some(2));
    assert_eq!(iter.next().unwrap(), Some(4));
    assert!(iter.next().is_err()); // closure error at x == 3
}

#[test]
fn test_fallible_map_into_iter_lender_error() {
    use fallible_iterator::FallibleIterator;

    let lender = ErrorAtLender::new(vec![1, 2, 3, 4], 2);
    let mut iter = lender.map_into_iter(|x: &i32| Ok::<_, String>(*x * 2));

    assert_eq!(iter.next().unwrap(), Some(2));
    assert_eq!(iter.next().unwrap(), Some(4));
    assert!(iter.next().is_err());
}

// ============================================================================
// Cloned: fold, count, nth, rfold
// ============================================================================

#[test]
fn test_fallible_cloned_fold() {
    let lender = VecFallibleLender::new(vec![1, 2, 3, 4]);
    let sum = lender.cloned().fold(0, |acc, x| Ok(acc + x)).unwrap();
    assert_eq!(sum, 10);
}

#[test]
fn test_fallible_cloned_fold_empty() {
    let lender = VecFallibleLender::new(vec![]);
    let sum = lender.cloned().fold(0, |acc, x: i32| Ok(acc + x)).unwrap();
    assert_eq!(sum, 0);
}

#[test]
fn test_fallible_cloned_fold_error() {
    let lender = ErrorAtLender::new(vec![1, 2, 3, 4], 2);
    let result = lender.cloned().fold(0, |acc, x| Ok(acc + x));
    assert!(result.is_err());
}

#[test]
fn test_fallible_cloned_count() {
    let lender = VecFallibleLender::new(vec![1, 2, 3]);
    assert_eq!(lender.cloned().count().unwrap(), 3);
}

#[test]
fn test_fallible_cloned_count_empty() {
    let lender = VecFallibleLender::new(vec![]);
    assert_eq!(lender.cloned().count().unwrap(), 0);
}

#[test]
fn test_fallible_cloned_count_error() {
    let lender = ErrorAtLender::new(vec![1, 2, 3], 1);
    assert!(lender.cloned().count().is_err());
}

#[test]
fn test_fallible_cloned_nth() {
    let lender = VecFallibleLender::new(vec![10, 20, 30, 40]);
    let mut cloned = lender.cloned();
    assert_eq!(cloned.nth(2).unwrap(), Some(30));
    assert_eq!(cloned.next().unwrap(), Some(40));
    assert_eq!(cloned.next().unwrap(), None);
}

#[test]
fn test_fallible_cloned_nth_past_end() {
    let lender = VecFallibleLender::new(vec![1, 2]);
    let mut cloned = lender.cloned();
    assert_eq!(cloned.nth(5).unwrap(), None);
}

#[test]
fn test_fallible_cloned_nth_error() {
    let lender = ErrorAtLender::new(vec![1, 2, 3, 4], 1);
    let mut cloned = lender.cloned();
    assert!(cloned.nth(2).is_err());
}

#[test]
fn test_fallible_cloned_rfold() {
    let lender = VecFallibleLender::new(vec![1, 2, 3]);
    let result = lender
        .cloned()
        .rfold(Vec::new(), |mut acc, x| {
            acc.push(x);
            Ok(acc)
        })
        .unwrap();
    assert_eq!(result, vec![3, 2, 1]);
}

#[test]
fn test_fallible_cloned_rfold_empty() {
    let lender = VecFallibleLender::new(vec![]);
    let result = lender
        .cloned()
        .rfold(Vec::new(), |mut acc, x: i32| {
            acc.push(x);
            Ok(acc)
        })
        .unwrap();
    assert!(result.is_empty());
}

// ============================================================================
// Copied: fold, count, nth, rfold
// ============================================================================

#[test]
fn test_fallible_copied_fold() {
    let lender = VecFallibleLender::new(vec![1, 2, 3, 4]);
    let sum = lender.copied().fold(0, |acc, x| Ok(acc + x)).unwrap();
    assert_eq!(sum, 10);
}

#[test]
fn test_fallible_copied_fold_empty() {
    let lender = VecFallibleLender::new(vec![]);
    let sum = lender.copied().fold(0, |acc, x: i32| Ok(acc + x)).unwrap();
    assert_eq!(sum, 0);
}

#[test]
fn test_fallible_copied_fold_error() {
    let lender = ErrorAtLender::new(vec![1, 2, 3, 4], 2);
    let result = lender.copied().fold(0, |acc, x| Ok(acc + x));
    assert!(result.is_err());
}

#[test]
fn test_fallible_copied_count() {
    let lender = VecFallibleLender::new(vec![1, 2, 3]);
    assert_eq!(lender.copied().count().unwrap(), 3);
}

#[test]
fn test_fallible_copied_count_empty() {
    let lender = VecFallibleLender::new(vec![]);
    assert_eq!(lender.copied().count().unwrap(), 0);
}

#[test]
fn test_fallible_copied_count_error() {
    let lender = ErrorAtLender::new(vec![1, 2, 3], 1);
    assert!(lender.copied().count().is_err());
}

#[test]
fn test_fallible_copied_nth() {
    let lender = VecFallibleLender::new(vec![10, 20, 30, 40]);
    let mut copied = lender.copied();
    assert_eq!(copied.nth(2).unwrap(), Some(30));
    assert_eq!(copied.next().unwrap(), Some(40));
    assert_eq!(copied.next().unwrap(), None);
}

#[test]
fn test_fallible_copied_nth_past_end() {
    let lender = VecFallibleLender::new(vec![1, 2]);
    let mut copied = lender.copied();
    assert_eq!(copied.nth(5).unwrap(), None);
}

#[test]
fn test_fallible_copied_nth_error() {
    let lender = ErrorAtLender::new(vec![1, 2, 3, 4], 1);
    let mut copied = lender.copied();
    assert!(copied.nth(2).is_err());
}

#[test]
fn test_fallible_copied_rfold() {
    let lender = VecFallibleLender::new(vec![1, 2, 3]);
    let result = lender
        .copied()
        .rfold(Vec::new(), |mut acc, x| {
            acc.push(x);
            Ok(acc)
        })
        .unwrap();
    assert_eq!(result, vec![3, 2, 1]);
}

#[test]
fn test_fallible_copied_rfold_empty() {
    let lender = VecFallibleLender::new(vec![]);
    let result = lender
        .copied()
        .rfold(Vec::new(), |mut acc, x: i32| {
            acc.push(x);
            Ok(acc)
        })
        .unwrap();
    assert!(result.is_empty());
}

// ============================================================================
// Owned: fold, count, nth, next_back, rfold
// (Uses into_iter().into_lender().into_fallible() for owned values)
// ============================================================================

#[test]
fn test_fallible_owned_fold() {
    let fallible = vec![1, 2, 3, 4].into_iter().into_lender().into_fallible();
    let sum = fallible.owned().fold(0, |acc, x| Ok(acc + x)).unwrap();
    assert_eq!(sum, 10);
}

#[test]
fn test_fallible_owned_fold_empty() {
    let fallible = Vec::<i32>::new().into_iter().into_lender().into_fallible();
    let sum = fallible.owned().fold(0, |acc, x: i32| Ok(acc + x)).unwrap();
    assert_eq!(sum, 0);
}

#[test]
fn test_fallible_owned_fold_error() {
    let data: Vec<Result<i32, String>> = vec![Ok(1), Ok(2), Err("error".into()), Ok(4)];
    let fallible = data.into_iter().into_lender().convert::<String>();
    let result = fallible.owned().fold(0, |acc, x| Ok(acc + x));
    assert!(result.is_err());
}

#[test]
fn test_fallible_owned_count() {
    let fallible = vec![1, 2, 3].into_iter().into_lender().into_fallible();
    assert_eq!(fallible.owned().count().unwrap(), 3);
}

#[test]
fn test_fallible_owned_count_empty() {
    let fallible = Vec::<i32>::new().into_iter().into_lender().into_fallible();
    assert_eq!(fallible.owned().count().unwrap(), 0);
}

#[test]
fn test_fallible_owned_count_error() {
    let data: Vec<Result<i32, String>> = vec![Ok(1), Err("error".into()), Ok(3)];
    let fallible = data.into_iter().into_lender().convert::<String>();
    assert!(fallible.owned().count().is_err());
}

#[test]
fn test_fallible_owned_nth() {
    let fallible = vec![10, 20, 30, 40]
        .into_iter()
        .into_lender()
        .into_fallible();
    let mut owned = fallible.owned();
    assert_eq!(owned.nth(2).unwrap(), Some(30));
    assert_eq!(owned.next().unwrap(), Some(40));
    assert_eq!(owned.next().unwrap(), None);
}

#[test]
fn test_fallible_owned_nth_past_end() {
    let fallible = vec![1, 2].into_iter().into_lender().into_fallible();
    let mut owned = fallible.owned();
    assert_eq!(owned.nth(5).unwrap(), None);
}

#[test]
fn test_fallible_owned_nth_error() {
    let data: Vec<Result<i32, String>> = vec![Ok(1), Err("error".into()), Ok(3), Ok(4)];
    let fallible = data.into_iter().into_lender().convert::<String>();
    let mut owned = fallible.owned();
    assert!(owned.nth(2).is_err());
}

#[test]
fn test_fallible_owned_next_back() {
    let fallible = vec![1, 2, 3].into_iter().into_lender().into_fallible();
    let mut owned = fallible.owned();
    assert_eq!(owned.next_back().unwrap(), Some(3));
    assert_eq!(owned.next().unwrap(), Some(1));
    assert_eq!(owned.next_back().unwrap(), Some(2));
    assert_eq!(owned.next().unwrap(), None);
}

#[test]
fn test_fallible_owned_rfold() {
    let fallible = vec![1, 2, 3].into_iter().into_lender().into_fallible();
    let result = fallible
        .owned()
        .rfold(Vec::new(), |mut acc, x| {
            acc.push(x);
            Ok(acc)
        })
        .unwrap();
    assert_eq!(result, vec![3, 2, 1]);
}

#[test]
fn test_fallible_owned_rfold_empty() {
    let fallible = Vec::<i32>::new().into_iter().into_lender().into_fallible();
    let result = fallible
        .owned()
        .rfold(Vec::new(), |mut acc, x: i32| {
            acc.push(x);
            Ok(acc)
        })
        .unwrap();
    assert!(result.is_empty());
}

// ============================================================================
// MapIntoIter: nth, fold, rfold
// ============================================================================

#[test]
fn test_fallible_map_into_iter_nth() {
    let lender = VecFallibleLender::new(vec![10, 20, 30, 40, 50]);
    let mut iter = lender.map_into_iter(|x: &i32| Ok(*x * 2));
    assert_eq!(iter.nth(2).unwrap(), Some(60));
    assert_eq!(iter.next().unwrap(), Some(80));
    assert_eq!(iter.next().unwrap(), Some(100));
    assert_eq!(iter.next().unwrap(), None);
}

#[test]
fn test_fallible_map_into_iter_nth_past_end() {
    let lender = VecFallibleLender::new(vec![1, 2]);
    let mut iter = lender.map_into_iter(|x: &i32| Ok(*x));
    assert_eq!(iter.nth(10).unwrap(), None);
}

#[test]
fn test_fallible_map_into_iter_nth_error_in_lender() {
    let lender = ErrorAtLender::new(vec![1, 2, 3, 4], 1);
    let mut iter = lender.map_into_iter(|x: &i32| Ok::<_, String>(*x));
    // nth(2) needs to advance past index 1 which errors
    assert!(iter.nth(2).is_err());
}

#[test]
fn test_fallible_map_into_iter_nth_error_in_closure() {
    let lender = ErrorAtLender::new(vec![1, 2, 3, 4, 5], 100);
    let mut iter = lender.map_into_iter(|x: &i32| {
        if *x == 3 {
            Err("closure error".to_string())
        } else {
            Ok(*x * 2)
        }
    });
    // nth(2) lands on element 3, which the closure errors on
    assert!(iter.nth(2).is_err());
}

#[test]
fn test_fallible_map_into_iter_fold() {
    let lender = VecFallibleLender::new(vec![1, 2, 3, 4]);
    let sum = lender
        .map_into_iter(|x: &i32| Ok(*x * 2))
        .fold(0, |acc, x| Ok(acc + x))
        .unwrap();
    assert_eq!(sum, 20); // (1+2+3+4)*2 = 20
}

#[test]
fn test_fallible_map_into_iter_fold_empty() {
    let lender = VecFallibleLender::new(vec![]);
    let sum = lender
        .map_into_iter(|x: &i32| Ok(*x))
        .fold(0, |acc, x| Ok(acc + x))
        .unwrap();
    assert_eq!(sum, 0);
}

#[test]
fn test_fallible_map_into_iter_fold_error_in_lender() {
    let lender = ErrorAtLender::new(vec![1, 2, 3, 4], 2);
    let result = lender
        .map_into_iter(|x: &i32| Ok::<_, String>(*x))
        .fold(0, |acc, x| Ok(acc + x));
    assert!(result.is_err());
}

#[test]
fn test_fallible_map_into_iter_fold_error_in_closure() {
    let lender = ErrorAtLender::new(vec![1, 2, 3, 4, 5], 100);
    let result = lender
        .map_into_iter(|x: &i32| {
            if *x == 3 {
                Err("closure error".to_string())
            } else {
                Ok(*x * 2)
            }
        })
        .fold(0, |acc, x| Ok(acc + x));
    assert!(result.is_err());
}

#[test]
fn test_fallible_map_into_iter_rfold() {
    let lender = VecFallibleLender::new(vec![1, 2, 3]);
    let result = lender
        .map_into_iter(|x: &i32| Ok(*x * 10))
        .rfold(Vec::new(), |mut acc, x| {
            acc.push(x);
            Ok(acc)
        })
        .unwrap();
    assert_eq!(result, vec![30, 20, 10]);
}

#[test]
fn test_fallible_map_into_iter_rfold_empty() {
    let lender = VecFallibleLender::new(vec![]);
    let result = lender
        .map_into_iter(|x: &i32| Ok(*x))
        .rfold(Vec::new(), |mut acc, x: i32| {
            acc.push(x);
            Ok(acc)
        })
        .unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_fallible_map_into_iter_rfold_error_in_closure() {
    // Use convert to get a DoubleEndedFallibleLender with String error
    let data: Vec<Result<i32, String>> = vec![Ok(1), Ok(2), Ok(3), Ok(4), Ok(5)];
    let lender = data.into_iter().into_lender().convert::<String>();
    let result = lender
        .map_into_iter(|x: i32| {
            if x == 3 {
                Err("closure error".to_string())
            } else {
                Ok(x)
            }
        })
        .rfold(Vec::new(), |mut acc, x| {
            acc.push(x);
            Ok(acc)
        });
    assert!(result.is_err());
}

// ============================================================================
// Chunky: nth, try_fold, fold
// ============================================================================

#[test]
fn test_fallible_chunky_nth() {
    let mut chunky = VecFallibleLender::new(vec![1, 2, 3, 4, 5, 6]).chunky(2);
    // Skip 1 chunk, get second chunk [3, 4]
    let mut chunk = chunky.nth(1).unwrap().unwrap();
    assert_eq!(chunk.next().unwrap(), Some(&3));
    assert_eq!(chunk.next().unwrap(), Some(&4));
    assert_eq!(chunk.next().unwrap(), None);
    // Third chunk [5, 6] should still be available
    let mut chunk = chunky.next().unwrap().unwrap();
    assert_eq!(chunk.next().unwrap(), Some(&5));
    assert_eq!(chunk.next().unwrap(), Some(&6));
}

#[test]
fn test_fallible_chunky_nth_zero() {
    let mut chunky = VecFallibleLender::new(vec![1, 2, 3, 4]).chunky(2);
    // nth(0) is equivalent to next()
    let mut chunk = chunky.nth(0).unwrap().unwrap();
    assert_eq!(chunk.next().unwrap(), Some(&1));
    assert_eq!(chunk.next().unwrap(), Some(&2));
}

#[test]
fn test_fallible_chunky_nth_past_end() {
    let mut chunky = VecFallibleLender::new(vec![1, 2, 3, 4]).chunky(2);
    // Only 2 chunks, nth(5) should be None
    assert!(chunky.nth(5).unwrap().is_none());
    // Exhausted
    assert!(chunky.next().unwrap().is_none());
}

#[test]
fn test_fallible_chunky_nth_exact_end() {
    let mut chunky = VecFallibleLender::new(vec![1, 2, 3, 4, 5, 6]).chunky(2);
    // 3 chunks, nth(2) gets the last one
    let mut chunk = chunky.nth(2).unwrap().unwrap();
    assert_eq!(chunk.next().unwrap(), Some(&5));
    assert_eq!(chunk.next().unwrap(), Some(&6));
    assert!(chunky.next().unwrap().is_none());
}

#[test]
fn test_fallible_chunky_size_hint() {
    let chunky = VecFallibleLender::new(vec![1, 2, 3, 4, 5, 6]).chunky(2);
    assert_eq!(chunky.size_hint(), (3, Some(3)));
}

#[test]
fn test_fallible_chunky_count() {
    let chunky = VecFallibleLender::new(vec![1, 2, 3, 4, 5, 6]).chunky(2);
    assert_eq!(chunky.count().unwrap(), 3);
}

#[test]
fn test_fallible_chunky_try_fold() {
    let mut chunky = VecFallibleLender::new(vec![1, 2, 3, 4, 5, 6]).chunky(2);
    let result: Result<Result<i32, ()>, _> = chunky.try_fold(0, |acc, mut chunk| {
        let mut sum = acc;
        while let Some(x) = chunk.next()? {
            sum += x;
        }
        Ok(Ok(sum))
    });
    assert_eq!(result, Ok(Ok(21))); // 1+2+3+4+5+6 = 21
}

#[test]
fn test_fallible_chunky_try_fold_break() {
    let mut chunky = VecFallibleLender::new(vec![1, 2, 3, 4, 5, 6]).chunky(2);
    // Break after processing the first chunk
    let result: Result<Result<i32, i32>, _> = chunky.try_fold(0, |acc, mut chunk| {
        let mut sum = acc;
        while let Some(x) = chunk.next()? {
            sum += x;
        }
        if sum > 5 {
            Ok(Err(sum)) // break
        } else {
            Ok(Ok(sum)) // continue
        }
    });
    // First chunk: 1+2=3 (continue), second: 3+3+4=10 (break)
    assert_eq!(result, Ok(Err(10)));
}

#[test]
fn test_fallible_chunky_fold() {
    let chunky = VecFallibleLender::new(vec![1, 2, 3, 4, 5, 6]).chunky(2);
    let result = chunky.fold(0, |acc, mut chunk| {
        let mut sum = acc;
        while let Some(x) = chunk.next()? {
            sum += x;
        }
        Ok(sum)
    });
    assert_eq!(result, Ok(21));
}

#[test]
fn test_fallible_chunky_fold_empty() {
    let chunky = VecFallibleLender::new(vec![]).chunky(2);
    let result = chunky.fold(0, |acc, mut chunk| {
        let mut sum = acc;
        while let Some(x) = chunk.next()? {
            sum += x;
        }
        Ok(sum)
    });
    assert_eq!(result, Ok(0));
}

// ============================================================================
// FilterMap: count, next_back, size_hint
// ============================================================================

#[test]
fn test_fallible_filter_map_count() {
    use lender::from_fallible_fn;

    let count = from_fallible_fn(
        0,
        covar_mut!(
            for<'lend> |state: &'lend mut i32| -> Result<Option<i32>, String> {
                *state += 1;
                if *state <= 6 {
                    Ok(Some(*state))
                } else {
                    Ok(None)
                }
            }
        ),
    )
    .filter_map(covar_mut!(
        for<'lend> |x: i32| -> Result<Option<i32>, String> {
            Ok(if x % 2 == 0 { Some(x) } else { None })
        }
    ))
    .count();
    assert_eq!(count, Ok(3)); // 2, 4, 6
}

#[test]
fn test_fallible_filter_map_count_empty() {
    use lender::from_fallible_fn;

    let count = from_fallible_fn(
        0,
        covar_mut!(for<'lend> |_state: &'lend mut i32| -> Result<Option<i32>, String> { Ok(None) }),
    )
    .filter_map(covar_mut!(
        for<'lend> |x: i32| -> Result<Option<i32>, String> { Ok(Some(x)) }
    ))
    .count();
    assert_eq!(count, Ok(0));
}

#[test]
fn test_fallible_filter_map_count_all_filtered() {
    use lender::from_fallible_fn;

    let count = from_fallible_fn(
        0,
        covar_mut!(
            for<'lend> |state: &'lend mut i32| -> Result<Option<i32>, String> {
                *state += 1;
                if *state <= 5 {
                    Ok(Some(*state))
                } else {
                    Ok(None)
                }
            }
        ),
    )
    .filter_map(covar_mut!(
        for<'lend> |_x: i32| -> Result<Option<i32>, String> { Ok(None) }
    ))
    .count();
    assert_eq!(count, Ok(0));
}

#[test]
fn test_fallible_filter_map_count_error_in_source() {
    let count = ErrorAtLender::new(vec![1, 2, 3, 4, 5], 2)
        .filter_map(covar_mut!(
            for<'lend> |x: &'lend i32| -> Result<Option<i32>, String> { Ok(Some(*x)) }
        ))
        .count();
    assert!(count.is_err());
}

#[test]
fn test_fallible_filter_map_count_error_in_closure() {
    use lender::from_fallible_fn;

    let count = from_fallible_fn(
        0,
        covar_mut!(
            for<'lend> |state: &'lend mut i32| -> Result<Option<i32>, String> {
                *state += 1;
                if *state <= 5 {
                    Ok(Some(*state))
                } else {
                    Ok(None)
                }
            }
        ),
    )
    .filter_map(covar_mut!(
        for<'lend> |x: i32| -> Result<Option<i32>, String> {
            if x == 3 {
                Err("closure error".to_string())
            } else {
                Ok(Some(x))
            }
        }
    ))
    .count();
    assert!(count.is_err());
}

#[test]
fn test_fallible_filter_map_next_back() {
    let lender = VecFallibleLender::new(vec![1, 2, 3, 4, 5, 6]);
    let mut fm = lender.filter_map(covar_mut!(for<'lend> |x: &'lend i32| -> Result<
        Option<i32>,
        std::convert::Infallible,
    > {
        Ok(if *x % 2 == 0 { Some(*x * 10) } else { None })
    }));
    // next_back: 6(even->60), then 5(odd,skip), 4(even->40)
    assert_eq!(fm.next_back().unwrap(), Some(60));
    assert_eq!(fm.next_back().unwrap(), Some(40));
    assert_eq!(fm.next_back().unwrap(), Some(20));
    assert_eq!(fm.next_back().unwrap(), None);
}

#[test]
fn test_fallible_filter_map_size_hint() {
    let lender = VecFallibleLender::new(vec![1, 2, 3, 4, 5]);
    let fm = lender.filter_map(covar_mut!(for<'lend> |x: &'lend i32| -> Result<
        Option<i32>,
        std::convert::Infallible,
    > {
        Ok(if *x % 2 == 0 { Some(*x) } else { None })
    }));
    let (lo, hi) = fm.size_hint();
    assert_eq!(lo, 0);
    assert_eq!(hi, Some(5));
}
