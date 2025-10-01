#![cfg(test)]
use ::lender::prelude::*;

struct WindowsMut<'a, T> {
    slice: &'a mut [T],
    begin: usize,
    len: usize,
}
impl<'lend, T> Lending<'lend> for WindowsMut<'_, T> {
    type Lend = &'lend mut [T];
}
impl<T> Lender for WindowsMut<'_, T> {
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
    WindowsMut { slice: &mut data, begin: 0, len: 3 }
        .for_each(hrc_mut!(for<'lend> |w: &'lend mut [u32]| { w[2] = w[0] + w[1] }));
    assert_eq!(data, [0, 1, 1, 2, 3, 5, 8, 13, 21]);
    WindowsMut { slice: &mut data, begin: 0, len: 3 }
        .filter(|x| x[0] > 0)
        .map(hrc_mut!(for<'lend> |x: &'lend mut [u32]| -> &'lend mut u32 { &mut x[0] }))
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
    let mut lines = LinesStr { buf, line: String::new() };
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
        fn next(&mut self) -> Option<Lend<'_, Self>> {
            Some(&mut self.0)
        }
    }
    let mut x = 0u32;
    let mut bar: MyLender<'_, u32> = MyLender(&mut x);
    let _ = bar.next();
    let _ = bar.next();
    let mut bar = bar.into_lender().mutate(|y| **y += 1).map(|x: &mut u32| *x + 1).iter();
    let _ = bar.find_map(|x| if x > 0 { Some(vec![1, 2, 3]) } else { None });
}

#[test]
fn from_lender() {
    let mut vec = vec![1u32, 2, 3, 4, 5];
    let windows = WindowsMut { slice: &mut vec, begin: 0, len: 3 };
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

    const ERR_MSG: &'static str = "Try Collect Error";

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
            Self { inner: ErrLenderInner::default(), dst }
        }
    }

    impl<'lend> Lending<'lend> for ErrLender<'_> {
        type Lend = Result<(), WriteOnDrop<'lend>>;
    }

    impl Lender for ErrLender<'_> {
        fn next(&mut self) -> Option<Lend<'_, Self>> {
            match self.inner {
                ErrLenderInner::Count(1) => {
                    let err = ERR_MSG.to_owned();
                    self.inner = ErrLenderInner::Err(err);
                    match &self.inner {
                        ErrLenderInner::Err(err) => Some(Err(WriteOnDrop { src: err.as_str(), dst: self.dst })),
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
