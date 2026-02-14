//! Tests for fallible sources: empty, once, repeat, once_with, repeat_with

use ::lender::prelude::*;
use fallible_iterator::IteratorExt as _;

// ============================================================================
// Fallible sources
// ============================================================================

#[test]
fn fallible_empty() {
    use lender::{fallible_empty, fallible_lend};

    // Test basic fallible_empty lender
    let mut empty = fallible_empty::<fallible_lend!(i32), String>();
    assert!(empty.next().unwrap().is_none());
    assert!(empty.next().unwrap().is_none()); // Should continue returning None

    // Test that it's fused
    let mut empty_fused = fallible_empty::<fallible_lend!(i32), String>();
    for _ in 0..10 {
        assert!(empty_fused.next().unwrap().is_none());
    }

    // Test fold operation
    let sum: Result<i32, String> =
        fallible_empty::<fallible_lend!(i32), String>().fold(0, |acc, _x: i32| Ok(acc + 1));
    assert_eq!(sum, Ok(0)); // Should never iterate so result is 0

    // Test count
    let count: Result<usize, String> = fallible_empty::<fallible_lend!(i32), String>().count();
    assert_eq!(count, Ok(0));

    // Test with reference type
    let mut empty_ref = fallible_empty::<fallible_lend!(&'lend str), String>();
    assert!(empty_ref.next().unwrap().is_none());

    // FallibleEmpty should implement ExactSizeFallibleLender
    let empty_exact = fallible_empty::<fallible_lend!(i32), String>();
    assert_eq!(lender::ExactSizeFallibleLender::len(&empty_exact), 0);
    assert!(lender::ExactSizeFallibleLender::is_empty(&empty_exact));
}

#[test]
fn fallible_once() {
    use lender::{fallible_lend, fallible_once, fallible_once_err};

    // Test with value
    let mut once = fallible_once::<fallible_lend!(i32), String>(42);
    assert_eq!(once.next().unwrap(), Some(42));
    assert!(once.next().unwrap().is_none());
    assert!(once.next().unwrap().is_none()); // Should continue returning None (fused)

    // Test with error
    let mut once_err = fallible_once_err::<fallible_lend!(i32), _>("error".to_string());
    match once_err.next() {
        Err(e) => assert_eq!(e, "error"),
        Ok(_) => panic!("Expected error"),
    }
    // After an error, should return None
    assert!(once_err.next().unwrap().is_none());

    // Test fold with value
    let sum: Result<i32, String> =
        fallible_once::<fallible_lend!(i32), String>(10).fold(0, |acc, x| Ok(acc + x));
    assert_eq!(sum, Ok(10));

    // Test fold with error
    let sum_err: Result<i32, String> =
        fallible_once_err::<fallible_lend!(i32), _>("error".to_string())
            .fold(0, |acc, x: i32| Ok(acc + x));
    assert!(sum_err.is_err());

    // Test count with value
    let count: Result<usize, String> = fallible_once::<fallible_lend!(i32), String>(42).count();
    assert_eq!(count, Ok(1));

    // Test count with error
    let count_err: Result<usize, String> =
        fallible_once_err::<fallible_lend!(i32), _>("error".to_string()).count();
    assert!(count_err.is_err());

    // FallibleOnce should implement ExactSizeFallibleLender
    let once_exact = fallible_once::<fallible_lend!(i32), String>(42);
    assert_eq!(lender::ExactSizeFallibleLender::len(&once_exact), 1);
    assert!(!lender::ExactSizeFallibleLender::is_empty(&once_exact));
}

#[test]
fn fallible_repeat() {
    use lender::{fallible_lend, fallible_repeat, fallible_repeat_err};

    // Test with value
    let mut repeat = fallible_repeat::<fallible_lend!(i32), String>(42);
    assert_eq!(repeat.next().unwrap(), Some(42));
    assert_eq!(repeat.next().unwrap(), Some(42));
    assert_eq!(repeat.next().unwrap(), Some(42));
    // Should continue repeating
    for _ in 0..100 {
        assert_eq!(repeat.next().unwrap(), Some(42));
    }

    // Test with error
    let mut repeat_err = fallible_repeat_err::<fallible_lend!(i32), _>("error".to_string());
    match repeat_err.next() {
        Err(e) => assert_eq!(e, "error"),
        Ok(_) => panic!("Expected error"),
    }
    // Should continue to return the same error
    match repeat_err.next() {
        Err(e) => assert_eq!(e, "error"),
        Ok(_) => panic!("Expected error"),
    }

    // Test take with value - manually collect
    let mut collected = Vec::new();
    let result = fallible_repeat::<fallible_lend!(i32), String>(5)
        .take(3)
        .for_each(|x| {
            collected.push(x);
            Ok(())
        });
    assert!(result.is_ok());
    assert_eq!(collected, vec![5, 5, 5]);

    // Test take with error - should fail on first item
    let mut collected_err = Vec::new();
    let result_err = fallible_repeat_err::<fallible_lend!(i32), _>("error".to_string())
        .take(3)
        .for_each(|x| {
            collected_err.push(x);
            Ok(())
        });
    assert!(result_err.is_err());
    assert!(collected_err.is_empty()); // Should not have collected anything

    // size_hint should indicate infinite iterator
    let repeat_hint = fallible_repeat::<fallible_lend!(i32), String>(42);
    assert_eq!(repeat_hint.size_hint(), (usize::MAX, None));

    // FallibleRepeat should be double-ended (infinite both ways)
    let mut repeat_de = fallible_repeat::<fallible_lend!(i32), String>(7);
    assert_eq!(repeat_de.next_back().unwrap(), Some(7));
    assert_eq!(repeat_de.next_back().unwrap(), Some(7));
    assert_eq!(repeat_de.next().unwrap(), Some(7));
}

#[test]
fn fallible_once_with() {
    use lender::{covar_once, fallible_once_with, fallible_once_with_err};

    // Test with value from closure
    let mut once_with = fallible_once_with::<_, String, _>(
        42,
        covar_once!(for<'lend> |x: &'lend mut i32| -> i32 { *x }),
    );
    assert_eq!(once_with.next().unwrap(), Some(42));
    assert!(once_with.next().unwrap().is_none());
    assert!(once_with.next().unwrap().is_none()); // Should be fused

    // Test with error from closure
    let mut once_with_err =
        fallible_once_with_err::<_, lender::fallible_lend!(i32), _>(42, |_x: &mut i32| {
            "error".to_string()
        });
    match once_with_err.next() {
        Err(e) => assert_eq!(e, "error"),
        Ok(_) => panic!("Expected error"),
    }
    assert!(once_with_err.next().unwrap().is_none());
}

#[test]
fn fallible_repeat_with() {
    use lender::{fallible_lend, fallible_repeat_with, fallible_repeat_with_err};

    // Test with closure that returns values
    let mut counter = 0;
    let mut repeat_with = fallible_repeat_with::<'_, fallible_lend!(i32), String, _>(move || {
        counter += 1;
        counter
    });
    assert_eq!(repeat_with.next().unwrap(), Some(1));
    assert_eq!(repeat_with.next().unwrap(), Some(2));
    assert_eq!(repeat_with.next().unwrap(), Some(3));

    // Test with closure that returns errors
    let mut repeat_with_err =
        fallible_repeat_with_err::<fallible_lend!(i32), _>(|| "error".to_string());
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
    let repeat_with_hint = fallible_repeat_with::<'_, fallible_lend!(i32), String, _>(|| 1);
    assert_eq!(repeat_with_hint.size_hint(), (usize::MAX, None));

    // FallibleRepeatWith should be double-ended (infinite both ways)
    let mut repeat_with_de = fallible_repeat_with::<'_, fallible_lend!(i32), String, _>(|| 99);
    assert_eq!(repeat_with_de.next_back().unwrap(), Some(99));
    assert_eq!(repeat_with_de.next_back().unwrap(), Some(99));
    assert_eq!(repeat_with_de.next().unwrap(), Some(99));
}

// ============================================================================
// FromFallibleIterRef source tests
// ============================================================================

#[test]
fn from_fallible_iter_ref_basic() {
    use fallible_iterator::IteratorExt;

    let mut lender = lender::from_fallible_iter_ref([1, 2, 3].into_iter().into_fallible());
    assert_eq!(lender.next().unwrap(), Some(&1));
    assert_eq!(lender.next().unwrap(), Some(&2));
    assert_eq!(lender.next().unwrap(), Some(&3));
    assert!(lender.next().unwrap().is_none());
}

#[test]
fn from_fallible_iter_ref_ext() {
    use fallible_iterator::IteratorExt;

    let mut lender = [1, 2, 3]
        .into_iter()
        .into_fallible()
        .into_fallible_ref_lender();
    assert_eq!(lender.next().unwrap(), Some(&1));
    assert_eq!(lender.next().unwrap(), Some(&2));
    assert_eq!(lender.next().unwrap(), Some(&3));
    assert!(lender.next().unwrap().is_none());
}

#[test]
fn from_fallible_iter_ref_size_hint() {
    use fallible_iterator::IteratorExt;

    let lender = lender::from_fallible_iter_ref(vec![1, 2, 3].into_iter().into_fallible());
    assert_eq!(lender.size_hint(), (3, Some(3)));
}

#[test]
fn from_fallible_iter_ref_count() {
    use fallible_iterator::IteratorExt;

    let lender = lender::from_fallible_iter_ref(vec![1, 2, 3].into_iter().into_fallible());
    assert_eq!(lender.count(), Ok(3));
}

#[test]
fn from_fallible_iter_ref_nth() {
    use fallible_iterator::IteratorExt;

    let mut lender = lender::from_fallible_iter_ref([1, 2, 3, 4, 5].into_iter().into_fallible());
    assert_eq!(lender.nth(2).unwrap(), Some(&3));
    assert_eq!(lender.next().unwrap(), Some(&4));
}

#[test]
fn from_fallible_iter_ref_fold() {
    use fallible_iterator::IteratorExt;

    let sum = lender::from_fallible_iter_ref([1, 2, 3, 4, 5].into_iter().into_fallible())
        .fold(0, |acc, &x| Ok(acc + x));
    assert_eq!(sum, Ok(15));
}

#[test]
fn from_fallible_iter_ref_double_ended() {
    use fallible_iterator::IteratorExt;

    let mut lender = lender::from_fallible_iter_ref([1, 2, 3].into_iter().into_fallible());
    assert_eq!(lender.next_back().unwrap(), Some(&3));
    assert_eq!(lender.next().unwrap(), Some(&1));
    assert_eq!(lender.next_back().unwrap(), Some(&2));
    assert!(lender.next().unwrap().is_none());
}

#[test]
fn from_fallible_iter_ref_rfold() {
    use fallible_iterator::IteratorExt;

    let values = lender::from_fallible_iter_ref([1, 2, 3].into_iter().into_fallible()).rfold(
        Vec::new(),
        |mut acc, &x| {
            acc.push(x);
            Ok(acc)
        },
    );
    assert_eq!(values, Ok(vec![3, 2, 1]));
}

#[test]
fn from_fallible_iter_ref_from_trait() {
    use fallible_iterator::IteratorExt;

    let mut lender = lender::FromFallibleIterRef::from([1, 2, 3].into_iter().into_fallible());
    assert_eq!(lender.next().unwrap(), Some(&1));
}

/// A [`FallibleIterator`] that yields items from a `Vec` and
/// errors at a specific index.
struct ErrorAtIter {
    data: Vec<i32>,
    front: usize,
    back: usize,
    error_at: usize,
}

impl ErrorAtIter {
    fn new(data: Vec<i32>, error_at: usize) -> Self {
        let back = data.len();
        Self {
            data,
            front: 0,
            back,
            error_at,
        }
    }
}

impl fallible_iterator::FallibleIterator for ErrorAtIter {
    type Item = i32;
    type Error = String;

    fn next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        if self.front >= self.back {
            Ok(None)
        } else if self.front == self.error_at {
            self.front += 1;
            Err(format!("error at index {}", self.error_at))
        } else {
            let item = self.data[self.front];
            self.front += 1;
            Ok(Some(item))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.back - self.front;
        (0, Some(remaining))
    }
}

impl fallible_iterator::DoubleEndedFallibleIterator for ErrorAtIter {
    fn next_back(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        if self.front >= self.back {
            Ok(None)
        } else {
            self.back -= 1;
            if self.back == self.error_at {
                Err(format!("error at index {}", self.error_at))
            } else {
                Ok(Some(self.data[self.back]))
            }
        }
    }
}

#[test]
fn from_fallible_iter_ref_last() {
    let mut lender = lender::from_fallible_iter_ref([1, 2, 3].into_iter().into_fallible());
    assert_eq!(lender.last().unwrap(), Some(&3));
}

#[test]
fn from_fallible_iter_ref_last_empty() {
    let mut lender = lender::from_fallible_iter_ref(core::iter::empty::<i32>().into_fallible());
    assert_eq!(lender.last().unwrap(), None);
}

#[test]
fn from_fallible_iter_ref_advance_by() {
    let mut lender = lender::from_fallible_iter_ref([1, 2, 3, 4, 5].into_iter().into_fallible());
    assert_eq!(lender.advance_by(2).unwrap(), Ok(()));
    assert_eq!(lender.next().unwrap(), Some(&3));
}

#[test]
fn from_fallible_iter_ref_advance_by_past_end() {
    let mut lender = lender::from_fallible_iter_ref([1, 2].into_iter().into_fallible());
    assert_eq!(
        lender.advance_by(5).unwrap(),
        Err(core::num::NonZeroUsize::new(3).unwrap())
    );
}

#[test]
fn from_fallible_iter_ref_advance_back_by() {
    let mut lender = lender::from_fallible_iter_ref([1, 2, 3, 4, 5].into_iter().into_fallible());
    assert_eq!(lender.advance_back_by(2).unwrap(), Ok(()));
    assert_eq!(lender.next_back().unwrap(), Some(&3));
}

#[test]
fn from_fallible_iter_ref_advance_back_by_past_end() {
    let mut lender = lender::from_fallible_iter_ref([1, 2].into_iter().into_fallible());
    assert_eq!(
        lender.advance_back_by(5).unwrap(),
        Err(core::num::NonZeroUsize::new(3).unwrap())
    );
}

#[test]
fn from_fallible_iter_ref_any() {
    let mut lender = lender::from_fallible_iter_ref([1, 2, 3, 4, 5].into_iter().into_fallible());
    assert!(lender.any(|&x| Ok(x == 3)).unwrap());
    assert_eq!(lender.next().unwrap(), Some(&4));
}

#[test]
fn from_fallible_iter_ref_find() {
    let mut lender = lender::from_fallible_iter_ref([1, 2, 3, 4, 5].into_iter().into_fallible());
    assert_eq!(lender.find(|&&x| Ok(x > 2)).unwrap(), Some(&3));
    assert_eq!(lender.next().unwrap(), Some(&4));
}

#[test]
fn from_fallible_iter_ref_rfind() {
    let mut lender = lender::from_fallible_iter_ref([1, 2, 3, 4, 5].into_iter().into_fallible());
    assert_eq!(lender.rfind(|&&x| Ok(x < 4)).unwrap(), Some(&3));
    assert_eq!(lender.next_back().unwrap(), Some(&2));
}

#[test]
fn from_fallible_iter_ref_error_in_last() {
    let mut lender = lender::from_fallible_iter_ref(ErrorAtIter::new(vec![1, 2, 3], 1));
    assert_eq!(lender.last().unwrap_err(), "error at index 1");
}

#[test]
fn from_fallible_iter_ref_error_in_advance_by() {
    let mut lender = lender::from_fallible_iter_ref(ErrorAtIter::new(vec![1, 2, 3], 1));
    assert_eq!(lender.advance_by(3).unwrap_err(), "error at index 1");
}

#[test]
fn from_fallible_iter_ref_error_in_advance_back_by() {
    let mut lender = lender::from_fallible_iter_ref(ErrorAtIter::new(vec![1, 2, 3], 1));
    assert_eq!(lender.advance_back_by(3).unwrap_err(), "error at index 1");
}

#[test]
fn from_fallible_iter_ref_error_in_any() {
    let mut lender = lender::from_fallible_iter_ref(ErrorAtIter::new(vec![1, 2, 3], 1));
    assert_eq!(lender.any(|&x| Ok(x == 5)).unwrap_err(), "error at index 1");
}

#[test]
fn from_fallible_iter_ref_error_in_next() {
    let mut lender = lender::from_fallible_iter_ref(ErrorAtIter::new(vec![1, 2, 3], 1));
    assert_eq!(lender.next().unwrap(), Some(&1));
    assert_eq!(lender.next().unwrap_err(), "error at index 1");
}

#[test]
fn from_fallible_iter_ref_error_in_nth() {
    let mut lender = lender::from_fallible_iter_ref(ErrorAtIter::new(vec![1, 2, 3], 2));
    assert_eq!(lender.nth(2).unwrap_err(), "error at index 2");
}

#[test]
fn from_fallible_iter_ref_error_in_fold() {
    let result = lender::from_fallible_iter_ref(ErrorAtIter::new(vec![1, 2, 3], 1))
        .fold(0, |acc, &x| Ok(acc + x));
    assert_eq!(result.unwrap_err(), "error at index 1");
}

#[test]
fn from_fallible_iter_ref_error_in_next_back() {
    let mut lender = lender::from_fallible_iter_ref(ErrorAtIter::new(vec![1, 2, 3], 1));
    assert_eq!(lender.next_back().unwrap(), Some(&3));
    // index 1 errors when reached from the back
    assert_eq!(lender.next_back().unwrap_err(), "error at index 1");
}

#[test]
fn from_fallible_iter_ref_error_in_rfold() {
    let result = lender::from_fallible_iter_ref(ErrorAtIter::new(vec![1, 2, 3], 1))
        .rfold(0, |acc, &x| Ok(acc + x));
    assert_eq!(result.unwrap_err(), "error at index 1");
}

#[test]
fn from_fallible_fn() {
    use lender::from_fallible_fn;

    // Test with stateful closure that counts up
    let mut from_fn = from_fallible_fn(
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
    assert_eq!(from_fn.next().unwrap(), Some(1));
    assert_eq!(from_fn.next().unwrap(), Some(2));
    assert_eq!(from_fn.next().unwrap(), Some(3));
    assert!(from_fn.next().unwrap().is_none());
    assert!(from_fn.next().unwrap().is_none()); // Should continue returning None

    // Test with closure that returns error
    let mut from_fn_err = from_fallible_fn(
        0,
        covar_mut!(
            for<'lend> |state: &'lend mut i32| -> Result<Option<i32>, String> {
                *state += 1;
                if *state == 2 {
                    Err("error".to_string())
                } else if *state < 4 {
                    Ok(Some(*state))
                } else {
                    Ok(None)
                }
            }
        ),
    );
    assert_eq!(from_fn_err.next().unwrap(), Some(1));
    match from_fn_err.next() {
        Err(e) => assert_eq!(e, "error"),
        Ok(_) => panic!("Expected error"),
    }
}
