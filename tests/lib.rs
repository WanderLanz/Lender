#![cfg(test)]
use ::lender::prelude::*;

struct WindowsMut<'a, T> {
    slice: &'a mut [T],
    begin: usize,
    len: usize,
}
impl<'lend, 'a, T> Lending<'lend> for WindowsMut<'a, T> {
    type Lend = &'lend mut [T];
}
impl<'a, T> Lender for WindowsMut<'a, T> {
    fn next<'lend>(&'lend mut self) -> Option<&'lend mut [T]> {
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
        fn next<'lend>(&'lend mut self) -> Option<io::Result<&'lend str>> {
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
        fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> { Some(&mut self.0) }
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
