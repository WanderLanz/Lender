mod common;
use ::lender::prelude::*;
use common::*;

// ============================================================================
// WindowsMut tests
// ============================================================================

#[test]
fn test_windows_mut() {
    // Fibonacci sequence
    let mut data = vec![0; 3 * 3];
    data[1] = 1;
    WindowsMut {
        slice: &mut data,
        begin: 0,
        len: 3,
    }
    .for_each(covar_mut!(for<'lend> |w: &'lend mut [i32]| { w[2] = w[0] + w[1] }).into_inner());
    assert_eq!(data, [0, 1, 1, 2, 3, 5, 8, 13, 21]);
    WindowsMut {
        slice: &mut data,
        begin: 0,
        len: 3,
    }
    .filter(|x| x[0] > 0)
    .map(covar_mut!(
        for<'lend> |x: &'lend mut [i32]| -> &'lend mut i32 { &mut x[0] }
    ))
    .for_each(covar_mut!(for<'lend> |x: &'lend mut i32| { *x += 1 }).into_inner());
    assert_eq!(data, [0, 2, 2, 3, 4, 6, 9, 13, 21]);
}

#[test]
fn test_windows_mut_fold() {
    let mut data = [0, 1, 2, 3, 4];
    let count = lender::windows_mut(&mut data, 2).fold(0, |acc, _window| acc + 1);
    assert_eq!(count, 4); // 4 windows of size 2 in array of 5
}

#[test]
fn test_windows_mut_rfold() {
    let mut data = [1, 2, 3, 4];
    let mut windows = Vec::new();
    lender::windows_mut(&mut data, 2).rfold((), |(), w| {
        windows.push(w.to_vec());
    });
    // Windows in reverse: [3,4], [2,3], [1,2]
    assert_eq!(windows, vec![vec![3, 4], vec![2, 3], vec![1, 2]]);
}

#[test]
fn test_array_windows_mut_fold() {
    let mut data = [0, 1, 2, 3, 4];
    let count = lender::array_windows_mut::<_, 2>(&mut data).fold(0, |acc, _window| acc + 1);
    assert_eq!(count, 4);
}

#[test]
fn test_array_windows_mut_rfold() {
    let mut data = [1, 2, 3, 4];
    let mut first_elements = Vec::new();
    lender::array_windows_mut::<_, 2>(&mut data).rfold((), |(), w| {
        first_elements.push(w[0]);
    });
    // Windows in reverse: [3,4], [2,3], [1,2] -> first elements: 3, 2, 1
    assert_eq!(first_elements, vec![3, 2, 1]);
}

#[test]
#[should_panic(expected = "window size must be non-zero")]
fn test_windows_mut_zero_panics() {
    let mut data = [1, 2, 3];
    let _ = lender::windows_mut(&mut data, 0);
}

#[test]
#[should_panic(expected = "window size must be non-zero")]
fn test_array_windows_mut_zero_panics() {
    let mut data = [1, 2, 3];
    let _ = lender::array_windows_mut::<_, 0>(&mut data);
}

// ============================================================================
// Sources tests - empty, once, repeat, from_fn, etc.
// ============================================================================

#[test]
fn test_source_empty_basic() {
    let mut empty = lender::empty::<lend!(i32)>();
    assert_eq!(empty.next(), None);
    assert_eq!(empty.next(), None); // Fused
}

#[test]
fn test_source_empty_size_hint() {
    let empty = lender::empty::<lend!(i32)>();
    assert_eq!(empty.size_hint(), (0, Some(0)));
}

#[test]
fn test_source_empty_count() {
    let empty = lender::empty::<lend!(i32)>();
    assert_eq!(empty.count(), 0);
}

#[test]
fn test_source_empty_double_ended() {
    let mut empty = lender::empty::<lend!(i32)>();
    assert_eq!(empty.next_back(), None);
}

#[test]
fn test_source_once_basic() {
    let mut once = lender::once::<lend!(i32)>(42);
    assert_eq!(once.next(), Some(42));
    assert_eq!(once.next(), None);
    assert_eq!(once.next(), None); // Fused
}

#[test]
fn test_source_once_size_hint() {
    let once = lender::once::<lend!(i32)>(42);
    assert_eq!(once.size_hint(), (1, Some(1)));
}

#[test]
fn test_source_once_count() {
    let once = lender::once::<lend!(i32)>(42);
    assert_eq!(once.count(), 1);
}

#[test]
fn test_source_once_double_ended() {
    let mut once = lender::once::<lend!(i32)>(42);
    assert_eq!(once.next_back(), Some(42));
    assert_eq!(once.next_back(), None);
}

#[test]
fn test_source_once_fold() {
    let sum = lender::once::<lend!(i32)>(42).fold(0, |acc, x| acc + x);
    assert_eq!(sum, 42);
}

#[test]
fn test_source_repeat_basic() {
    let mut repeat = lender::repeat::<lend!(i32)>(42);
    assert_eq!(repeat.next(), Some(42));
    assert_eq!(repeat.next(), Some(42));
    assert_eq!(repeat.next(), Some(42));
    // Infinite - continues forever
}

#[test]
fn test_source_repeat_take() {
    let sum = lender::repeat::<lend!(i32)>(5)
        .take(4)
        .fold(0, |acc, x| acc + x);
    assert_eq!(sum, 20); // 5 * 4
}

#[test]
fn test_source_repeat_double_ended() {
    let mut repeat = lender::repeat::<lend!(i32)>(42);
    // Repeat is double-ended (infinite both ways)
    assert_eq!(repeat.next_back(), Some(42));
    assert_eq!(repeat.next_back(), Some(42));
}

#[test]
fn test_source_repeat_size_hint() {
    // Per Iterator::repeat docs: size_hint returns (usize::MAX, None) for infinite iterators
    let repeat = lender::repeat::<lend!(i32)>(42);
    assert_eq!(repeat.size_hint(), (usize::MAX, None));
}

#[test]
fn test_source_repeat_with_basic() {
    let mut counter = 0;
    let mut repeat_with = lender::repeat_with::<lend!(i32), _>(|| {
        counter += 1;
        counter
    });

    assert_eq!(repeat_with.next(), Some(1));
    assert_eq!(repeat_with.next(), Some(2));
    assert_eq!(repeat_with.next(), Some(3));
}

#[test]
fn test_source_repeat_with_size_hint() {
    let repeat_with = lender::repeat_with::<lend!(i32), _>(|| 42);
    assert_eq!(repeat_with.size_hint(), (usize::MAX, None));
}

#[test]
fn test_source_repeat_with_double_ended() {
    use lender::DoubleEndedLender;

    let mut repeat_with = lender::repeat_with::<lend!(i32), _>(|| 42);
    // RepeatWith is double-ended (infinite both ways)
    assert_eq!(repeat_with.next_back(), Some(42));
    assert_eq!(repeat_with.next_back(), Some(42));
    assert_eq!(repeat_with.next(), Some(42));
}

#[test]
fn test_source_from_fn_basic() {
    let mut from_fn = lender::from_fn(
        0,
        covar_mut!(for<'all> |s: &'all mut i32| -> Option<i32> {
            *s += 1;
            if *s <= 3 { Some(*s) } else { None }
        }),
    );

    assert_eq!(from_fn.next(), Some(1));
    assert_eq!(from_fn.next(), Some(2));
    assert_eq!(from_fn.next(), Some(3));
    assert_eq!(from_fn.next(), None);
}

#[test]
fn test_source_once_with_basic() {
    let mut once_with = lender::once_with(
        42,
        covar_once!(for<'lend> |state: &'lend mut i32| -> &'lend mut i32 {
            *state += 1;
            state
        }),
    );

    assert_eq!(once_with.next(), Some(&mut 43));
    assert_eq!(once_with.next(), None);
}

// ============================================================================
// Additional source tests
// ============================================================================

#[test]
fn test_source_repeat_advance_by() {
    let mut repeat = lender::repeat::<lend!(i32)>(42);
    // Advance does nothing on repeat (always Ok)
    assert_eq!(repeat.advance_by(100), Ok(()));
    assert_eq!(repeat.next(), Some(42));
}

#[test]
fn test_source_empty_fold_additional() {
    let sum = lender::empty::<lend!(i32)>().fold(0, |acc, x| acc + x);
    assert_eq!(sum, 0);
}

#[test]
fn test_source_empty_rfold_additional() {
    use lender::DoubleEndedLender;

    let sum = lender::empty::<lend!(i32)>().rfold(0, |acc, x| acc + x);
    assert_eq!(sum, 0);
}

#[test]
fn test_source_once_rfold_additional() {
    use lender::DoubleEndedLender;

    let sum = lender::once::<lend!(i32)>(42).rfold(0, |acc, x| acc + x);
    assert_eq!(sum, 42);
}

// ============================================================================
// FromIter source tests
// ============================================================================

#[test]
fn test_from_iter_basic() {
    let mut lender = vec![1, 2, 3].into_iter().into_lender();

    assert_eq!(lender.next(), Some(1));
    assert_eq!(lender.next(), Some(2));
    assert_eq!(lender.next(), Some(3));
    assert_eq!(lender.next(), None);
}

#[test]
fn test_from_iter_size_hint() {
    let lender = vec![1, 2, 3].into_iter().into_lender();
    assert_eq!(lender.size_hint(), (3, Some(3)));
}

#[test]
fn test_from_iter_double_ended() {
    let mut lender = vec![1, 2, 3].into_iter().into_lender();

    assert_eq!(lender.next_back(), Some(3));
    assert_eq!(lender.next(), Some(1));
    assert_eq!(lender.next_back(), Some(2));
    assert_eq!(lender.next(), None);
}

// ============================================================================
// FromIter source additional tests
// ============================================================================

#[test]
fn test_from_iter_fold_additional() {
    let sum = vec![1, 2, 3, 4, 5]
        .into_iter()
        .into_lender()
        .fold(0, |acc, x| acc + x);
    assert_eq!(sum, 15);
}

#[test]
fn test_from_iter_rfold_additional() {
    use lender::DoubleEndedLender;

    let values: Vec<i32> =
        vec![1, 2, 3]
            .into_iter()
            .into_lender()
            .rfold(Vec::new(), |mut acc, x| {
                acc.push(x);
                acc
            });
    assert_eq!(values, vec![3, 2, 1]);
}

#[test]
fn test_from_iter_try_fold_additional() {
    let result: Option<i32> = vec![1, 2, 3]
        .into_iter()
        .into_lender()
        .try_fold(0, |acc, x| Some(acc + x));
    assert_eq!(result, Some(6));
}

#[test]
fn test_from_iter_try_rfold_additional() {
    use lender::DoubleEndedLender;

    let result: Option<i32> = vec![1, 2, 3]
        .into_iter()
        .into_lender()
        .try_rfold(0, |acc, x| Some(acc + x));
    assert_eq!(result, Some(6));
}

#[test]
fn test_from_iter_nth_additional() {
    let mut lender = vec![1, 2, 3, 4, 5].into_iter().into_lender();
    assert_eq!(lender.nth(2), Some(3));
}

#[test]
fn test_from_iter_nth_back_additional() {
    use lender::DoubleEndedLender;

    let mut lender = vec![1, 2, 3, 4, 5].into_iter().into_lender();
    assert_eq!(lender.nth_back(2), Some(3));
}

#[test]
fn test_from_iter_fallible_coverage() {
    use lender::{DoubleEndedFallibleLender, FallibleLender};

    let data = [1, 2, 3];
    let fallible: lender::IntoFallible<_> = data.iter().into_lender().into_fallible();
    let mut lender = fallible;
    assert_eq!(lender.next(), Ok(Some(&1)));
    assert_eq!(lender.next_back(), Ok(Some(&3)));
}

// ============================================================================
// FromIterRef source tests
// ============================================================================

#[test]
fn test_from_iter_ref_basic() {
    let mut lender = lender::from_iter_ref([1, 2, 3].into_iter());

    assert_eq!(lender.next(), Some(&1));
    assert_eq!(lender.next(), Some(&2));
    assert_eq!(lender.next(), Some(&3));
    assert_eq!(lender.next(), None);
}

#[test]
fn test_from_iter_ref_ext() {
    let mut lender = [1, 2, 3].into_iter().into_ref_lender();

    assert_eq!(lender.next(), Some(&1));
    assert_eq!(lender.next(), Some(&2));
    assert_eq!(lender.next(), Some(&3));
    assert_eq!(lender.next(), None);
}

#[test]
fn test_from_iter_ref_size_hint() {
    let lender = lender::from_iter_ref(vec![1, 2, 3].into_iter());
    assert_eq!(lender.size_hint(), (3, Some(3)));
}

#[test]
fn test_from_iter_ref_count() {
    let lender = lender::from_iter_ref(vec![1, 2, 3].into_iter());
    assert_eq!(lender.count(), 3);
}

#[test]
fn test_from_iter_ref_nth() {
    let mut lender = lender::from_iter_ref([1, 2, 3, 4, 5].into_iter());
    assert_eq!(lender.nth(2), Some(&3));
    assert_eq!(lender.next(), Some(&4));
}

#[test]
fn test_from_iter_ref_advance_by() {
    let mut lender = lender::from_iter_ref([1, 2, 3, 4, 5].into_iter());
    assert_eq!(lender.advance_by(2), Ok(()));
    assert_eq!(lender.next(), Some(&3));
}

#[test]
fn test_from_iter_ref_advance_by_zero() {
    let mut lender = lender::from_iter_ref([1, 2, 3].into_iter());
    assert_eq!(lender.advance_by(0), Ok(()));
    assert_eq!(lender.next(), Some(&1));
}

#[test]
fn test_from_iter_ref_advance_by_past_end() {
    let mut lender = lender::from_iter_ref([1, 2].into_iter());
    let result = lender.advance_by(5);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().get(), 3); // 5 - 2 = 3 remaining
}

#[test]
fn test_from_iter_ref_last() {
    let mut lender = lender::from_iter_ref([1, 2, 3].into_iter());
    assert_eq!(lender.last(), Some(&3));
}

#[test]
fn test_from_iter_ref_last_empty() {
    let mut lender = lender::from_iter_ref(core::iter::empty::<i32>());
    assert_eq!(lender.last(), None);
}

#[test]
fn test_from_iter_ref_fold() {
    let sum = lender::from_iter_ref([1, 2, 3, 4, 5].into_iter()).fold(0, |acc, &x| acc + x);
    assert_eq!(sum, 15);
}

#[test]
fn test_from_iter_ref_double_ended() {
    let mut lender = lender::from_iter_ref([1, 2, 3].into_iter());

    assert_eq!(lender.next_back(), Some(&3));
    assert_eq!(lender.next(), Some(&1));
    assert_eq!(lender.next_back(), Some(&2));
    assert_eq!(lender.next(), None);
}

#[test]
fn test_from_iter_ref_nth_back() {
    let mut lender = lender::from_iter_ref([1, 2, 3, 4, 5].into_iter());
    assert_eq!(lender.nth_back(2), Some(&3));
    assert_eq!(lender.next_back(), Some(&2));
}

#[test]
fn test_from_iter_ref_advance_back_by() {
    let mut lender = lender::from_iter_ref([1, 2, 3, 4, 5].into_iter());
    assert_eq!(lender.advance_back_by(2), Ok(()));
    assert_eq!(lender.next_back(), Some(&3));
}

#[test]
fn test_from_iter_ref_advance_back_by_past_end() {
    let mut lender = lender::from_iter_ref([1, 2].into_iter());
    let result = lender.advance_back_by(5);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().get(), 3);
}

#[test]
fn test_from_iter_ref_rfold() {
    let values = lender::from_iter_ref([1, 2, 3].into_iter()).rfold(Vec::new(), |mut acc, &x| {
        acc.push(x);
        acc
    });
    assert_eq!(values, vec![3, 2, 1]);
}

#[test]
fn test_from_iter_ref_exact_size() {
    let lender = lender::from_iter_ref(vec![1, 2, 3].into_iter());
    assert_eq!(lender.len(), 3);
}

#[test]
fn test_from_iter_ref_from_trait() {
    let mut lender = lender::FromIterRef::from([1, 2, 3].into_iter());
    assert_eq!(lender.next(), Some(&1));
}

#[test]
fn test_from_iter_ref_any() {
    let mut lender = lender::from_iter_ref([1, 2, 3, 4, 5].into_iter());
    assert!(lender.any(|&x| x == 3));
    // After finding 3, the lender should have remaining elements.
    assert_eq!(lender.next(), Some(&4));
}

#[test]
fn test_from_iter_ref_find() {
    let mut lender = lender::from_iter_ref([1, 2, 3, 4, 5].into_iter());
    assert_eq!(lender.find(|&&x| x > 2), Some(&3));
    assert_eq!(lender.next(), Some(&4));
}

#[test]
fn test_from_iter_ref_position() {
    let mut lender = lender::from_iter_ref([10, 20, 30].into_iter());
    assert_eq!(lender.position(|&x| x == 20), Some(1));
}

#[test]
fn test_from_iter_ref_rfind() {
    let mut lender = lender::from_iter_ref([1, 2, 3, 4, 5].into_iter());
    assert_eq!(lender.rfind(|&&x| x < 4), Some(&3));
    assert_eq!(lender.next_back(), Some(&2));
}

#[test]
fn test_from_iter_ref_all() {
    let mut lender = lender::from_iter_ref([2, 4, 6].into_iter());
    assert!(lender.all(|&x| x % 2 == 0));
}

// ============================================================================
// LendIter source tests
// ============================================================================

#[test]
fn test_lend_iter_basic() {
    let data = [1, 2, 3];
    let mut lender = lender::lend_iter::<lend!(&'lend i32), _>(data.iter());

    assert_eq!(lender.next(), Some(&1));
    assert_eq!(lender.next(), Some(&2));
    assert_eq!(lender.next(), Some(&3));
    assert_eq!(lender.next(), None);
}

// ============================================================================
// LendIter source additional tests
// ============================================================================

#[test]
fn test_lend_iter_fold_additional() {
    let data = [1, 2, 3, 4, 5];
    let sum = lender::lend_iter::<lend!(&'lend i32), _>(data.iter()).fold(0, |acc, &x| acc + x);
    assert_eq!(sum, 15);
}

#[test]
fn test_lend_iter_rfold_additional() {
    use lender::DoubleEndedLender;

    let data = [1, 2, 3];
    let values: Vec<i32> =
        lender::lend_iter::<lend!(&'lend i32), _>(data.iter()).rfold(Vec::new(), |mut acc, &x| {
            acc.push(x);
            acc
        });
    assert_eq!(values, vec![3, 2, 1]);
}

#[test]
fn test_lend_iter_try_fold_additional() {
    let data = [1, 2, 3];
    let result: Option<i32> =
        lender::lend_iter::<lend!(&'lend i32), _>(data.iter()).try_fold(0, |acc, &x| Some(acc + x));
    assert_eq!(result, Some(6));
}

#[test]
fn test_lend_iter_nth_additional() {
    let data = [1, 2, 3, 4, 5];
    let mut lender = lender::lend_iter::<lend!(&'lend i32), _>(data.iter());
    assert_eq!(lender.nth(2), Some(&3));
}
