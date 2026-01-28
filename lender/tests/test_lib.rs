#![cfg(test)]
use ::lender::prelude::*;

// ============================================================================
// Helper struct for Lender tests - a simple double-ended lender over a Vec
// ============================================================================
#[derive(Clone)]
struct VecLender {
    data: Vec<i32>,
    front: usize,
    back: usize,
}

impl VecLender {
    fn new(data: Vec<i32>) -> Self {
        let len = data.len();
        Self {
            data,
            front: 0,
            back: len,
        }
    }
}

impl<'lend> Lending<'lend> for VecLender {
    type Lend = i32;
}

impl Lender for VecLender {
    check_covariance!();

    fn next(&mut self) -> Option<Lend<'_, Self>> {
        if self.front < self.back {
            let item = self.data[self.front];
            self.front += 1;
            Some(item)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.back - self.front;
        (len, Some(len))
    }
}

impl DoubleEndedLender for VecLender {
    fn next_back(&mut self) -> Option<Lend<'_, Self>> {
        if self.front < self.back {
            self.back -= 1;
            let item = self.data[self.back];
            Some(item)
        } else {
            None
        }
    }
}

impl ExactSizeLender for VecLender {
    fn len(&self) -> usize {
        self.back - self.front
    }
}

impl lender::FusedLender for VecLender {}

struct WindowsMut<'a, T> {
    slice: &'a mut [T],
    begin: usize,
    len: usize,
}
impl<'lend, T> Lending<'lend> for WindowsMut<'_, T> {
    type Lend = &'lend mut [T];
}
impl<T> Lender for WindowsMut<'_, T> {
    check_covariance!();
    fn next(&mut self) -> Option<&mut [T]> {
        let begin = self.begin;
        self.begin = self.begin.saturating_add(1);
        self.slice.get_mut(begin..begin + self.len)
    }
}

#[test]
fn windows_mut() {
    // Fibonacci sequence
    let mut data = vec![0u32; 3 * 3];
    data[1] = 1;
    WindowsMut {
        slice: &mut data,
        begin: 0,
        len: 3,
    }
    .for_each(hrc_mut!(for<'lend> |w: &'lend mut [u32]| {
        w[2] = w[0] + w[1]
    }));
    assert_eq!(data, [0, 1, 1, 2, 3, 5, 8, 13, 21]);
    WindowsMut {
        slice: &mut data,
        begin: 0,
        len: 3,
    }
    .filter(|x| x[0] > 0)
    .map(hrc_mut!(
        for<'lend> |x: &'lend mut [u32]| -> &'lend mut u32 { &mut x[0] }
    ))
    .for_each(hrc_mut!(for<'lend> |x: &'lend mut u32| { *x += 1 }));
    assert_eq!(data, [0, 2, 2, 3, 4, 6, 9, 13, 21]);
}

#[test]
fn lines_str() {
    use std::io;

    struct LinesStr<B> {
        buf: B,
        line: String,
    }
    impl<'lend, B: io::BufRead> Lending<'lend> for LinesStr<B> {
        type Lend = io::Result<&'lend str>;
    }
    impl<B: io::BufRead> Lender for LinesStr<B> {
        check_covariance!();
        fn next(&mut self) -> Option<io::Result<&str>> {
            self.line.clear();
            match self.buf.read_line(&mut self.line) {
                Err(e) => return Some(Err(e)),
                Ok(0) => return None,
                Ok(_nread) => (),
            };
            if self.line.ends_with('\n') {
                self.line.pop();
                if self.line.ends_with('\r') {
                    self.line.pop();
                }
            }
            Some(Ok(&self.line))
        }
    }

    let buf = io::BufReader::with_capacity(10, "Hello\nWorld\n".as_bytes());
    let mut lines = LinesStr {
        buf,
        line: String::new(),
    };
    assert_eq!(lines.next().unwrap().unwrap(), "Hello");
    assert_eq!(lines.next().unwrap().unwrap(), "World");
}

#[test]
fn simple_lender() {
    struct MyLender<'a, T: 'a>(&'a mut T);
    impl<'lend, 'a, T: 'a> Lending<'lend> for MyLender<'a, T> {
        type Lend = &'lend mut T;
    }
    impl<'a, T: 'a> Lender for MyLender<'a, T> {
        check_covariance!();
        fn next(&mut self) -> Option<Lend<'_, Self>> {
            Some(&mut self.0)
        }
    }
    let mut x = 0u32;
    let mut bar: MyLender<'_, u32> = MyLender(&mut x);
    let _ = bar.next();
    let _ = bar.next();
    let mut bar = bar
        .into_lender()
        .mutate(|y| **y += 1)
        .map(|x: &mut u32| *x + 1)
        .iter();
    let _ = bar.find_map(|x| if x > 0 { Some(vec![1, 2, 3]) } else { None });
}

#[test]
fn from_lender() {
    let mut vec = vec![1u32, 2, 3, 4, 5];
    let windows = WindowsMut {
        slice: &mut vec,
        begin: 0,
        len: 3,
    };
    let vec = MyVec::<Vec<u32>>::from_lender(windows);
    assert_eq!(vec.0, vec![&[1, 2, 3][..], &[2, 3, 4][..], &[3, 4, 5][..]]);

    struct MyVec<T>(Vec<T>);
    impl<L: Lender> FromLender<L> for MyVec<Vec<u32>>
    where
        for<'all> L: Lending<'all, Lend = &'all mut [u32]>,
    {
        fn from_lender(lender: L) -> Self {
            let mut vec = Vec::new();
            lender.for_each(|x| {
                let x = ToOwned::to_owned(x);
                vec.push(x)
            });
            MyVec(vec)
        }
    }
}

#[test]
fn try_collect() {
    use stable_try_trait_v2::ChangeOutputType;

    const ERR_MSG: &str = "Try Collect Error";

    #[derive(Debug)]
    struct WriteOnDrop<'a> {
        src: &'a str,
        dst: &'a mut String,
    }

    impl Drop for WriteOnDrop<'_> {
        fn drop(&mut self) {
            use std::fmt::Write;
            self.dst.write_str(self.src).expect("Write failed")
        }
    }

    enum ErrLenderInner {
        Count(usize),
        Err(String),
    }

    impl Default for ErrLenderInner {
        fn default() -> Self {
            Self::Count(0)
        }
    }

    struct ErrLender<'a> {
        inner: ErrLenderInner,
        dst: &'a mut String,
    }

    impl<'a> ErrLender<'a> {
        fn new(dst: &'a mut String) -> Self {
            Self {
                inner: ErrLenderInner::default(),
                dst,
            }
        }
    }

    impl<'lend> Lending<'lend> for ErrLender<'_> {
        type Lend = Result<(), WriteOnDrop<'lend>>;
    }

    impl Lender for ErrLender<'_> {
        check_covariance!();
        fn next(&mut self) -> Option<Lend<'_, Self>> {
            match self.inner {
                ErrLenderInner::Count(1) => {
                    let err = ERR_MSG.to_owned();
                    self.inner = ErrLenderInner::Err(err);
                    match &self.inner {
                        ErrLenderInner::Err(err) => Some(Err(WriteOnDrop {
                            src: err.as_str(),
                            dst: self.dst,
                        })),
                        ErrLenderInner::Count(_) => unreachable!(),
                    }
                }
                ErrLenderInner::Count(count) => {
                    self.inner = ErrLenderInner::Count(count + 1);
                    Some(Ok(()))
                }
                ErrLenderInner::Err(_) => {
                    self.inner = ErrLenderInner::Count(0);
                    Some(Ok(()))
                }
            }
        }
    }

    #[derive(Debug)]
    struct Wrapper;

    impl<L> FromLender<L> for Wrapper
    where
        L: IntoLender,
    {
        fn from_lender(lender: L) -> Self {
            let mut lender = lender.into_lender();
            while lender.next().is_some() {}
            let _ = lender.next();
            Self
        }
    }

    let mut err = String::new();
    let mut lender = ErrLender::new(&mut err);
    let res: ChangeOutputType<Result<(), _>, _> = lender.try_collect::<Wrapper>();
    let write_on_drop = res.expect_err("Expected an error");
    drop(write_on_drop);
    assert_eq!(err, ERR_MSG);
}

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
    let repeat_with_hint =
        fallible_repeat_with::<'_, fallible_lend!(i32), String, _>(|| Ok(1));
    assert_eq!(repeat_with_hint.size_hint(), (usize::MAX, None));
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
    check_covariance_fallible!();
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

// Helper struct for testing fallible traits
struct VecFallibleLender {
    data: Vec<i32>,
    front: usize,
    back: usize,
}

impl VecFallibleLender {
    fn new(data: Vec<i32>) -> Self {
        let len = data.len();
        Self {
            data,
            front: 0,
            back: len,
        }
    }
}

impl<'lend> lender::FallibleLending<'lend> for VecFallibleLender {
    type Lend = i32;
}

impl lender::FallibleLender for VecFallibleLender {
    type Error = std::convert::Infallible;
    check_covariance_fallible!();

    fn next(&mut self) -> Result<Option<lender::FallibleLend<'_, Self>>, Self::Error> {
        if self.front < self.back {
            let item = self.data[self.front];
            self.front += 1;
            Ok(Some(item))
        } else {
            Ok(None)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.back - self.front;
        (len, Some(len))
    }
}

impl lender::DoubleEndedFallibleLender for VecFallibleLender {
    fn next_back(&mut self) -> Result<Option<lender::FallibleLend<'_, Self>>, Self::Error> {
        if self.front < self.back {
            self.back -= 1;
            let item = self.data[self.back];
            Ok(Some(item))
        } else {
            Ok(None)
        }
    }
}

impl lender::ExactSizeFallibleLender for VecFallibleLender {
    fn len(&self) -> usize {
        self.back - self.front
    }
}

impl lender::FusedFallibleLender for VecFallibleLender {}

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
    assert_eq!(lender.next().unwrap(), Some(1));
    assert_eq!(lender.next_back().unwrap(), Some(5));
    assert_eq!(lender.next().unwrap(), Some(2));
    assert_eq!(lender.next_back().unwrap(), Some(4));
    assert_eq!(lender.next().unwrap(), Some(3));
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
    assert_eq!(lender.next().unwrap(), Some(1));
    assert_eq!(lender.next().unwrap(), None);
    assert_eq!(lender.next().unwrap(), None);
    assert_eq!(lender.next().unwrap(), None);
}

#[test]
fn fallible_trait_adapters_map() {
    use lender::{ExactSizeFallibleLender, FusedFallibleLender};

    fn assert_exact_size<L: ExactSizeFallibleLender>(_: &L) {}
    fn assert_fused<L: FusedFallibleLender>(_: &L) {}

    let lender = VecFallibleLender::new(vec![1, 2, 3]);
    let mapped = lender.map(hrc_mut!(for<'lend> |x: i32| -> Result<
        i32,
        std::convert::Infallible,
    > { Ok(x * 2) }));

    assert_exact_size(&mapped);
    assert_fused(&mapped);
}

#[test]
fn fallible_trait_adapters_filter() {
    use lender::FusedFallibleLender;

    fn assert_fused<L: FusedFallibleLender>(_: &L) {}

    let lender = VecFallibleLender::new(vec![1, 2, 3, 4, 5]);
    let filtered = lender.filter(|x| Ok(*x > 2));

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
    assert_eq!(skipped.next_back().unwrap(), Some(5));
    assert_eq!(skipped.next_back().unwrap(), Some(4));
    assert_eq!(skipped.next_back().unwrap(), Some(3));
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
    assert_eq!(taken.next_back().unwrap(), Some(3));
    assert_eq!(taken.next_back().unwrap(), Some(2));
    assert_eq!(taken.next_back().unwrap(), Some(1));
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
    assert_eq!(zipped.next_back().unwrap(), Some((3, 30)));
    assert_eq!(zipped.next_back().unwrap(), Some((2, 20)));
    assert_eq!(zipped.next_back().unwrap(), Some((1, 10)));
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
    assert_eq!(reversed.next().unwrap(), Some(3));
    assert_eq!(reversed.next().unwrap(), Some(2));
    assert_eq!(reversed.next().unwrap(), Some(1));
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
    assert_eq!(stepped.next().unwrap(), Some(1));
    assert_eq!(stepped.next().unwrap(), Some(3));
    assert_eq!(stepped.next().unwrap(), Some(5));
    assert_eq!(stepped.next().unwrap(), None);

    // Test step_by with next_back
    let mut stepped = VecFallibleLender::new(vec![1, 2, 3, 4, 5, 6]).step_by(2);
    assert_eq!(stepped.next_back().unwrap(), Some(5));
    assert_eq!(stepped.next_back().unwrap(), Some(3));
    assert_eq!(stepped.next_back().unwrap(), Some(1));
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
    assert_eq!(chained.next().unwrap(), Some(1));
    assert_eq!(chained.next().unwrap(), Some(2));
    assert_eq!(chained.next().unwrap(), Some(3));
    assert_eq!(chained.next().unwrap(), Some(4));
    assert_eq!(chained.next().unwrap(), None);
}

#[test]
fn fallible_trait_adapters_inspect() {
    use lender::{ExactSizeFallibleLender, FusedFallibleLender};

    fn assert_exact_size<L: ExactSizeFallibleLender>(_: &L) {}
    fn assert_fused<L: FusedFallibleLender>(_: &L) {}

    let lender = VecFallibleLender::new(vec![1, 2, 3]);
    let inspected = lender.inspect(|_| Ok(()));

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
// Chain adapter tests (Lender)
// ============================================================================

#[test]
fn chain_basic_forward_iteration() {
    // Documented semantics: "first yield all lends from self, then all lends from other"
    let mut chained = VecLender::new(vec![1, 2]).chain(VecLender::new(vec![3, 4]));

    // First lender's elements come first
    assert_eq!(chained.next(), Some(1));
    assert_eq!(chained.next(), Some(2));
    // Then second lender's elements
    assert_eq!(chained.next(), Some(3));
    assert_eq!(chained.next(), Some(4));
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
    assert_eq!(chained.next_back(), Some(4));
    assert_eq!(chained.next_back(), Some(3));
    // Then from the end of the first lender
    assert_eq!(chained.next_back(), Some(2));
    assert_eq!(chained.next_back(), Some(1));
    // Exhausted
    assert_eq!(chained.next_back(), None);
}

#[test]
fn chain_mixed_forward_backward() {
    // Mixed iteration: front and back should not interfere incorrectly
    let mut chained = VecLender::new(vec![1, 2, 3]).chain(VecLender::new(vec![4, 5, 6]));

    // Take from front
    assert_eq!(chained.next(), Some(1));
    // Take from back
    assert_eq!(chained.next_back(), Some(6));
    // Continue from front
    assert_eq!(chained.next(), Some(2));
    // Continue from back
    assert_eq!(chained.next_back(), Some(5));
    // Should meet in the middle
    assert_eq!(chained.next(), Some(3));
    assert_eq!(chained.next_back(), Some(4));
    // Now exhausted
    assert_eq!(chained.next(), None);
    assert_eq!(chained.next_back(), None);
}

#[test]
fn chain_empty_first_lender() {
    // When first lender is empty, should immediately yield from second
    let mut chained = VecLender::new(vec![]).chain(VecLender::new(vec![1, 2]));

    assert_eq!(chained.next(), Some(1));
    assert_eq!(chained.next(), Some(2));
    assert_eq!(chained.next(), None);
}

#[test]
fn chain_empty_second_lender() {
    // When second lender is empty, should yield from first then be exhausted
    let mut chained = VecLender::new(vec![1, 2]).chain(VecLender::new(vec![]));

    assert_eq!(chained.next(), Some(1));
    assert_eq!(chained.next(), Some(2));
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
    assert_eq!(chained.nth(0), Some(1));
    // nth(1) skips one and returns the next
    assert_eq!(chained.nth(1), Some(3));
    // Now we should be at element 4 (index 3 originally, but we've consumed 0, 1, 2)
    // nth(1) skips 4 and returns 5
    assert_eq!(chained.nth(1), Some(5));
    // Only 6 left, nth(0) returns it
    assert_eq!(chained.nth(0), Some(6));
    // Exhausted
    assert_eq!(chained.nth(0), None);
}

#[test]
fn chain_nth_crossing_boundary() {
    // nth that crosses the boundary between first and second lender
    let mut chained = VecLender::new(vec![1, 2]).chain(VecLender::new(vec![3, 4, 5]));

    // Skip past first lender entirely and into second
    assert_eq!(chained.nth(3), Some(4)); // skip 1,2,3 -> return 4
    assert_eq!(chained.next(), Some(5));
    assert_eq!(chained.next(), None);
}

#[test]
fn chain_nth_back() {
    // nth_back(n) should skip n elements from the back
    let mut chained = VecLender::new(vec![1, 2, 3]).chain(VecLender::new(vec![4, 5, 6]));

    // nth_back(0) is same as next_back()
    assert_eq!(chained.nth_back(0), Some(6));
    // nth_back(1) skips 5 and returns 4
    assert_eq!(chained.nth_back(1), Some(4));
    // Now crossing into first lender: nth_back(1) skips 3 and returns 2
    assert_eq!(chained.nth_back(1), Some(2));
    // Only 1 left
    assert_eq!(chained.nth_back(0), Some(1));
    assert_eq!(chained.nth_back(0), None);
}

#[test]
fn chain_last() {
    // last() should return the last element of the second lender (if non-empty)
    let mut chained = VecLender::new(vec![1, 2]).chain(VecLender::new(vec![3, 4]));
    assert_eq!(chained.last(), Some(4));

    // If second lender is empty, should return last of first
    let mut chained2 = VecLender::new(vec![1, 2]).chain(VecLender::new(vec![]));
    assert_eq!(chained2.last(), Some(2));

    // If both empty, should return None
    let mut chained3 = VecLender::new(vec![]).chain(VecLender::new(vec![]));
    assert_eq!(chained3.last(), None);
}

#[test]
fn chain_find() {
    // find() should search first lender, then second
    let mut chained = VecLender::new(vec![1, 2, 3]).chain(VecLender::new(vec![4, 5, 6]));

    // Find in first lender
    assert_eq!(chained.find(|&x| x == 2), Some(2));
    // Now first lender is partially consumed (1 was skipped), continue search
    // find in second lender
    assert_eq!(chained.find(|&x| x > 4), Some(5));
}

#[test]
fn chain_rfind() {
    // rfind() should search second lender first (from back), then first
    let mut chained = VecLender::new(vec![1, 2, 3]).chain(VecLender::new(vec![4, 5, 6]));

    // rfind in second lender
    assert_eq!(chained.rfind(|&x| x == 5), Some(5));
    // rfind crosses to first lender
    assert_eq!(chained.rfind(|&x| x < 3), Some(2));
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
        .fold(0, |acc, x| acc + x);
    assert_eq!(sum, 21); // 1+2+3+4+5+6
}

#[test]
fn chain_rfold() {
    // rfold should process elements in reverse order (second lender first, from back)
    let mut order = Vec::new();
    VecLender::new(vec![1, 2, 3])
        .chain(VecLender::new(vec![4, 5, 6]))
        .rfold((), |(), x| order.push(x));
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

    assert_eq!(fused.next(), Some(1));
    assert_eq!(fused.next(), Some(2));
    assert_eq!(fused.next(), Some(3));
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

    assert_eq!(fused.next(), Some(1));
    assert_eq!(fused.next_back(), Some(4));
    assert_eq!(fused.next(), Some(2));
    assert_eq!(fused.next_back(), Some(3));
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
    assert_eq!(fused.nth(2), Some(3));
    // nth(0) returns next element (4)
    assert_eq!(fused.nth(0), Some(4));
    // nth(0) returns 5
    assert_eq!(fused.nth(0), Some(5));
    // Now exhausted
    assert_eq!(fused.nth(0), None);
    // Fused - stays None
    assert_eq!(fused.nth(0), None);
}

#[test]
fn fuse_nth_back() {
    let mut fused = VecLender::new(vec![1, 2, 3, 4, 5]).fuse();

    // nth_back(1) skips 5 and returns 4
    assert_eq!(fused.nth_back(1), Some(4));
    // nth_back(0) returns 3
    assert_eq!(fused.nth_back(0), Some(3));
    // Continue
    assert_eq!(fused.nth_back(0), Some(2));
    assert_eq!(fused.nth_back(0), Some(1));
    // Exhausted
    assert_eq!(fused.nth_back(0), None);
    // Fused - stays None
    assert_eq!(fused.nth_back(0), None);
}

#[test]
fn fuse_last() {
    // last() should return the last element
    let mut fused = VecLender::new(vec![1, 2, 3]).fuse();
    assert_eq!(fused.last(), Some(3));

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
        .fold(0, |acc, x| acc + x);
    assert_eq!(sum, 10);
}

#[test]
fn fuse_rfold() {
    let mut order = Vec::new();
    VecLender::new(vec![1, 2, 3])
        .fuse()
        .rfold((), |(), x| order.push(x));
    assert_eq!(order, vec![3, 2, 1]);
}

#[test]
fn fuse_find() {
    let mut fused = VecLender::new(vec![1, 2, 3, 4, 5]).fuse();

    assert_eq!(fused.find(|&x| x > 2), Some(3));
    assert_eq!(fused.find(|&x| x > 4), Some(5));
    assert_eq!(fused.find(|&x| x > 10), None);
    // After returning None from find, should stay fused
    assert_eq!(fused.next(), None);
}

#[test]
fn fuse_rfind() {
    let mut fused = VecLender::new(vec![1, 2, 3, 4, 5]).fuse();

    assert_eq!(fused.rfind(|&x| x < 4), Some(3));
    assert_eq!(fused.rfind(|&x| x < 2), Some(1));
    assert_eq!(fused.rfind(|&x| x < 0), None);
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

// ============================================================================
// StepBy adapter tests (Lender)
// Semantics: step_by(n) returns every nth element, starting with the first.
// Specifically: returns elements at indices 0, n, 2n, 3n, ...
// ============================================================================

#[test]
fn step_by_basic() {
    // Documented example: step_by(2) yields elements at indices 0, 2, 4, 6, 8...
    let mut stepped = VecLender::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]).step_by(2);

    assert_eq!(stepped.next(), Some(1)); // index 0
    assert_eq!(stepped.next(), Some(3)); // index 2
    assert_eq!(stepped.next(), Some(5)); // index 4
    assert_eq!(stepped.next(), Some(7)); // index 6
    assert_eq!(stepped.next(), Some(9)); // index 8
    assert_eq!(stepped.next(), None);
}

#[test]
fn step_by_step_1() {
    // step_by(1) should yield all elements
    let mut stepped = VecLender::new(vec![1, 2, 3]).step_by(1);

    assert_eq!(stepped.next(), Some(1));
    assert_eq!(stepped.next(), Some(2));
    assert_eq!(stepped.next(), Some(3));
    assert_eq!(stepped.next(), None);
}

#[test]
fn step_by_step_larger_than_len() {
    // step_by(10) on 3 elements: only first element
    let mut stepped = VecLender::new(vec![1, 2, 3]).step_by(10);

    assert_eq!(stepped.next(), Some(1));
    assert_eq!(stepped.next(), None);
}

#[test]
fn step_by_empty() {
    let mut stepped = VecLender::new(vec![]).step_by(2);
    assert_eq!(stepped.next(), None);
}

#[test]
#[should_panic(expected = "assertion failed")]
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

    assert_eq!(stepped.next_back(), Some(5)); // last step position
    assert_eq!(stepped.next_back(), Some(3));
    assert_eq!(stepped.next_back(), Some(1));
    assert_eq!(stepped.next_back(), None);
}

#[test]
fn step_by_mixed_forward_backward() {
    // Mixed iteration
    let mut stepped = VecLender::new(vec![1, 2, 3, 4, 5, 6, 7]).step_by(2);
    // Positions: 0,2,4,6 -> values 1,3,5,7

    assert_eq!(stepped.next(), Some(1)); // front: 1
    assert_eq!(stepped.next_back(), Some(7)); // back: 7
    assert_eq!(stepped.next(), Some(3)); // front: 3
    assert_eq!(stepped.next_back(), Some(5)); // back: 5
    assert_eq!(stepped.next(), None);
    assert_eq!(stepped.next_back(), None);
}

#[test]
fn step_by_nth() {
    // nth on step_by should skip appropriately
    let mut stepped = VecLender::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9]).step_by(2);
    // Positions: 0,2,4,6,8 -> values 1,3,5,7,9

    // nth(0) is same as next()
    assert_eq!(stepped.nth(0), Some(1));
    // nth(1) skips one step position (3) and returns next (5)
    assert_eq!(stepped.nth(1), Some(5));
    // nth(0) returns 7
    assert_eq!(stepped.nth(0), Some(7));
    // nth(1) would need to skip 9 and get next, but only 9 left
    assert_eq!(stepped.nth(1), None);
}

#[test]
fn step_by_nth_back() {
    let mut stepped = VecLender::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9]).step_by(2);
    // Positions: 0,2,4,6,8 -> values 1,3,5,7,9

    // nth_back(0) returns last step position (9)
    assert_eq!(stepped.nth_back(0), Some(9));
    // nth_back(1) skips 7 and returns 5
    assert_eq!(stepped.nth_back(1), Some(5));
    // nth_back(0) returns 3
    assert_eq!(stepped.nth_back(0), Some(3));
    // nth_back(0) returns 1
    assert_eq!(stepped.nth_back(0), Some(1));
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
        .fold(0, |acc, x| acc + x);
    // Positions 0,2,4 -> values 1,3,5 -> sum = 9
    assert_eq!(sum, 9);
}

#[test]
fn step_by_rfold() {
    let mut order = Vec::new();
    VecLender::new(vec![1, 2, 3, 4, 5, 6])
        .step_by(2)
        .rfold((), |(), x| order.push(x));
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

// ============================================================================
// Peekable adapter tests (Lender)
// Semantics: peek() returns a reference to the next element without consuming it.
// ============================================================================

#[test]
fn peekable_basic() {
    let mut peekable = VecLender::new(vec![1, 2, 3]).peekable();

    // peek() returns reference to next element
    assert_eq!(peekable.peek(), Some(&1));
    // peek() again returns same element (not consumed)
    assert_eq!(peekable.peek(), Some(&1));
    // next() consumes it
    assert_eq!(peekable.next(), Some(1));

    // Now peek sees 2
    assert_eq!(peekable.peek(), Some(&2));
    assert_eq!(peekable.next(), Some(2));

    // Continue
    assert_eq!(peekable.peek(), Some(&3));
    assert_eq!(peekable.next(), Some(3));

    // Exhausted
    assert_eq!(peekable.peek(), None);
    assert_eq!(peekable.next(), None);
}

#[test]
fn peekable_peek_mut() {
    let mut peekable = VecLender::new(vec![1, 2, 3]).peekable();

    // peek_mut allows modifying the peeked value
    if let Some(val) = peekable.peek_mut() {
        *val = 100;
    }
    // next() returns the modified value
    assert_eq!(peekable.next(), Some(100));
    assert_eq!(peekable.next(), Some(2));
}

#[test]
fn peekable_next_if() {
    let mut peekable = VecLender::new(vec![1, 2, 3, 4]).peekable();

    // next_if returns Some if predicate matches
    assert_eq!(peekable.next_if(|&x| x == 1), Some(1));
    // next_if returns None if predicate doesn't match, element is put back
    assert_eq!(peekable.next_if(|&x| x == 10), None);
    // Element wasn't consumed
    assert_eq!(peekable.peek(), Some(&2));
    assert_eq!(peekable.next(), Some(2));
}

#[test]
fn peekable_next_if_eq() {
    let mut peekable = VecLender::new(vec![1, 2, 3]).peekable();

    // next_if_eq returns Some if element equals given value
    assert_eq!(peekable.next_if_eq(&1), Some(1));
    // next_if_eq returns None if not equal
    assert_eq!(peekable.next_if_eq(&10), None);
    assert_eq!(peekable.next(), Some(2));
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
    assert_eq!(peekable.nth(2), Some(3));
    assert_eq!(peekable.next(), Some(4));
}

#[test]
fn peekable_last() {
    let mut peekable = VecLender::new(vec![1, 2, 3]).peekable();
    assert_eq!(peekable.last(), Some(3));
}

#[test]
fn peekable_fold() {
    let sum = VecLender::new(vec![1, 2, 3, 4])
        .peekable()
        .fold(0, |acc, x| acc + x);
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

// ============================================================================
// Cycle adapter tests (Lender)
// Semantics: cycle() repeats the lender infinitely
// ============================================================================

#[test]
fn cycle_basic() {
    let mut cycled = VecLender::new(vec![1, 2, 3]).cycle();

    // First cycle
    assert_eq!(cycled.next(), Some(1));
    assert_eq!(cycled.next(), Some(2));
    assert_eq!(cycled.next(), Some(3));
    // Second cycle
    assert_eq!(cycled.next(), Some(1));
    assert_eq!(cycled.next(), Some(2));
    assert_eq!(cycled.next(), Some(3));
    // Third cycle
    assert_eq!(cycled.next(), Some(1));
}

#[test]
fn cycle_single_element() {
    let mut cycled = VecLender::new(vec![42]).cycle();

    for _ in 0..10 {
        assert_eq!(cycled.next(), Some(42));
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

// ============================================================================
// Enumerate adapter tests (Lender)
// Semantics: enumerate() pairs each element with its index starting from 0
// ============================================================================

#[test]
fn enumerate_basic() {
    let mut enumerated = VecLender::new(vec![10, 20, 30]).enumerate();

    assert_eq!(enumerated.next(), Some((0, 10)));
    assert_eq!(enumerated.next(), Some((1, 20)));
    assert_eq!(enumerated.next(), Some((2, 30)));
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
    assert_eq!(enumerated.next_back(), Some((3, 40)));
    assert_eq!(enumerated.next_back(), Some((2, 30)));
    assert_eq!(enumerated.next(), Some((0, 10)));
    assert_eq!(enumerated.next(), Some((1, 20)));
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
    assert_eq!(enumerated.nth(2), Some((2, 30)));
    // nth(0) returns next
    assert_eq!(enumerated.nth(0), Some((3, 40)));
}

#[test]
fn enumerate_nth_back() {
    let mut enumerated = VecLender::new(vec![10, 20, 30, 40, 50]).enumerate();

    // nth_back(1) skips (4,50), returns (3,40)
    assert_eq!(enumerated.nth_back(1), Some((3, 40)));
    // nth_back(0) returns (2,30)
    assert_eq!(enumerated.nth_back(0), Some((2, 30)));
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
            order.push((i, v));
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

// ============================================================================
// Core Lender trait method tests
// ============================================================================

#[test]
fn lender_advance_by() {
    use core::num::NonZeroUsize;

    let mut lender = VecLender::new(vec![1, 2, 3, 4, 5]);

    // advance_by(2) skips 2 elements
    assert_eq!(lender.advance_by(2), Ok(()));
    assert_eq!(lender.next(), Some(3));

    // advance_by with remaining elements
    assert_eq!(lender.advance_by(1), Ok(()));
    assert_eq!(lender.next(), Some(5));

    // advance_by past end returns Err with remaining count
    assert_eq!(lender.advance_by(5), Err(NonZeroUsize::new(5).unwrap()));
}

#[test]
fn lender_count() {
    let lender = VecLender::new(vec![1, 2, 3, 4, 5]);
    assert_eq!(lender.count(), 5);

    let empty = VecLender::new(vec![]);
    assert_eq!(empty.count(), 0);
}

#[test]
fn lender_last() {
    let mut lender = VecLender::new(vec![1, 2, 3]);
    assert_eq!(lender.last(), Some(3));

    let mut empty = VecLender::new(vec![]);
    assert_eq!(empty.last(), None);
}

#[test]
fn lender_for_each() {
    let mut collected = Vec::new();
    VecLender::new(vec![1, 2, 3]).for_each(|x| collected.push(x));
    assert_eq!(collected, vec![1, 2, 3]);
}

#[test]
fn lender_all() {
    assert!(VecLender::new(vec![2, 4, 6]).all(|x| x % 2 == 0));
    assert!(!VecLender::new(vec![2, 3, 6]).all(|x| x % 2 == 0));
    assert!(VecLender::new(vec![]).all(|_x: i32| false)); // vacuously true
}

#[test]
fn lender_any() {
    assert!(VecLender::new(vec![1, 2, 3]).any(|x| x == 2));
    assert!(!VecLender::new(vec![1, 2, 3]).any(|x| x == 10));
    assert!(!VecLender::new(vec![]).any(|_x: i32| true)); // vacuously false
}

#[test]
fn lender_find() {
    let mut lender = VecLender::new(vec![1, 2, 3, 4, 5]);
    assert_eq!(lender.find(|&x| x > 2), Some(3));
    assert_eq!(lender.find(|&x| x > 10), None);
}

#[test]
fn lender_find_map() {
    let mut lender = VecLender::new(vec![1, 2, 3, 4, 5]);
    let result = lender.find_map(|x| if x > 2 { Some(x * 10) } else { None });
    assert_eq!(result, Some(30));
}

#[test]
fn lender_position() {
    let mut lender = VecLender::new(vec![1, 2, 3, 4, 5]);
    assert_eq!(lender.position(|x| x == 3), Some(2));

    let mut lender2 = VecLender::new(vec![1, 2, 3]);
    assert_eq!(lender2.position(|x| x == 10), None);
}

#[test]
fn lender_max() {
    assert_eq!(VecLender::new(vec![1, 5, 3, 2, 4]).max(), Some(5));
    assert_eq!(VecLender::new(vec![]).max(), None);
    // Per Iterator::max docs: "If several elements are equally maximum, the last element is returned."
    assert_eq!(VecLender::new(vec![1, 3, 3, 1]).max(), Some(3));
}

#[test]
fn lender_min() {
    assert_eq!(VecLender::new(vec![3, 1, 5, 2, 4]).min(), Some(1));
    assert_eq!(VecLender::new(vec![]).min(), None);
    // Per Iterator::min docs: "If several elements are equally minimum, the first element is returned."
    assert_eq!(VecLender::new(vec![3, 1, 1, 3]).min(), Some(1));
}

#[test]
fn lender_max_by_key() {
    // max_by_key returns element with maximum key
    assert_eq!(
        VecLender::new(vec![-3, 0, 1, 5, -2]).max_by_key(|&x| x.abs()),
        Some(5)
    );
}

#[test]
fn lender_min_by_key() {
    // min_by_key returns element with minimum key
    assert_eq!(
        VecLender::new(vec![-3, 0, 1, 5, -2]).min_by_key(|&x| x.abs()),
        Some(0)
    );
}

#[test]
fn lender_max_by() {
    assert_eq!(
        VecLender::new(vec![1, 5, 3]).max_by(|a, b| a.cmp(b)),
        Some(5)
    );
    // Per Iterator::max_by docs: "If several elements are equally maximum, the last element is returned."
    // Use abs() comparison so that -3 and 3 are equal; last should win.
    assert_eq!(
        VecLender::new(vec![-3, 1, 3]).max_by(|a, b| a.abs().cmp(&b.abs())),
        Some(3)
    );
}

#[test]
fn lender_min_by() {
    assert_eq!(
        VecLender::new(vec![3, 1, 5]).min_by(|a, b| a.cmp(b)),
        Some(1)
    );
    // Per Iterator::min_by docs: "If several elements are equally minimum, the first element is returned."
    // Use abs() comparison so that -1 and 1 are equal; first should win.
    assert_eq!(
        VecLender::new(vec![3, -1, 1]).min_by(|a, b| a.abs().cmp(&b.abs())),
        Some(-1)
    );
}

// Note: sum() and product() require SumLender/ProductLender trait implementations
// which are complex to set up. They are tested through the doc examples.

// Note: The comparison methods (cmp, partial_cmp, eq, ne, lt, le, gt, ge) have complex
// HRTB trait bound requirements that make them difficult to test with generic lenders.
// They are tested indirectly through other adapter tests.

#[test]
fn lender_is_sorted() {
    assert!(VecLender::new(vec![1, 2, 3, 4]).is_sorted());
    assert!(VecLender::new(vec![1, 1, 2, 2]).is_sorted());
    assert!(!VecLender::new(vec![1, 3, 2]).is_sorted());
    assert!(VecLender::new(vec![]).is_sorted());
    assert!(VecLender::new(vec![1]).is_sorted());
}

#[test]
fn lender_is_sorted_by() {
    // Sorted in reverse order
    assert!(VecLender::new(vec![4, 3, 2, 1]).is_sorted_by(|a, b| Some(b.cmp(a))));
}

#[test]
fn lender_is_sorted_by_key() {
    // Sorted by absolute value
    assert!(VecLender::new(vec![0, -1, 2, -3]).is_sorted_by_key(|x| x.abs()));
}

// ============================================================================
// DoubleEndedLender trait method tests
// ============================================================================

#[test]
fn double_ended_lender_next_back() {
    let mut lender = VecLender::new(vec![1, 2, 3]);
    assert_eq!(lender.next_back(), Some(3));
    assert_eq!(lender.next_back(), Some(2));
    assert_eq!(lender.next_back(), Some(1));
    assert_eq!(lender.next_back(), None);
}

#[test]
fn double_ended_lender_advance_back_by() {
    use core::num::NonZeroUsize;

    let mut lender = VecLender::new(vec![1, 2, 3, 4, 5]);

    // advance_back_by(2) skips 2 elements from back
    assert_eq!(lender.advance_back_by(2), Ok(()));
    assert_eq!(lender.next_back(), Some(3));

    // advance_back_by past remaining
    assert_eq!(
        lender.advance_back_by(5),
        Err(NonZeroUsize::new(3).unwrap())
    );
}

#[test]
fn double_ended_lender_nth_back() {
    let mut lender = VecLender::new(vec![1, 2, 3, 4, 5]);

    // nth_back(1) skips 5 and returns 4
    assert_eq!(lender.nth_back(1), Some(4));
    // nth_back(0) returns 3
    assert_eq!(lender.nth_back(0), Some(3));
}

#[test]
fn double_ended_lender_rfind() {
    let mut lender = VecLender::new(vec![1, 2, 3, 4, 5]);
    // rfind searches from back
    assert_eq!(lender.rfind(|&x| x < 4), Some(3));
}

#[test]
fn double_ended_lender_rposition() {
    let mut lender = VecLender::new(vec![1, 2, 3, 2, 1]);
    // rposition searches from the end but returns a front-based index
    // Element 2 appears at indices 1 and 3; searching from the end finds index 3 first
    assert_eq!(lender.rposition(|x| x == 2), Some(3));
}

// ============================================================================
// Additional adapter tests for coverage
// ============================================================================

#[test]
fn filter_basic() {
    let mut filtered = VecLender::new(vec![1, 2, 3, 4, 5, 6]).filter(|&x| x % 2 == 0);

    assert_eq!(filtered.next(), Some(2));
    assert_eq!(filtered.next(), Some(4));
    assert_eq!(filtered.next(), Some(6));
    assert_eq!(filtered.next(), None);
}

#[test]
fn filter_empty_result() {
    let mut filtered = VecLender::new(vec![1, 3, 5]).filter(|&x| x % 2 == 0);
    assert_eq!(filtered.next(), None);
}

#[test]
fn skip_basic() {
    let mut skipped = VecLender::new(vec![1, 2, 3, 4, 5]).skip(2);

    assert_eq!(skipped.next(), Some(3));
    assert_eq!(skipped.next(), Some(4));
    assert_eq!(skipped.next(), Some(5));
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
    assert_eq!(skipped.next_back(), Some(5));
    assert_eq!(skipped.next_back(), Some(4));
    assert_eq!(skipped.next_back(), Some(3));
    assert_eq!(skipped.next_back(), None);
}

#[test]
fn take_basic() {
    let mut taken = VecLender::new(vec![1, 2, 3, 4, 5]).take(3);

    assert_eq!(taken.next(), Some(1));
    assert_eq!(taken.next(), Some(2));
    assert_eq!(taken.next(), Some(3));
    assert_eq!(taken.next(), None);
}

#[test]
fn take_more_than_length() {
    let mut taken = VecLender::new(vec![1, 2]).take(10);

    assert_eq!(taken.next(), Some(1));
    assert_eq!(taken.next(), Some(2));
    assert_eq!(taken.next(), None);
}

#[test]
fn take_double_ended() {
    let mut taken = VecLender::new(vec![1, 2, 3, 4, 5]).take(3);

    // next_back returns from back of taken portion
    assert_eq!(taken.next_back(), Some(3));
    assert_eq!(taken.next_back(), Some(2));
    assert_eq!(taken.next_back(), Some(1));
    assert_eq!(taken.next_back(), None);
}

#[test]
fn zip_basic() {
    let mut zipped = VecLender::new(vec![1, 2, 3]).zip(VecLender::new(vec![10, 20, 30]));

    assert_eq!(zipped.next(), Some((1, 10)));
    assert_eq!(zipped.next(), Some((2, 20)));
    assert_eq!(zipped.next(), Some((3, 30)));
    assert_eq!(zipped.next(), None);
}

#[test]
fn zip_different_lengths() {
    // Stops at shorter lender
    let mut zipped = VecLender::new(vec![1, 2]).zip(VecLender::new(vec![10, 20, 30]));

    assert_eq!(zipped.next(), Some((1, 10)));
    assert_eq!(zipped.next(), Some((2, 20)));
    assert_eq!(zipped.next(), None);
}

#[test]
fn zip_double_ended() {
    let mut zipped = VecLender::new(vec![1, 2, 3]).zip(VecLender::new(vec![10, 20, 30]));

    assert_eq!(zipped.next_back(), Some((3, 30)));
    assert_eq!(zipped.next_back(), Some((2, 20)));
    assert_eq!(zipped.next_back(), Some((1, 10)));
    assert_eq!(zipped.next_back(), None);
}

#[test]
fn rev_basic() {
    let mut reversed = VecLender::new(vec![1, 2, 3]).rev();

    assert_eq!(reversed.next(), Some(3));
    assert_eq!(reversed.next(), Some(2));
    assert_eq!(reversed.next(), Some(1));
    assert_eq!(reversed.next(), None);
}

#[test]
fn rev_double_ended() {
    let mut reversed = VecLender::new(vec![1, 2, 3]).rev();

    // next_back on Rev is next on original
    assert_eq!(reversed.next_back(), Some(1));
    assert_eq!(reversed.next_back(), Some(2));
}

#[test]
fn inspect_basic() {
    let mut inspected_values = Vec::new();
    let mut lender = VecLender::new(vec![1, 2, 3]).inspect(|&x| inspected_values.push(x));

    assert_eq!(lender.next(), Some(1));
    assert_eq!(lender.next(), Some(2));
    assert_eq!(lender.next(), Some(3));

    assert_eq!(inspected_values, vec![1, 2, 3]);
}

#[test]
fn mutate_basic() {
    let mut lender = VecLender::new(vec![1, 2, 3]).mutate(|x| *x *= 10);

    assert_eq!(lender.next(), Some(10));
    assert_eq!(lender.next(), Some(20));
    assert_eq!(lender.next(), Some(30));
}

// ============================================================================
// SkipWhile adapter tests
// Semantics: skip elements while predicate is true, then yield all remaining
// ============================================================================

#[test]
fn skip_while_basic() {
    let mut skipped = VecLender::new(vec![1, 2, 3, 4, 5]).skip_while(|&x| x < 3);

    // Skips 1, 2; yields 3, 4, 5
    assert_eq!(skipped.next(), Some(3));
    assert_eq!(skipped.next(), Some(4));
    assert_eq!(skipped.next(), Some(5));
    assert_eq!(skipped.next(), None);
}

#[test]
fn skip_while_none_skipped() {
    // Predicate is false from the start
    let mut skipped = VecLender::new(vec![5, 4, 3, 2, 1]).skip_while(|&x| x < 3);

    assert_eq!(skipped.next(), Some(5));
    assert_eq!(skipped.next(), Some(4));
}

#[test]
fn skip_while_all_skipped() {
    // Predicate is always true
    let mut skipped = VecLender::new(vec![1, 2, 3]).skip_while(|&x| x < 10);
    assert_eq!(skipped.next(), None);
}

#[test]
fn skip_while_empty() {
    let mut skipped = VecLender::new(vec![]).skip_while(|&x: &i32| x < 3);
    assert_eq!(skipped.next(), None);
}

#[test]
fn skip_while_fold() {
    let sum = VecLender::new(vec![1, 2, 3, 4, 5])
        .skip_while(|&x| x < 3)
        .fold(0, |acc, x| acc + x);
    // 3 + 4 + 5 = 12
    assert_eq!(sum, 12);
}

// ============================================================================
// TakeWhile adapter tests
// Semantics: yield elements while predicate is true, then stop
// ============================================================================

#[test]
fn take_while_basic() {
    let mut taken = VecLender::new(vec![1, 2, 3, 4, 5]).take_while(|&x| x < 4);

    assert_eq!(taken.next(), Some(1));
    assert_eq!(taken.next(), Some(2));
    assert_eq!(taken.next(), Some(3));
    assert_eq!(taken.next(), None);
    // Should stay None even though 4, 5 remain in underlying lender
    assert_eq!(taken.next(), None);
}

#[test]
fn take_while_none_taken() {
    // Predicate is false from the start
    let mut taken = VecLender::new(vec![5, 4, 3]).take_while(|&x| x < 3);
    assert_eq!(taken.next(), None);
}

#[test]
fn take_while_all_taken() {
    // Predicate is always true
    let mut taken = VecLender::new(vec![1, 2, 3]).take_while(|&x| x < 10);

    assert_eq!(taken.next(), Some(1));
    assert_eq!(taken.next(), Some(2));
    assert_eq!(taken.next(), Some(3));
    assert_eq!(taken.next(), None);
}

#[test]
fn take_while_empty() {
    let mut taken = VecLender::new(vec![]).take_while(|&x: &i32| x < 3);
    assert_eq!(taken.next(), None);
}

#[test]
fn take_while_fold() {
    let sum = VecLender::new(vec![1, 2, 3, 4, 5])
        .take_while(|&x| x < 4)
        .fold(0, |acc, x| acc + x);
    // 1 + 2 + 3 = 6
    assert_eq!(sum, 6);
}

// ============================================================================
// FilterMap adapter tests
// Semantics: filter and map in one operation
// ============================================================================

#[test]
fn filter_map_basic() {
    let mut fm = VecLender::new(vec![1, 2, 3, 4, 5])
        .filter_map(|x| if x % 2 == 0 { Some(x * 10) } else { None });

    assert_eq!(fm.next(), Some(20));
    assert_eq!(fm.next(), Some(40));
    assert_eq!(fm.next(), None);
}

#[test]
fn filter_map_all_none() {
    let mut fm =
        VecLender::new(vec![1, 3, 5]).filter_map(|x| if x % 2 == 0 { Some(x * 10) } else { None });
    assert_eq!(fm.next(), None);
}

#[test]
fn filter_map_all_some() {
    let mut fm = VecLender::new(vec![2, 4, 6]).filter_map(|x| Some(x / 2));

    assert_eq!(fm.next(), Some(1));
    assert_eq!(fm.next(), Some(2));
    assert_eq!(fm.next(), Some(3));
    assert_eq!(fm.next(), None);
}

#[test]
fn filter_map_fold() {
    let sum = VecLender::new(vec![1, 2, 3, 4, 5])
        .filter_map(|x| if x % 2 == 0 { Some(x) } else { None })
        .fold(0, |acc, x| acc + x);
    // 2 + 4 = 6
    assert_eq!(sum, 6);
}

// ============================================================================
// Scan adapter tests
// Semantics: like fold but yields intermediate states
// Note: scan requires hrc_mut! macro for the closure
// ============================================================================

#[test]
fn scan_basic() {
    let mut scanned = VecLender::new(vec![1, 2, 3]).scan(
        0i32,
        hrc_mut!(for<'all> |args: (&'all mut i32, i32)| -> Option<i32> {
            *args.0 += args.1;
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
        0i32,
        hrc_mut!(for<'all> |args: (&'all mut i32, i32)| -> Option<i32> {
            *args.0 += args.1;
            if *args.0 > 5 { None } else { Some(*args.0) }
        }),
    );

    assert_eq!(scanned.next(), Some(1)); // state = 1
    assert_eq!(scanned.next(), Some(3)); // state = 3
    // x=3: state=6, 6 > 5, return None
    assert_eq!(scanned.next(), None);
}

// ============================================================================
// MapWhile adapter tests
// Semantics: like map but stops when function returns None
// Note: map_while requires hrc_mut! macro for the closure
// ============================================================================

#[test]
fn map_while_basic() {
    let mut mw = VecLender::new(vec![1, 2, 3, 4, 5]).map_while(hrc_mut!(
        for<'all> |x: i32| -> Option<i32> { if x < 4 { Some(x * 10) } else { None } }
    ));

    assert_eq!(mw.next(), Some(10));
    assert_eq!(mw.next(), Some(20));
    assert_eq!(mw.next(), Some(30));
    assert_eq!(mw.next(), None);
}

#[test]
fn map_while_all_mapped() {
    let mut mw = VecLender::new(vec![1, 2, 3])
        .map_while(hrc_mut!(for<'all> |x: i32| -> Option<i32> { Some(x * 2) }));

    assert_eq!(mw.next(), Some(2));
    assert_eq!(mw.next(), Some(4));
    assert_eq!(mw.next(), Some(6));
    assert_eq!(mw.next(), None);
}

#[test]
fn map_while_immediate_none() {
    let mut mw =
        VecLender::new(vec![5, 4, 3]).map_while(hrc_mut!(for<'all> |x: i32| -> Option<i32> {
            if x < 4 { Some(x) } else { None }
        }));
    assert_eq!(mw.next(), None);
}

// ============================================================================
// Intersperse adapter tests (Lender)
// Semantics: insert separator between elements
// ============================================================================

#[test]
fn intersperse_basic() {
    let mut interspersed = VecLender::new(vec![1, 2, 3]).intersperse(0);

    assert_eq!(interspersed.next(), Some(1));
    assert_eq!(interspersed.next(), Some(0)); // separator
    assert_eq!(interspersed.next(), Some(2));
    assert_eq!(interspersed.next(), Some(0)); // separator
    assert_eq!(interspersed.next(), Some(3));
    assert_eq!(interspersed.next(), None);
}

#[test]
fn intersperse_single_element() {
    let mut interspersed = VecLender::new(vec![42]).intersperse(0);

    assert_eq!(interspersed.next(), Some(42));
    assert_eq!(interspersed.next(), None);
}

#[test]
fn intersperse_empty() {
    let mut interspersed = VecLender::new(vec![]).intersperse(0);
    assert_eq!(interspersed.next(), None);
}

#[test]
fn intersperse_fold() {
    let sum = VecLender::new(vec![1, 2, 3])
        .intersperse(10)
        .fold(0, |acc, x| acc + x);
    // 1 + 10 + 2 + 10 + 3 = 26
    assert_eq!(sum, 26);
}

#[test]
fn intersperse_with_basic() {
    let mut counter = 0;
    let mut interspersed = VecLender::new(vec![1, 2, 3]).intersperse_with(|| {
        counter += 1;
        counter * 10
    });

    assert_eq!(interspersed.next(), Some(1));
    assert_eq!(interspersed.next(), Some(10)); // counter = 1
    assert_eq!(interspersed.next(), Some(2));
    assert_eq!(interspersed.next(), Some(20)); // counter = 2
    assert_eq!(interspersed.next(), Some(3));
    assert_eq!(interspersed.next(), None);
}

// ============================================================================
// Chunk adapter tests
// ============================================================================

#[test]
fn chunk_basic() {
    let mut lender = VecLender::new(vec![1, 2, 3, 4, 5]);
    let mut chunk = lender.next_chunk(3);

    assert_eq!(chunk.next(), Some(1));
    assert_eq!(chunk.next(), Some(2));
    assert_eq!(chunk.next(), Some(3));
    assert_eq!(chunk.next(), None);

    // Remaining elements
    assert_eq!(lender.next(), Some(4));
    assert_eq!(lender.next(), Some(5));
}

#[test]
fn chunk_larger_than_remaining() {
    let mut lender = VecLender::new(vec![1, 2]);
    let mut chunk = lender.next_chunk(5);

    assert_eq!(chunk.next(), Some(1));
    assert_eq!(chunk.next(), Some(2));
    assert_eq!(chunk.next(), None);
}

#[test]
fn chunk_empty_lender() {
    let mut lender = VecLender::new(vec![]);
    let mut chunk = lender.next_chunk(3);
    assert_eq!(chunk.next(), None);
}

// Note: cloned/copied adapters have complex trait bounds that require specific
// Lend types. They are tested through the iter() conversion which uses these.

// ============================================================================
// Map adapter additional tests
// ============================================================================

#[test]
fn map_basic() {
    let mut mapped = VecLender::new(vec![1, 2, 3]).map(|x| x * 2);

    assert_eq!(mapped.next(), Some(2));
    assert_eq!(mapped.next(), Some(4));
    assert_eq!(mapped.next(), Some(6));
    assert_eq!(mapped.next(), None);
}

#[test]
fn map_fold() {
    let sum = VecLender::new(vec![1, 2, 3])
        .map(|x| x * 10)
        .fold(0, |acc, x| acc + x);
    assert_eq!(sum, 60);
}

#[test]
fn map_double_ended() {
    let mut mapped = VecLender::new(vec![1, 2, 3]).map(|x| x * 10);

    assert_eq!(mapped.next_back(), Some(30));
    assert_eq!(mapped.next(), Some(10));
    assert_eq!(mapped.next_back(), Some(20));
    assert_eq!(mapped.next(), None);
}

// ============================================================================
// Additional Skip/Take tests for better coverage
// ============================================================================

#[test]
fn skip_nth() {
    let mut skipped = VecLender::new(vec![1, 2, 3, 4, 5]).skip(2);
    // After skip(2), elements are 3, 4, 5
    // nth(1) skips 3 and returns 4
    assert_eq!(skipped.nth(1), Some(4));
    assert_eq!(skipped.next(), Some(5));
}

#[test]
fn skip_fold() {
    let sum = VecLender::new(vec![1, 2, 3, 4, 5])
        .skip(2)
        .fold(0, |acc, x| acc + x);
    // 3 + 4 + 5 = 12
    assert_eq!(sum, 12);
}

#[test]
fn take_nth() {
    let mut taken = VecLender::new(vec![1, 2, 3, 4, 5]).take(4);
    // nth(2) skips 1, 2 and returns 3
    assert_eq!(taken.nth(2), Some(3));
    assert_eq!(taken.next(), Some(4));
    assert_eq!(taken.next(), None);
}

#[test]
fn take_fold() {
    let sum = VecLender::new(vec![1, 2, 3, 4, 5])
        .take(3)
        .fold(0, |acc, x| acc + x);
    // 1 + 2 + 3 = 6
    assert_eq!(sum, 6);
}

// ============================================================================
// Rev adapter additional tests
// ============================================================================

#[test]
fn rev_fold() {
    let mut order = Vec::new();
    VecLender::new(vec![1, 2, 3])
        .rev()
        .fold((), |(), x| order.push(x));
    assert_eq!(order, vec![3, 2, 1]);
}

#[test]
fn rev_rfold() {
    let mut order = Vec::new();
    VecLender::new(vec![1, 2, 3])
        .rev()
        .rfold((), |(), x| order.push(x));
    // rfold on Rev goes forward on original
    assert_eq!(order, vec![1, 2, 3]);
}

#[test]
fn rev_nth() {
    let mut reversed = VecLender::new(vec![1, 2, 3, 4, 5]).rev();
    // nth(2) on reversed: skips 5, 4 and returns 3
    assert_eq!(reversed.nth(2), Some(3));
}

#[test]
fn rev_nth_back() {
    let mut reversed = VecLender::new(vec![1, 2, 3, 4, 5]).rev();
    // nth_back(1) on reversed: skips 1 and returns 2
    assert_eq!(reversed.nth_back(1), Some(2));
}

// ============================================================================
// Zip adapter additional tests
// ============================================================================

#[test]
fn zip_fold() {
    let sum = VecLender::new(vec![1, 2, 3])
        .zip(VecLender::new(vec![10, 20, 30]))
        .fold(0, |acc, (a, b)| acc + a + b);
    // (1+10) + (2+20) + (3+30) = 66
    assert_eq!(sum, 66);
}

#[test]
fn zip_nth() {
    let mut zipped = VecLender::new(vec![1, 2, 3, 4]).zip(VecLender::new(vec![10, 20, 30, 40]));
    assert_eq!(zipped.nth(2), Some((3, 30)));
    assert_eq!(zipped.next(), Some((4, 40)));
}

// ============================================================================
// Inspect adapter additional tests
// ============================================================================

#[test]
fn inspect_fold() {
    let mut inspected = Vec::new();
    let sum = VecLender::new(vec![1, 2, 3])
        .inspect(|&x| inspected.push(x))
        .fold(0, |acc, x| acc + x);

    assert_eq!(sum, 6);
    assert_eq!(inspected, vec![1, 2, 3]);
}

#[test]
fn inspect_double_ended() {
    let mut inspected = Vec::new();
    let mut lender = VecLender::new(vec![1, 2, 3]).inspect(|&x| inspected.push(x));

    assert_eq!(lender.next_back(), Some(3));
    assert_eq!(lender.next(), Some(1));
    assert_eq!(inspected, vec![3, 1]);
}

// ============================================================================
// Filter adapter additional tests
// ============================================================================

#[test]
fn filter_fold() {
    let sum = VecLender::new(vec![1, 2, 3, 4, 5, 6])
        .filter(|&x| x % 2 == 0)
        .fold(0, |acc, x| acc + x);
    // 2 + 4 + 6 = 12
    assert_eq!(sum, 12);
}

#[test]
fn filter_double_ended() {
    let mut filtered = VecLender::new(vec![1, 2, 3, 4, 5, 6]).filter(|&x| x % 2 == 0);

    assert_eq!(filtered.next_back(), Some(6));
    assert_eq!(filtered.next(), Some(2));
    assert_eq!(filtered.next_back(), Some(4));
    assert_eq!(filtered.next(), None);
}

// ============================================================================
// Mutate adapter additional tests
// ============================================================================

#[test]
fn mutate_fold() {
    let sum = VecLender::new(vec![1, 2, 3])
        .mutate(|x| *x *= 10)
        .fold(0, |acc, x| acc + x);
    assert_eq!(sum, 60);
}

#[test]
fn mutate_double_ended() {
    let mut mutated = VecLender::new(vec![1, 2, 3]).mutate(|x| *x += 100);

    assert_eq!(mutated.next_back(), Some(103));
    assert_eq!(mutated.next(), Some(101));
    assert_eq!(mutated.next_back(), Some(102));
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
    assert_eq!(chunk1.next(), Some(1));
    assert_eq!(chunk1.next(), Some(2));
    assert_eq!(chunk1.next(), None);

    // Second chunk: 3, 4
    let mut chunk2 = chunky.next().unwrap();
    assert_eq!(chunk2.next(), Some(3));
    assert_eq!(chunk2.next(), Some(4));
    assert_eq!(chunk2.next(), None);

    // Third chunk: 5, 6
    let mut chunk3 = chunky.next().unwrap();
    assert_eq!(chunk3.next(), Some(5));
    assert_eq!(chunk3.next(), Some(6));
    assert_eq!(chunk3.next(), None);

    // No more chunks
    assert!(chunky.next().is_none());
}

#[test]
fn chunky_uneven() {
    // 5 elements with chunk size 2 = 3 chunks (2, 2, 1)
    let mut chunky = VecLender::new(vec![1, 2, 3, 4, 5]).chunky(2);

    let mut chunk1 = chunky.next().unwrap();
    assert_eq!(chunk1.next(), Some(1));
    assert_eq!(chunk1.next(), Some(2));
    assert_eq!(chunk1.next(), None);

    let mut chunk2 = chunky.next().unwrap();
    assert_eq!(chunk2.next(), Some(3));
    assert_eq!(chunk2.next(), Some(4));
    assert_eq!(chunk2.next(), None);

    // Last chunk has only 1 element
    let mut chunk3 = chunky.next().unwrap();
    assert_eq!(chunk3.next(), Some(5));
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
fn chunky_exact_size() {
    use lender::ExactSizeLender;

    let mut chunky = VecLender::new(vec![1, 2, 3, 4, 5, 6]).chunky(2);
    // 6 elements, chunk_size 2 -> 3 chunks
    assert_eq!(chunky.len(), 3);
    chunky.next();
    assert_eq!(chunky.len(), 2);
    chunky.next();
    assert_eq!(chunky.len(), 1);
    chunky.next();
    assert_eq!(chunky.len(), 0);

    // With uneven division: 5 elements, chunk_size 2 -> ceil(5/2) = 3 chunks
    let chunky2 = VecLender::new(vec![1, 2, 3, 4, 5]).chunky(2);
    assert_eq!(chunky2.len(), 3);
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
// Sources tests - empty, once, repeat, from_fn, etc.
// ============================================================================

#[test]
fn source_empty_basic() {
    let mut empty = lender::empty::<lend!(i32)>();
    assert_eq!(empty.next(), None);
    assert_eq!(empty.next(), None); // Fused
}

#[test]
fn source_empty_size_hint() {
    let empty = lender::empty::<lend!(i32)>();
    assert_eq!(empty.size_hint(), (0, Some(0)));
}

#[test]
fn source_empty_count() {
    let empty = lender::empty::<lend!(i32)>();
    assert_eq!(empty.count(), 0);
}

#[test]
fn source_empty_double_ended() {
    let mut empty = lender::empty::<lend!(i32)>();
    assert_eq!(empty.next_back(), None);
}

#[test]
fn source_once_basic() {
    let mut once = lender::once::<lend!(i32)>(42);
    assert_eq!(once.next(), Some(42));
    assert_eq!(once.next(), None);
    assert_eq!(once.next(), None); // Fused
}

#[test]
fn source_once_size_hint() {
    let once = lender::once::<lend!(i32)>(42);
    assert_eq!(once.size_hint(), (1, Some(1)));
}

#[test]
fn source_once_count() {
    let once = lender::once::<lend!(i32)>(42);
    assert_eq!(once.count(), 1);
}

#[test]
fn source_once_double_ended() {
    let mut once = lender::once::<lend!(i32)>(42);
    assert_eq!(once.next_back(), Some(42));
    assert_eq!(once.next_back(), None);
}

#[test]
fn source_once_fold() {
    let sum = lender::once::<lend!(i32)>(42).fold(0, |acc, x| acc + x);
    assert_eq!(sum, 42);
}

#[test]
fn source_repeat_basic() {
    let mut repeat = lender::repeat::<lend!(i32)>(42);
    assert_eq!(repeat.next(), Some(42));
    assert_eq!(repeat.next(), Some(42));
    assert_eq!(repeat.next(), Some(42));
    // Infinite - continues forever
}

#[test]
fn source_repeat_take() {
    let sum = lender::repeat::<lend!(i32)>(5)
        .take(4)
        .fold(0, |acc, x| acc + x);
    assert_eq!(sum, 20); // 5 * 4
}

#[test]
fn source_repeat_double_ended() {
    let mut repeat = lender::repeat::<lend!(i32)>(42);
    // Repeat is double-ended (infinite both ways)
    assert_eq!(repeat.next_back(), Some(42));
    assert_eq!(repeat.next_back(), Some(42));
}

#[test]
fn source_repeat_size_hint() {
    // Per Iterator::repeat docs: size_hint returns (usize::MAX, None) for infinite iterators
    let repeat = lender::repeat::<lend!(i32)>(42);
    assert_eq!(repeat.size_hint(), (usize::MAX, None));
}

#[test]
fn source_repeat_with_basic() {
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
fn source_repeat_with_size_hint() {
    let repeat_with = lender::repeat_with::<lend!(i32), _>(|| 42);
    assert_eq!(repeat_with.size_hint(), (usize::MAX, None));
}

#[test]
fn source_from_fn_basic() {
    let mut from_fn = lender::from_fn(
        0i32,
        hrc_mut!(for<'all> |s: &'all mut i32| -> Option<i32> {
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
fn source_once_with_basic() {
    let mut once_with = lender::once_with(
        42u8,
        hrc_once!(for<'lend> |state: &'lend mut u8| -> &'lend mut u8 {
            *state += 1;
            state
        }),
    );

    assert_eq!(once_with.next(), Some(&mut 43));
    assert_eq!(once_with.next(), None);
}

// ============================================================================
// FromIter source tests
// ============================================================================

#[test]
fn from_iter_basic() {
    let mut lender = lender::from_iter(vec![1, 2, 3].into_iter());

    assert_eq!(lender.next(), Some(1));
    assert_eq!(lender.next(), Some(2));
    assert_eq!(lender.next(), Some(3));
    assert_eq!(lender.next(), None);
}

#[test]
fn from_iter_size_hint() {
    let lender = lender::from_iter(vec![1, 2, 3].into_iter());
    assert_eq!(lender.size_hint(), (3, Some(3)));
}

#[test]
fn from_iter_double_ended() {
    let mut lender = lender::from_iter(vec![1, 2, 3].into_iter());

    assert_eq!(lender.next_back(), Some(3));
    assert_eq!(lender.next(), Some(1));
    assert_eq!(lender.next_back(), Some(2));
    assert_eq!(lender.next(), None);
}

#[test]
fn lend_iter_basic() {
    let data = vec![1, 2, 3];
    let mut lender = lender::lend_iter::<lend!(&'lend i32), _>(data.iter());

    assert_eq!(lender.next(), Some(&1));
    assert_eq!(lender.next(), Some(&2));
    assert_eq!(lender.next(), Some(&3));
    assert_eq!(lender.next(), None);
}

// ============================================================================
// Flatten adapter additional tests
// ============================================================================

/// Helper lender that yields VecLenders
struct VecOfVecLender {
    data: Vec<Vec<i32>>,
    index: usize,
}

impl VecOfVecLender {
    fn new(data: Vec<Vec<i32>>) -> Self {
        Self { data, index: 0 }
    }
}

impl<'lend> Lending<'lend> for VecOfVecLender {
    type Lend = VecLender;
}

impl Lender for VecOfVecLender {
    check_covariance!();

    fn next(&mut self) -> Option<Lend<'_, Self>> {
        if self.index < self.data.len() {
            let vec = self.data[self.index].clone();
            self.index += 1;
            Some(VecLender::new(vec))
        } else {
            None
        }
    }
}

#[test]
fn flatten_basic() {
    let mut flattened = VecOfVecLender::new(vec![vec![1, 2], vec![3, 4], vec![5]]).flatten();

    assert_eq!(flattened.next(), Some(1));
    assert_eq!(flattened.next(), Some(2));
    assert_eq!(flattened.next(), Some(3));
    assert_eq!(flattened.next(), Some(4));
    assert_eq!(flattened.next(), Some(5));
    assert_eq!(flattened.next(), None);
}

#[test]
fn flatten_empty_inner() {
    let mut flattened = VecOfVecLender::new(vec![vec![1], vec![], vec![2, 3]]).flatten();

    assert_eq!(flattened.next(), Some(1));
    assert_eq!(flattened.next(), Some(2));
    assert_eq!(flattened.next(), Some(3));
    assert_eq!(flattened.next(), None);
}

#[test]
fn flatten_empty_outer() {
    let mut flattened = VecOfVecLender::new(vec![]).flatten();
    assert_eq!(flattened.next(), None);
}

// ============================================================================
// Additional double-ended tests for better coverage
// ============================================================================

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

#[test]
fn skip_rfold() {
    let mut values = Vec::new();
    VecLender::new(vec![1, 2, 3, 4, 5])
        .skip(2)
        .rfold((), |(), x| {
            values.push(x);
        });
    // skip(2) leaves [3, 4, 5], rfold processes: 5, 4, 3
    assert_eq!(values, vec![5, 4, 3]);
}

#[test]
fn take_rfold() {
    let mut values = Vec::new();
    VecLender::new(vec![1, 2, 3, 4, 5])
        .take(3)
        .rfold((), |(), x| {
            values.push(x);
        });
    // take(3) gives [1, 2, 3], rfold processes: 3, 2, 1
    assert_eq!(values, vec![3, 2, 1]);
}

#[test]
fn zip_rfold() {
    let mut values = Vec::new();
    VecLender::new(vec![1, 2, 3])
        .zip(VecLender::new(vec![10, 20, 30]))
        .rfold((), |(), (a, b)| {
            values.push((a, b));
        });
    assert_eq!(values, vec![(3, 30), (2, 20), (1, 10)]);
}

// ============================================================================
// Try operations tests
// ============================================================================

#[test]
fn lender_try_for_each() {
    let mut sum = 0;
    let result: Result<(), &str> = VecLender::new(vec![1, 2, 3]).try_for_each(|x| {
        sum += x;
        Ok(())
    });
    assert!(result.is_ok());
    assert_eq!(sum, 6);
}

#[test]
fn lender_try_for_each_early_exit() {
    let mut sum = 0;
    let result: Result<(), &str> = VecLender::new(vec![1, 2, 3, 4, 5]).try_for_each(|x| {
        if x > 3 {
            Err("too big")
        } else {
            sum += x;
            Ok(())
        }
    });
    assert_eq!(result, Err("too big"));
    assert_eq!(sum, 6); // 1 + 2 + 3
}

#[test]
fn lender_try_fold() {
    let result: Result<i32, &str> = VecLender::new(vec![1, 2, 3]).try_fold(0, |acc, x| Ok(acc + x));
    assert_eq!(result, Ok(6));
}

#[test]
fn lender_try_fold_early_exit() {
    let result: Result<i32, &str> = VecLender::new(vec![1, 2, 3, 4, 5])
        .try_fold(0, |acc, x| if x > 3 { Err("too big") } else { Ok(acc + x) });
    assert_eq!(result, Err("too big"));
}

// ============================================================================
// Additional adapter tests for better coverage
// ============================================================================

#[test]
fn filter_size_hint() {
    let filtered = VecLender::new(vec![1, 2, 3, 4, 5]).filter(|&x| x % 2 == 0);
    // Filter can't know exact count, so lower is 0
    let (lower, upper) = filtered.size_hint();
    assert_eq!(lower, 0);
    assert_eq!(upper, Some(5));
}

#[test]
fn skip_size_hint() {
    let skipped = VecLender::new(vec![1, 2, 3, 4, 5]).skip(2);
    assert_eq!(skipped.size_hint(), (3, Some(3)));
}

#[test]
fn skip_exact_size() {
    use lender::ExactSizeLender;

    let mut skipped = VecLender::new(vec![1, 2, 3, 4, 5]).skip(2);
    assert_eq!(skipped.len(), 3);
    skipped.next();
    assert_eq!(skipped.len(), 2);
}

#[test]
fn take_size_hint() {
    let taken = VecLender::new(vec![1, 2, 3, 4, 5]).take(3);
    assert_eq!(taken.size_hint(), (3, Some(3)));
}

#[test]
fn take_exact_size() {
    use lender::ExactSizeLender;

    let mut taken = VecLender::new(vec![1, 2, 3, 4, 5]).take(3);
    assert_eq!(taken.len(), 3);
    taken.next();
    assert_eq!(taken.len(), 2);
}

#[test]
fn map_size_hint() {
    let mapped = VecLender::new(vec![1, 2, 3]).map(|x| x * 2);
    assert_eq!(mapped.size_hint(), (3, Some(3)));
}

#[test]
fn inspect_double_ended_fold() {
    let mut inspected = Vec::new();
    let values: Vec<i32> = VecLender::new(vec![1, 2, 3])
        .inspect(|&x| inspected.push(x))
        .rfold(Vec::new(), |mut acc, x| {
            acc.push(x);
            acc
        });
    assert_eq!(values, vec![3, 2, 1]);
    assert_eq!(inspected, vec![3, 2, 1]);
}

// ============================================================================
// Windows_mut additional tests
// ============================================================================

#[test]
fn windows_mut_fold() {
    let mut data = [0, 1, 2, 3, 4];
    let count = lender::windows_mut(&mut data, 2).fold(0, |acc, _window| acc + 1);
    assert_eq!(count, 4); // 4 windows of size 2 in array of 5
}

#[test]
fn windows_mut_rfold() {
    let mut data = [1, 2, 3, 4];
    let mut windows = Vec::new();
    lender::windows_mut(&mut data, 2).rfold((), |(), w| {
        windows.push(w.to_vec());
    });
    // Windows in reverse: [3,4], [2,3], [1,2]
    assert_eq!(windows, vec![vec![3, 4], vec![2, 3], vec![1, 2]]);
}

#[test]
fn array_windows_mut_fold() {
    let mut data = [0, 1, 2, 3, 4];
    let count = lender::array_windows_mut::<_, 2>(&mut data).fold(0, |acc, _window| acc + 1);
    assert_eq!(count, 4);
}

#[test]
fn array_windows_mut_rfold() {
    let mut data = [1, 2, 3, 4];
    let mut first_elements = Vec::new();
    lender::array_windows_mut::<_, 2>(&mut data).rfold((), |(), w| {
        first_elements.push(w[0]);
    });
    // Windows in reverse: [3,4], [2,3], [1,2] -> first elements: 3, 2, 1
    assert_eq!(first_elements, vec![3, 2, 1]);
}

// ============================================================================
// Fallible lender tests (using into_fallible)
// ============================================================================

#[test]
fn fallible_into_fallible_basic() {
    use lender::FallibleLender;

    let mut fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2, 3]).into_fallible();

    assert_eq!(fallible.next(), Ok(Some(1)));
    assert_eq!(fallible.next(), Ok(Some(2)));
    assert_eq!(fallible.next(), Ok(Some(3)));
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

    assert_eq!(fallible.next_back(), Ok(Some(3)));
    assert_eq!(fallible.next(), Ok(Some(1)));
    assert_eq!(fallible.next_back(), Ok(Some(2)));
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

    let result: Result<Option<i32>, ()> = fallible.try_fold(0, |acc, x| Ok(Some(acc + x)));
    assert_eq!(result, Ok(Some(6)));
}

#[test]
fn fallible_into_fallible_try_rfold() {
    use lender::DoubleEndedFallibleLender;

    let mut fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2, 3]).into_fallible();

    let result: Result<Option<i32>, ()> = fallible.try_rfold(0, |acc, x| Ok(Some(acc + x)));
    assert_eq!(result, Ok(Some(6)));
}

#[test]
fn fallible_into_inner() {
    let fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2, 3]).into_fallible();
    let inner = fallible.into_inner();
    assert_eq!(inner.count(), 3);
}

// ============================================================================
// Additional Rev adapter tests (that don't duplicate existing ones)
// ============================================================================

#[test]
fn rev_double_rev() {
    // Reversing twice should give original order
    let mut lender = VecLender::new(vec![1, 2, 3]).rev().rev();
    assert_eq!(lender.next(), Some(1));
    assert_eq!(lender.next(), Some(2));
    assert_eq!(lender.next(), Some(3));
}

#[test]
fn rev_try_fold_additional() {
    let result: Option<i32> = VecLender::new(vec![1, 2, 3])
        .rev()
        .try_fold(0, |acc, x| Some(acc + x));
    assert_eq!(result, Some(6));
}

#[test]
fn rev_try_rfold_additional() {
    let result: Option<i32> = VecLender::new(vec![1, 2, 3])
        .rev()
        .try_rfold(0, |acc, x| Some(acc + x));
    assert_eq!(result, Some(6));
}

// ============================================================================
// Additional Mutate adapter tests
// ============================================================================

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

// ============================================================================
// Additional Zip adapter tests
// ============================================================================

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
        .try_fold(0, |acc, (a, b)| Some(acc + a + b));
    assert_eq!(result, Some(21)); // (1+4) + (2+5) + (3+6) = 5 + 7 + 9 = 21
}

#[test]
fn zip_try_rfold_additional() {
    let result: Option<i32> = VecLender::new(vec![1, 2, 3])
        .zip(VecLender::new(vec![4, 5, 6]))
        .try_rfold(0, |acc, (a, b)| Some(acc + a + b));
    assert_eq!(result, Some(21));
}

// ============================================================================
// Additional Intersperse adapter tests
// ============================================================================

#[test]
fn intersperse_single_element_additional() {
    let mut lender = VecLender::new(vec![42]).intersperse(0);
    assert_eq!(lender.next(), Some(42));
    assert_eq!(lender.next(), None);
}

#[test]
fn intersperse_empty_additional() {
    let mut lender = VecLender::new(vec![]).intersperse(0);
    assert_eq!(lender.next(), None);
}

// ============================================================================
// Additional source tests
// ============================================================================

#[test]
fn source_repeat_advance_by() {
    let mut repeat = lender::repeat::<lend!(i32)>(42);
    // Advance does nothing on repeat (always Ok)
    assert_eq!(repeat.advance_by(100), Ok(()));
    assert_eq!(repeat.next(), Some(42));
}

#[test]
fn source_empty_fold_additional() {
    let sum = lender::empty::<lend!(i32)>().fold(0, |acc, x| acc + x);
    assert_eq!(sum, 0);
}

#[test]
fn source_empty_rfold_additional() {
    use lender::DoubleEndedLender;

    let sum = lender::empty::<lend!(i32)>().rfold(0, |acc, x| acc + x);
    assert_eq!(sum, 0);
}

#[test]
fn source_once_rfold_additional() {
    use lender::DoubleEndedLender;

    let sum = lender::once::<lend!(i32)>(42).rfold(0, |acc, x| acc + x);
    assert_eq!(sum, 42);
}

// ============================================================================
// Chunk adapter tests (inner lender of Chunky)
// ============================================================================

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
// Advance_by tests for various adapters
// ============================================================================

#[test]
fn advance_by_chain_additional() {
    let mut chained = VecLender::new(vec![1, 2]).chain(VecLender::new(vec![3, 4]));
    assert_eq!(chained.advance_by(3), Ok(())); // Skip 1, 2, 3
    assert_eq!(chained.next(), Some(4));
}

#[test]
fn advance_back_by_chain_additional() {
    use lender::DoubleEndedLender;

    let mut chained = VecLender::new(vec![1, 2]).chain(VecLender::new(vec![3, 4]));
    assert_eq!(chained.advance_back_by(3), Ok(())); // Skip 4, 3, 2
    assert_eq!(chained.next(), Some(1));
}

#[test]
fn advance_by_skip_additional() {
    let mut skipped = VecLender::new(vec![1, 2, 3, 4, 5]).skip(2);
    assert_eq!(skipped.advance_by(2), Ok(())); // Skip 3, 4
    assert_eq!(skipped.next(), Some(5));
}

#[test]
fn advance_by_take_additional() {
    let mut taken = VecLender::new(vec![1, 2, 3, 4, 5]).take(4);
    assert_eq!(taken.advance_by(3), Ok(())); // Skip 1, 2, 3
    assert_eq!(taken.next(), Some(4));
}

#[test]
fn advance_back_by_take_additional() {
    use lender::DoubleEndedLender;

    let mut taken = VecLender::new(vec![1, 2, 3, 4, 5]).take(4);
    assert_eq!(taken.advance_back_by(2), Ok(())); // Skip 4, 3
    assert_eq!(taken.next_back(), Some(2));
}

// ============================================================================
// Additional peekable tests
// ============================================================================

#[test]
fn peekable_peek_multiple() {
    let mut peekable = VecLender::new(vec![1, 2, 3]).peekable();
    assert_eq!(peekable.peek(), Some(&1));
    assert_eq!(peekable.peek(), Some(&1)); // Peeking again returns same value
    assert_eq!(peekable.next(), Some(1));
    assert_eq!(peekable.peek(), Some(&2));
}

#[test]
fn peekable_peek_mut_additional() {
    let mut peekable = VecLender::new(vec![1, 2, 3]).peekable();
    if let Some(v) = peekable.peek_mut() {
        *v *= 10;
    }
    assert_eq!(peekable.next(), Some(10));
    assert_eq!(peekable.next(), Some(2)); // Original unchanged
}

#[test]
fn peekable_next_if_additional() {
    let mut peekable = VecLender::new(vec![1, 2, 3]).peekable();
    assert_eq!(peekable.next_if(|&x| x < 2), Some(1));
    assert_eq!(peekable.next_if(|&x| x < 2), None); // 2 is not < 2
    assert_eq!(peekable.next(), Some(2));
}

#[test]
fn peekable_next_if_eq_additional() {
    let mut peekable = VecLender::new(vec![1, 2, 3]).peekable();
    assert_eq!(peekable.next_if_eq(&1), Some(1));
    assert_eq!(peekable.next_if_eq(&1), None); // Next is 2, not 1
    assert_eq!(peekable.next_if_eq(&2), Some(2));
}

// ============================================================================
// Additional Cycle adapter tests
// ============================================================================

#[test]
fn cycle_multiple_rounds() {
    let mut cycle = VecLender::new(vec![1, 2]).cycle();
    assert_eq!(cycle.next(), Some(1));
    assert_eq!(cycle.next(), Some(2));
    assert_eq!(cycle.next(), Some(1)); // Start of second cycle
    assert_eq!(cycle.next(), Some(2));
    assert_eq!(cycle.next(), Some(1)); // Third cycle
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
        .try_fold(0, |acc, x| Some(acc + x));
    // 1 + 2 + 1 + 2 + 1 = 7
    assert_eq!(result, Some(7));
}

// ============================================================================
// Additional Fuse adapter tests
// ============================================================================

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
// Additional Filter adapter tests
// ============================================================================

#[test]
fn filter_fold_additional() {
    let sum = VecLender::new(vec![1, 2, 3, 4, 5, 6])
        .filter(|&x| x % 2 == 0)
        .fold(0, |acc, x| acc + x);
    assert_eq!(sum, 12); // 2 + 4 + 6
}

#[test]
fn filter_try_fold_additional() {
    let result: Option<i32> = VecLender::new(vec![1, 2, 3, 4, 5])
        .filter(|&x| x % 2 == 1)
        .try_fold(0, |acc, x| Some(acc + x));
    assert_eq!(result, Some(9)); // 1 + 3 + 5
}

#[test]
fn filter_rfold_additional() {
    use lender::DoubleEndedLender;

    let values: Vec<i32> = VecLender::new(vec![1, 2, 3, 4, 5, 6])
        .filter(|&x| x % 2 == 0)
        .rfold(Vec::new(), |mut acc, x| {
            acc.push(x);
            acc
        });
    assert_eq!(values, vec![6, 4, 2]);
}

// ============================================================================
// Additional Map adapter tests
// ============================================================================

#[test]
fn map_fold_additional() {
    let sum = VecLender::new(vec![1, 2, 3])
        .map(|x| x * 2)
        .fold(0, |acc, x| acc + x);
    assert_eq!(sum, 12); // 2 + 4 + 6
}

#[test]
fn map_try_fold_additional() {
    let result: Option<i32> = VecLender::new(vec![1, 2, 3])
        .map(|x| x * 2)
        .try_fold(0, |acc, x| Some(acc + x));
    assert_eq!(result, Some(12));
}

#[test]
fn map_rfold_additional() {
    let values: Vec<i32> =
        VecLender::new(vec![1, 2, 3])
            .map(|x| x * 2)
            .rfold(Vec::new(), |mut acc, x| {
                acc.push(x);
                acc
            });
    assert_eq!(values, vec![6, 4, 2]);
}

// ============================================================================
// Additional StepBy adapter tests
// ============================================================================

#[test]
fn step_by_fold_additional() {
    let sum = VecLender::new(vec![1, 2, 3, 4, 5])
        .step_by(2)
        .fold(0, |acc, x| acc + x);
    // Elements: 1, 3, 5
    assert_eq!(sum, 9);
}

#[test]
fn step_by_try_fold_additional() {
    let result: Option<i32> = VecLender::new(vec![1, 2, 3, 4, 5])
        .step_by(2)
        .try_fold(0, |acc, x| Some(acc + x));
    assert_eq!(result, Some(9));
}

// Note: step_by's nth behavior is already tested in existing tests

// ============================================================================
// MapIntoIter adapter tests - converts lender to iterator via mapping
// ============================================================================

#[test]
fn map_into_iter_basic() {
    // MapIntoIter converts a lender to an iterator by applying a function
    let iter = VecLender::new(vec![1, 2, 3]).map_into_iter(|x| x * 2);
    let collected: Vec<i32> = iter.collect();
    assert_eq!(collected, vec![2, 4, 6]);
}

#[test]
fn map_into_iter_size_hint() {
    let iter = VecLender::new(vec![1, 2, 3, 4, 5]).map_into_iter(|x| x);
    assert_eq!(iter.size_hint(), (5, Some(5)));
}

#[test]
fn map_into_iter_double_ended() {
    let mut iter = VecLender::new(vec![1, 2, 3]).map_into_iter(|x| x * 2);
    assert_eq!(iter.next_back(), Some(6));
    assert_eq!(iter.next(), Some(2));
    assert_eq!(iter.next_back(), Some(4));
    assert_eq!(iter.next(), None);
}

#[test]
fn map_into_iter_exact_size() {
    let iter = VecLender::new(vec![1, 2, 3]).map_into_iter(|x| x);
    assert_eq!(iter.len(), 3);
}

#[test]
fn map_into_iter_into_inner() {
    let iter = VecLender::new(vec![1, 2, 3]).map_into_iter(|x| x * 2);
    let lender = iter.into_inner();
    assert_eq!(lender.count(), 3);
}

#[test]
fn map_into_iter_into_parts() {
    let iter = VecLender::new(vec![1, 2, 3]).map_into_iter(|x| x * 2);
    let (lender, _f) = iter.into_parts();
    assert_eq!(lender.count(), 3);
}

// ============================================================================
// Iter adapter tests - converts lender to iterator when Lend is 'static-like
// ============================================================================

#[test]
fn iter_adapter_basic() {
    // Iter works when the lend type can outlive the lender borrow
    // Using from_iter which yields owned values
    let iter = lender::from_iter(vec![1, 2, 3].into_iter()).iter();
    let collected: Vec<i32> = iter.collect();
    assert_eq!(collected, vec![1, 2, 3]);
}

#[test]
fn iter_adapter_size_hint() {
    let iter = lender::from_iter(vec![1, 2, 3, 4, 5].into_iter()).iter();
    assert_eq!(iter.size_hint(), (5, Some(5)));
}

#[test]
fn iter_adapter_double_ended() {
    let mut iter = lender::from_iter(vec![1, 2, 3].into_iter()).iter();
    assert_eq!(iter.next_back(), Some(3));
    assert_eq!(iter.next(), Some(1));
    assert_eq!(iter.next_back(), Some(2));
    assert_eq!(iter.next(), None);
}

#[test]
fn iter_adapter_exact_size() {
    let iter = lender::from_iter(vec![1, 2, 3].into_iter()).iter();
    assert_eq!(iter.len(), 3);
}

#[test]
fn iter_adapter_into_inner() {
    let iter = lender::from_iter(vec![1, 2, 3].into_iter()).iter();
    let lender = iter.into_inner();
    assert_eq!(lender.count(), 3);
}

// Note: Owned adapter has complex HRTB trait bounds that are difficult to satisfy
// in tests with lend_iter. The owned() method works with specific lenders that have
// the right ToOwned implementations. Coverage is partial due to these constraints.

// ============================================================================
// Scan adapter additional tests
// ============================================================================

#[test]
fn scan_into_inner() {
    let scan = VecLender::new(vec![1, 2, 3]).scan(
        0i32,
        hrc_mut!(for<'all> |args: (&'all mut i32, i32)| -> Option<i32> { Some(*args.0 + args.1) }),
    );
    let lender = scan.into_inner();
    assert_eq!(lender.count(), 3);
}

#[test]
fn scan_into_parts() {
    let scan = VecLender::new(vec![1, 2, 3]).scan(
        10i32,
        hrc_mut!(for<'all> |args: (&'all mut i32, i32)| -> Option<i32> { Some(*args.0 + args.1) }),
    );
    let (lender, state, _f) = scan.into_parts();
    assert_eq!(lender.count(), 3);
    assert_eq!(state, 10); // Initial state
}

#[test]
fn scan_size_hint_additional() {
    let scan = VecLender::new(vec![1, 2, 3, 4, 5]).scan(
        0i32,
        hrc_mut!(for<'all> |args: (&'all mut i32, i32)| -> Option<i32> { Some(args.1) }),
    );
    // Scan can terminate early, so lower bound is 0
    let (lower, upper) = scan.size_hint();
    assert_eq!(lower, 0);
    assert_eq!(upper, Some(5));
}

// Note: scan_early_termination is already tested above

// ============================================================================
// MapWhile adapter additional tests
// ============================================================================

#[test]
fn map_while_into_inner() {
    let map_while = VecLender::new(vec![1, 2, 3])
        .map_while(hrc_mut!(for<'all> |x: i32| -> Option<i32> { Some(x * 2) }));
    let lender = map_while.into_inner();
    assert_eq!(lender.count(), 3);
}

#[test]
fn map_while_into_parts() {
    let map_while = VecLender::new(vec![1, 2, 3])
        .map_while(hrc_mut!(for<'all> |x: i32| -> Option<i32> { Some(x * 2) }));
    let (lender, _predicate) = map_while.into_parts();
    assert_eq!(lender.count(), 3);
}

#[test]
fn map_while_size_hint_additional() {
    let map_while = VecLender::new(vec![1, 2, 3, 4, 5])
        .map_while(hrc_mut!(for<'all> |x: i32| -> Option<i32> { Some(x) }));
    // MapWhile can terminate early, so lower bound is 0
    let (lower, upper) = map_while.size_hint();
    assert_eq!(lower, 0);
    assert_eq!(upper, Some(5));
}

#[test]
fn map_while_all_some() {
    // When all return Some, all elements are yielded
    let values: Vec<i32> = VecLender::new(vec![1, 2, 3])
        .map_while(hrc_mut!(for<'all> |x: i32| -> Option<i32> { Some(x * 10) }))
        .fold(Vec::new(), |mut acc, x| {
            acc.push(x);
            acc
        });
    assert_eq!(values, vec![10, 20, 30]);
}

// ============================================================================
// FilterMap adapter additional tests
// ============================================================================

#[test]
fn filter_map_into_inner() {
    let filter_map =
        VecLender::new(vec![1, 2, 3]).filter_map(hrc_mut!(for<'all> |x: i32| -> Option<i32> {
            if x % 2 == 0 { Some(x * 2) } else { None }
        }));
    let lender = filter_map.into_inner();
    assert_eq!(lender.count(), 3);
}

#[test]
fn filter_map_into_parts() {
    let filter_map = VecLender::new(vec![1, 2, 3])
        .filter_map(hrc_mut!(for<'all> |x: i32| -> Option<i32> { Some(x) }));
    let (lender, _f) = filter_map.into_parts();
    assert_eq!(lender.count(), 3);
}

// ============================================================================
// SkipWhile adapter additional tests
// ============================================================================

#[test]
fn skip_while_into_inner() {
    let skip_while = VecLender::new(vec![1, 2, 3, 4, 5]).skip_while(|&x| x < 3);
    let lender = skip_while.into_inner();
    assert_eq!(lender.count(), 5);
}

#[test]
fn skip_while_into_parts() {
    let skip_while = VecLender::new(vec![1, 2, 3, 4, 5]).skip_while(|&x| x < 3);
    let (lender, _predicate) = skip_while.into_parts();
    assert_eq!(lender.count(), 5);
}

#[test]
fn skip_while_fold_additional() {
    let sum = VecLender::new(vec![1, 2, 3, 4, 5])
        .skip_while(|&x| x < 3)
        .fold(0, |acc, x| acc + x);
    // Skips 1, 2; sums 3 + 4 + 5 = 12
    assert_eq!(sum, 12);
}

#[test]
fn skip_while_try_fold_additional() {
    let result: Option<i32> = VecLender::new(vec![1, 2, 3, 4, 5])
        .skip_while(|&x| x < 3)
        .try_fold(0, |acc, x| Some(acc + x));
    assert_eq!(result, Some(12));
}

// ============================================================================
// TakeWhile adapter additional tests
// ============================================================================

#[test]
fn take_while_into_inner() {
    let take_while = VecLender::new(vec![1, 2, 3, 4, 5]).take_while(|&x| x < 3);
    let lender = take_while.into_inner();
    assert_eq!(lender.count(), 5);
}

#[test]
fn take_while_into_parts() {
    let take_while = VecLender::new(vec![1, 2, 3, 4, 5]).take_while(|&x| x < 3);
    let (lender, _predicate) = take_while.into_parts();
    assert_eq!(lender.count(), 5);
}

#[test]
fn take_while_try_fold_additional() {
    let result: Option<i32> = VecLender::new(vec![1, 2, 3, 4, 5])
        .take_while(|&x| x < 4)
        .try_fold(0, |acc, x| Some(acc + x));
    // Takes 1, 2, 3 (until 4 fails condition)
    assert_eq!(result, Some(6));
}

// ============================================================================
// Skip adapter additional tests
// ============================================================================

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
        .fold(0, |acc, x| acc + x);
    // Skips 1, 2; sums 3 + 4 + 5 = 12
    assert_eq!(sum, 12);
}

#[test]
fn skip_try_fold_additional() {
    let result: Option<i32> = VecLender::new(vec![1, 2, 3, 4, 5])
        .skip(2)
        .try_fold(0, |acc, x| Some(acc + x));
    assert_eq!(result, Some(12));
}

#[test]
fn skip_nth_additional() {
    let mut skipped = VecLender::new(vec![1, 2, 3, 4, 5]).skip(2);
    // After skip(2), we have [3, 4, 5]
    assert_eq!(skipped.nth(1), Some(4)); // [3, 4, 5] -> nth(1) = 4
}

#[test]
fn skip_rfold_additional() {
    use lender::DoubleEndedLender;

    let values: Vec<i32> =
        VecLender::new(vec![1, 2, 3, 4, 5])
            .skip(2)
            .rfold(Vec::new(), |mut acc, x| {
                acc.push(x);
                acc
            });
    // Skips 1, 2; rfolds 5, 4, 3
    assert_eq!(values, vec![5, 4, 3]);
}

// ============================================================================
// Take adapter additional tests
// ============================================================================

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
        .try_fold(0, |acc, x| Some(acc + x));
    // Takes 1, 2, 3
    assert_eq!(result, Some(6));
}

#[test]
fn take_try_rfold_additional() {
    let result: Option<i32> = VecLender::new(vec![1, 2, 3, 4, 5])
        .take(3)
        .try_rfold(0, |acc, x| Some(acc + x));
    assert_eq!(result, Some(6));
}

// ============================================================================
// Inspect adapter additional tests
// ============================================================================

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
        .inspect(|&x| inspected.push(x))
        .try_fold(0, |acc, x| Some(acc + x));
    assert_eq!(result, Some(6));
    assert_eq!(inspected, vec![1, 2, 3]);
}

#[test]
fn inspect_try_rfold_additional() {
    use lender::DoubleEndedLender;

    let mut inspected = Vec::new();
    let result: Option<i32> = VecLender::new(vec![1, 2, 3])
        .inspect(|&x| inspected.push(x))
        .try_rfold(0, |acc, x| Some(acc + x));
    assert_eq!(result, Some(6));
    assert_eq!(inspected, vec![3, 2, 1]); // Reverse order
}

// Note: chain_into_inner is already tested above. Additional chain tests too.

// ============================================================================
// Enumerate adapter additional tests
// ============================================================================

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

// ============================================================================
// Map adapter additional tests
// ============================================================================

#[test]
fn map_into_inner() {
    let map = VecLender::new(vec![1, 2, 3]).map(|x| x * 2);
    let lender = map.into_inner();
    assert_eq!(lender.count(), 3);
}

#[test]
fn map_into_parts_additional() {
    let map = VecLender::new(vec![1, 2, 3]).map(|x| x * 2);
    let (lender, _f) = map.into_parts();
    assert_eq!(lender.count(), 3);
}

#[test]
fn map_try_rfold_additional() {
    let result: Option<i32> = VecLender::new(vec![1, 2, 3])
        .map(|x| x * 2)
        .try_rfold(0, |acc, x| Some(acc + x));
    assert_eq!(result, Some(12)); // 6 + 4 + 2 = 12
}

// ============================================================================
// Filter adapter additional tests
// ============================================================================

#[test]
fn filter_into_inner() {
    let filter = VecLender::new(vec![1, 2, 3, 4, 5]).filter(|&x| x % 2 == 0);
    let lender = filter.into_inner();
    assert_eq!(lender.count(), 5);
}

#[test]
fn filter_into_parts_additional() {
    let filter = VecLender::new(vec![1, 2, 3, 4, 5]).filter(|&x| x % 2 == 0);
    let (lender, _predicate) = filter.into_parts();
    assert_eq!(lender.count(), 5);
}

#[test]
fn filter_try_rfold_additional() {
    let result: Option<i32> = VecLender::new(vec![1, 2, 3, 4, 5, 6])
        .filter(|&x| x % 2 == 0)
        .try_rfold(0, |acc, x| Some(acc + x));
    // Even numbers in reverse: 6, 4, 2
    assert_eq!(result, Some(12));
}

// Note: fuse_into_inner and peekable_into_inner are already tested above

#[test]
fn peekable_fold_additional() {
    let sum = VecLender::new(vec![1, 2, 3])
        .peekable()
        .fold(0, |acc, x| acc + x);
    assert_eq!(sum, 6);
}

#[test]
fn peekable_try_fold_additional() {
    let result: Option<i32> = VecLender::new(vec![1, 2, 3])
        .peekable()
        .try_fold(0, |acc, x| Some(acc + x));
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

// ============================================================================
// Intersperse adapter additional tests
// ============================================================================

#[test]
fn intersperse_into_inner() {
    let intersperse = VecLender::new(vec![1, 2, 3]).intersperse(0);
    let lender = intersperse.into_inner();
    assert_eq!(lender.count(), 3);
}

#[test]
fn intersperse_try_fold_additional() {
    let result: Option<i32> = VecLender::new(vec![1, 2, 3])
        .intersperse(10)
        .try_fold(0, |acc, x| Some(acc + x));
    // 1 + 10 + 2 + 10 + 3 = 26
    assert_eq!(result, Some(26));
}

// ============================================================================
// Mutate adapter additional tests
// ============================================================================

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
// Zip adapter additional tests
// ============================================================================

#[test]
fn zip_into_inner() {
    let zip = VecLender::new(vec![1, 2, 3]).zip(VecLender::new(vec![4, 5, 6]));
    let (a, b) = zip.into_inner();
    assert_eq!(a.count(), 3);
    assert_eq!(b.count(), 3);
}

// ============================================================================
// FromIter source additional tests
// ============================================================================

#[test]
fn from_iter_fold_additional() {
    let sum = lender::from_iter(vec![1, 2, 3, 4, 5].into_iter()).fold(0, |acc, x| acc + x);
    assert_eq!(sum, 15);
}

#[test]
fn from_iter_rfold_additional() {
    use lender::DoubleEndedLender;

    let values: Vec<i32> =
        lender::from_iter(vec![1, 2, 3].into_iter()).rfold(Vec::new(), |mut acc, x| {
            acc.push(x);
            acc
        });
    assert_eq!(values, vec![3, 2, 1]);
}

#[test]
fn from_iter_try_fold_additional() {
    let result: Option<i32> =
        lender::from_iter(vec![1, 2, 3].into_iter()).try_fold(0, |acc, x| Some(acc + x));
    assert_eq!(result, Some(6));
}

#[test]
fn from_iter_try_rfold_additional() {
    use lender::DoubleEndedLender;

    let result: Option<i32> =
        lender::from_iter(vec![1, 2, 3].into_iter()).try_rfold(0, |acc, x| Some(acc + x));
    assert_eq!(result, Some(6));
}

#[test]
fn from_iter_nth_additional() {
    let mut lender = lender::from_iter(vec![1, 2, 3, 4, 5].into_iter());
    assert_eq!(lender.nth(2), Some(3));
}

#[test]
fn from_iter_nth_back_additional() {
    use lender::DoubleEndedLender;

    let mut lender = lender::from_iter(vec![1, 2, 3, 4, 5].into_iter());
    assert_eq!(lender.nth_back(2), Some(3));
}

// ============================================================================
// LendIter source additional tests
// ============================================================================

#[test]
fn lend_iter_fold_additional() {
    let data = vec![1, 2, 3, 4, 5];
    let sum = lender::lend_iter::<lend!(&'lend i32), _>(data.iter()).fold(0, |acc, &x| acc + x);
    assert_eq!(sum, 15);
}

#[test]
fn lend_iter_rfold_additional() {
    use lender::DoubleEndedLender;

    let data = vec![1, 2, 3];
    let values: Vec<i32> =
        lender::lend_iter::<lend!(&'lend i32), _>(data.iter()).rfold(Vec::new(), |mut acc, &x| {
            acc.push(x);
            acc
        });
    assert_eq!(values, vec![3, 2, 1]);
}

#[test]
fn lend_iter_try_fold_additional() {
    let data = vec![1, 2, 3];
    let result: Option<i32> =
        lender::lend_iter::<lend!(&'lend i32), _>(data.iter()).try_fold(0, |acc, &x| Some(acc + x));
    assert_eq!(result, Some(6));
}

#[test]
fn lend_iter_nth_additional() {
    let data = vec![1, 2, 3, 4, 5];
    let mut lender = lender::lend_iter::<lend!(&'lend i32), _>(data.iter());
    assert_eq!(lender.nth(2), Some(&3));
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
    assert_eq!(chunk.next(), Ok(Some(1)));
    assert_eq!(chunk.next(), Ok(Some(2)));
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
    assert_eq!(fallible.last(), Ok(Some(3)));
}

#[test]
fn fallible_lender_advance_by() {
    use lender::FallibleLender;

    let mut fallible: lender::IntoFallible<(), _> =
        VecLender::new(vec![1, 2, 3, 4, 5]).into_fallible();
    assert_eq!(fallible.advance_by(2), Ok(Ok(())));
    assert_eq!(fallible.next(), Ok(Some(3)));
}

#[test]
fn fallible_lender_nth() {
    use lender::FallibleLender;

    let mut fallible: lender::IntoFallible<(), _> =
        VecLender::new(vec![1, 2, 3, 4, 5]).into_fallible();
    assert_eq!(fallible.nth(2), Ok(Some(3)));
}

#[test]
fn fallible_lender_step_by() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2, 3, 4, 5]).into_fallible();
    let mut stepped = fallible.step_by(2);
    assert_eq!(stepped.next(), Ok(Some(1)));
    assert_eq!(stepped.next(), Ok(Some(3)));
    assert_eq!(stepped.next(), Ok(Some(5)));
    assert_eq!(stepped.next(), Ok(None));
}

#[test]
fn fallible_lender_chain() {
    use lender::FallibleLender;

    let fallible1: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2]).into_fallible();
    let fallible2: lender::IntoFallible<(), _> = VecLender::new(vec![3, 4]).into_fallible();
    let mut chained = fallible1.chain(fallible2);
    assert_eq!(chained.next(), Ok(Some(1)));
    assert_eq!(chained.next(), Ok(Some(2)));
    assert_eq!(chained.next(), Ok(Some(3)));
    assert_eq!(chained.next(), Ok(Some(4)));
    assert_eq!(chained.next(), Ok(None));
}

#[test]
fn fallible_lender_zip() {
    use lender::FallibleLender;

    let fallible1: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2, 3]).into_fallible();
    let fallible2: lender::IntoFallible<(), _> = VecLender::new(vec![4, 5, 6]).into_fallible();
    let mut zipped = fallible1.zip(fallible2);
    assert_eq!(zipped.next(), Ok(Some((1, 4))));
    assert_eq!(zipped.next(), Ok(Some((2, 5))));
    assert_eq!(zipped.next(), Ok(Some((3, 6))));
    assert_eq!(zipped.next(), Ok(None));
}

#[test]
fn fallible_lender_map() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2, 3]).into_fallible();
    let mut mapped = fallible.map(|x| Ok(x * 2));
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
    let mut filtered = fallible.filter(|&x| Ok(x % 2 == 0));
    assert_eq!(filtered.next(), Ok(Some(2)));
    assert_eq!(filtered.next(), Ok(Some(4)));
    assert_eq!(filtered.next(), Ok(Some(6)));
    assert_eq!(filtered.next(), Ok(None));
}

#[test]
fn fallible_lender_enumerate() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<(), _> = VecLender::new(vec![10, 20, 30]).into_fallible();
    let mut enumerated = fallible.enumerate();
    assert_eq!(enumerated.next(), Ok(Some((0, 10))));
    assert_eq!(enumerated.next(), Ok(Some((1, 20))));
    assert_eq!(enumerated.next(), Ok(Some((2, 30))));
    assert_eq!(enumerated.next(), Ok(None));
}

#[test]
fn fallible_lender_skip() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2, 3, 4, 5]).into_fallible();
    let mut skipped = fallible.skip(2);
    assert_eq!(skipped.next(), Ok(Some(3)));
    assert_eq!(skipped.next(), Ok(Some(4)));
    assert_eq!(skipped.next(), Ok(Some(5)));
    assert_eq!(skipped.next(), Ok(None));
}

#[test]
fn fallible_lender_take() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2, 3, 4, 5]).into_fallible();
    let mut taken = fallible.take(3);
    assert_eq!(taken.next(), Ok(Some(1)));
    assert_eq!(taken.next(), Ok(Some(2)));
    assert_eq!(taken.next(), Ok(Some(3)));
    assert_eq!(taken.next(), Ok(None));
}

#[test]
fn fallible_lender_skip_while() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2, 3, 4, 5]).into_fallible();
    let mut skipped = fallible.skip_while(|&x| Ok(x < 3));
    assert_eq!(skipped.next(), Ok(Some(3)));
    assert_eq!(skipped.next(), Ok(Some(4)));
    assert_eq!(skipped.next(), Ok(Some(5)));
    assert_eq!(skipped.next(), Ok(None));
}

#[test]
fn fallible_lender_take_while() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2, 3, 4, 5]).into_fallible();
    let mut taken = fallible.take_while(|&x| Ok(x < 4));
    assert_eq!(taken.next(), Ok(Some(1)));
    assert_eq!(taken.next(), Ok(Some(2)));
    assert_eq!(taken.next(), Ok(Some(3)));
    assert_eq!(taken.next(), Ok(None));
}

#[test]
fn fallible_lender_inspect() {
    use lender::FallibleLender;

    let mut inspected = Vec::new();
    let fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2, 3]).into_fallible();
    let mut lender = fallible.inspect(|&x| {
        inspected.push(x);
        Ok(())
    });
    assert_eq!(lender.next(), Ok(Some(1)));
    assert_eq!(lender.next(), Ok(Some(2)));
    assert_eq!(lender.next(), Ok(Some(3)));
    assert_eq!(inspected, vec![1, 2, 3]);
}

#[test]
fn fallible_lender_fuse() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2]).into_fallible();
    let mut fused = fallible.fuse();
    assert_eq!(fused.next(), Ok(Some(1)));
    assert_eq!(fused.next(), Ok(Some(2)));
    assert_eq!(fused.next(), Ok(None));
    assert_eq!(fused.next(), Ok(None)); // Fused stays None
}

#[test]
fn fallible_lender_fold() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2, 3, 4, 5]).into_fallible();
    let sum = fallible.fold(0, |acc, x| Ok(acc + x));
    assert_eq!(sum, Ok(15));
}

#[test]
fn fallible_lender_for_each() {
    use lender::FallibleLender;

    let mut collected = Vec::new();
    let fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2, 3]).into_fallible();
    let result = fallible.for_each(|x| {
        collected.push(x);
        Ok(())
    });
    assert_eq!(result, Ok(()));
    assert_eq!(collected, vec![1, 2, 3]);
}

#[test]
fn fallible_lender_all() {
    use lender::FallibleLender;

    let mut fallible: lender::IntoFallible<(), _> = VecLender::new(vec![2, 4, 6]).into_fallible();
    assert_eq!(fallible.all(|x| Ok(x % 2 == 0)), Ok(true));

    let mut fallible2: lender::IntoFallible<(), _> = VecLender::new(vec![2, 3, 6]).into_fallible();
    assert_eq!(fallible2.all(|x| Ok(x % 2 == 0)), Ok(false));
}

#[test]
fn fallible_lender_any() {
    use lender::FallibleLender;

    let mut fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 3, 5]).into_fallible();
    assert_eq!(fallible.any(|x| Ok(x % 2 == 0)), Ok(false));

    let mut fallible2: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2, 3]).into_fallible();
    assert_eq!(fallible2.any(|x| Ok(x % 2 == 0)), Ok(true));
}

#[test]
fn fallible_lender_find() {
    use lender::FallibleLender;

    let mut fallible: lender::IntoFallible<(), _> =
        VecLender::new(vec![1, 2, 3, 4, 5]).into_fallible();
    assert_eq!(fallible.find(|&x| Ok(x > 3)), Ok(Some(4)));
}

#[test]
fn fallible_lender_position() {
    use lender::FallibleLender;

    let mut fallible: lender::IntoFallible<(), _> =
        VecLender::new(vec![1, 2, 3, 4, 5]).into_fallible();
    assert_eq!(fallible.position(|x| Ok(x == 3)), Ok(Some(2)));
}

#[test]
fn fallible_lender_chunky() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<(), _> =
        VecLender::new(vec![1, 2, 3, 4, 5, 6]).into_fallible();
    let mut chunky = fallible.chunky(2);

    let mut chunk1 = chunky.next().unwrap().unwrap();
    assert_eq!(chunk1.next(), Ok(Some(1)));
    assert_eq!(chunk1.next(), Ok(Some(2)));
    assert_eq!(chunk1.next(), Ok(None));
}

#[test]
fn fallible_lender_rev() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2, 3]).into_fallible();
    let mut rev = fallible.rev();
    assert_eq!(rev.next(), Ok(Some(3)));
    assert_eq!(rev.next(), Ok(Some(2)));
    assert_eq!(rev.next(), Ok(Some(1)));
    assert_eq!(rev.next(), Ok(None));
}

// ============================================================================
// DoubleEndedLender comprehensive tests
// ============================================================================

#[test]
fn double_ended_advance_back_by() {
    use lender::DoubleEndedLender;

    let mut lender = VecLender::new(vec![1, 2, 3, 4, 5]);
    assert_eq!(lender.advance_back_by(2), Ok(()));
    assert_eq!(lender.next_back(), Some(3));
}

#[test]
fn double_ended_nth_back() {
    use lender::DoubleEndedLender;

    let mut lender = VecLender::new(vec![1, 2, 3, 4, 5]);
    assert_eq!(lender.nth_back(2), Some(3)); // 5, 4, [3]
}

#[test]
fn double_ended_try_rfold() {
    use lender::DoubleEndedLender;

    let result: Option<i32> = VecLender::new(vec![1, 2, 3]).try_rfold(0, |acc, x| Some(acc + x));
    assert_eq!(result, Some(6));
}

#[test]
fn double_ended_rfold() {
    use lender::DoubleEndedLender;

    let values: Vec<i32> = VecLender::new(vec![1, 2, 3]).rfold(Vec::new(), |mut acc, x| {
        acc.push(x);
        acc
    });
    assert_eq!(values, vec![3, 2, 1]);
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
    assert_eq!(fallible.next_back(), Ok(Some(3)));
}

#[test]
fn double_ended_fallible_nth_back() {
    use lender::DoubleEndedFallibleLender;

    let mut fallible: lender::IntoFallible<(), _> =
        VecLender::new(vec![1, 2, 3, 4, 5]).into_fallible();
    assert_eq!(fallible.nth_back(2), Ok(Some(3)));
}

#[test]
fn double_ended_fallible_try_rfold() {
    use lender::DoubleEndedFallibleLender;

    let mut fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2, 3]).into_fallible();
    let result: Result<Option<i32>, ()> = fallible.try_rfold(0, |acc, x| Ok(Some(acc + x)));
    assert_eq!(result, Ok(Some(6)));
}

#[test]
fn double_ended_fallible_rfold() {
    use lender::DoubleEndedFallibleLender;

    let fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2, 3]).into_fallible();
    let values: Result<Vec<i32>, ()> = fallible.rfold(Vec::new(), |mut acc, x| {
        acc.push(x);
        Ok(acc)
    });
    assert_eq!(values, Ok(vec![3, 2, 1]));
}

// ============================================================================
// ExactSizeLender tests
// ============================================================================

#[test]
fn exact_size_len() {
    use lender::ExactSizeLender;

    let lender = VecLender::new(vec![1, 2, 3, 4, 5]);
    assert_eq!(lender.len(), 5);
}

#[test]
fn exact_size_is_empty() {
    use lender::ExactSizeLender;

    let lender = VecLender::new(vec![1, 2, 3]);
    assert!(!lender.is_empty());

    let empty_lender = VecLender::new(Vec::<i32>::new());
    assert!(empty_lender.is_empty());
}

// ============================================================================
// Tests for unsafe code paths
// ============================================================================

// Peekable::nth with peeked value when n == 0 (covers unsafe transmute at line 138-139)
#[test]
fn peekable_nth_zero_with_peeked() {
    let mut peekable = VecLender::new(vec![1, 2, 3]).peekable();
    // Peek to store a value
    assert_eq!(peekable.peek(), Some(&1));
    // nth(0) should return the peeked value through the unsafe transmute path
    assert_eq!(peekable.nth(0), Some(1));
    assert_eq!(peekable.next(), Some(2));
}

// Peekable::last with peeked value (covers unsafe transmute at line 153)
#[test]
fn peekable_last_with_peeked_only() {
    let mut peekable = VecLender::new(vec![1]).peekable();
    // Peek the only value
    assert_eq!(peekable.peek(), Some(&1));
    // last() should return the peeked value through the unsafe transmute path
    // when the underlying lender returns None
    assert_eq!(peekable.last(), Some(1));
}

// Peekable::next_back with peeked value when underlying lender is empty
// (covers unsafe transmute at line 208)
#[test]
fn peekable_next_back_with_peeked_exhausted() {
    use lender::DoubleEndedLender;

    let mut peekable = VecLender::new(vec![1]).peekable();
    // Peek the only value
    assert_eq!(peekable.peek(), Some(&1));
    // next_back should return the peeked value through the unsafe transmute path
    assert_eq!(peekable.next_back(), Some(1));
    assert_eq!(peekable.next(), None);
}

// FalliblePeekable::nth with peeked value when n == 0
#[test]
fn fallible_peekable_nth_zero_with_peeked() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2, 3]).into_fallible();
    let mut peekable = fallible.peekable();
    // Peek to store a value
    assert_eq!(peekable.peek(), Ok(Some(&1)));
    // nth(0) should return the peeked value through the unsafe transmute path
    assert_eq!(peekable.nth(0), Ok(Some(1)));
    assert_eq!(peekable.next(), Ok(Some(2)));
}

// FalliblePeekable::last with peeked value
#[test]
fn fallible_peekable_last_with_peeked_only() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1]).into_fallible();
    let mut peekable = fallible.peekable();
    // Peek the only value
    assert_eq!(peekable.peek(), Ok(Some(&1)));
    // last() should return the peeked value through the unsafe transmute path
    assert_eq!(peekable.last(), Ok(Some(1)));
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
    assert_eq!(peekable.next_back(), Ok(Some(1)));
}

// FalliblePeekable::peek_mut (covers unsafe at line 57, 65)
#[test]
fn fallible_peekable_peek_mut() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2, 3]).into_fallible();
    let mut peekable = fallible.peekable();
    // peek_mut to store a value and get mutable reference
    let peeked = peekable.peek_mut().unwrap();
    assert_eq!(peeked, Some(&mut 1));
}

// FalliblePeekable::next_if (covers unsafe at lines 76, 85)
#[test]
fn fallible_peekable_next_if_match() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2, 3]).into_fallible();
    let mut peekable = fallible.peekable();
    // next_if should return Some when predicate matches
    assert_eq!(peekable.next_if(|&x| x == 1), Ok(Some(1)));
    // Should have advanced
    assert_eq!(peekable.next(), Ok(Some(2)));
}

#[test]
fn fallible_peekable_next_if_no_match() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2, 3]).into_fallible();
    let mut peekable = fallible.peekable();
    // next_if should return None when predicate doesn't match (and store in peeked)
    assert_eq!(peekable.next_if(|&x| x == 5), Ok(None));
    // Value should still be available
    assert_eq!(peekable.next(), Ok(Some(1)));
}

// Iter adapter FallibleIterator next (covers unsafe at line 101-102)
#[test]
fn iter_fallible_iterator_next() {
    use fallible_iterator::FallibleIterator;

    let fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2, 3]).into_fallible();
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

    let fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2, 3]).into_fallible();
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

// Intersperse with separator clone (covers unsafe at line 62)
#[test]
fn intersperse_separator_coverage() {
    let mut intersperse = VecLender::new(vec![1, 2, 3]).intersperse(0);
    // Consume all to exercise the separator clone path
    let mut results = Vec::new();
    while let Some(x) = intersperse.next() {
        results.push(x);
    }
    assert_eq!(results, vec![1, 0, 2, 0, 3]);
}

// IntersperseWith (covers unsafe at line 142)
#[test]
fn intersperse_with_coverage() {
    let mut intersperse = VecLender::new(vec![1, 2, 3]).intersperse_with(|| 0);
    let mut results = Vec::new();
    while let Some(x) = intersperse.next() {
        results.push(x);
    }
    assert_eq!(results, vec![1, 0, 2, 0, 3]);
}

// Cycle fallible next (covers unsafe reborrow at line 129)
#[test]
fn cycle_fallible_next_coverage() {
    use lender::FallibleLender;

    let fallible: lender::IntoFallible<(), _> = VecLender::new(vec![1, 2]).into_fallible();
    let mut cycle = fallible.cycle();
    // Call next() multiple times to exercise the unsafe reborrow and cycling
    assert_eq!(cycle.next(), Ok(Some(1)));
    assert_eq!(cycle.next(), Ok(Some(2)));
    // This should cycle back to the beginning
    assert_eq!(cycle.next(), Ok(Some(1)));
    assert_eq!(cycle.next(), Ok(Some(2)));
    assert_eq!(cycle.next(), Ok(Some(1)));
}

// FilterMap unsafe transmute (covers unsafe at lines 50, 70)
#[test]
fn filter_map_double_ended_coverage() {
    use lender::DoubleEndedLender;

    let mut fm = VecLender::new(vec![1, 2, 3, 4, 5])
        .filter_map(|x| if x % 2 == 0 { Some(x * 2) } else { None });
    // Use next_back to exercise the DoubleEndedLender unsafe path
    assert_eq!(fm.next_back(), Some(8)); // 4 * 2
    assert_eq!(fm.next(), Some(4)); // 2 * 2
    assert_eq!(fm.next_back(), None);
}

// FromIter fallible (covers unsafe at lines 381, 399)
#[test]
fn from_iter_fallible_coverage() {
    use lender::{DoubleEndedFallibleLender, FallibleLender};

    let data = vec![1, 2, 3];
    let fallible: lender::IntoFallible<(), _> = lender::from_iter(data.iter()).into_fallible();
    let mut lender = fallible;
    assert_eq!(lender.next(), Ok(Some(&1)));
    assert_eq!(lender.next_back(), Ok(Some(&3)));
}
