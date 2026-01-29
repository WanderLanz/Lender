mod common;
use common::*;
use ::lender::prelude::*;

// ============================================================================
// Misc tests
// ============================================================================

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
// Reduce, partition, sum, product, unzip tests (Lender)
// ============================================================================

#[test]
fn lender_reduce() {
    // Basic reduce: sum via reduce
    assert_eq!(
        VecLender::new(vec![1, 2, 3, 4]).reduce(|acc, x| acc + x),
        Some(10)
    );
    // Single element
    assert_eq!(VecLender::new(vec![42]).reduce(|acc, x| acc + x), Some(42));
    // Empty
    assert_eq!(VecLender::new(vec![]).reduce(|acc, x| acc + x), None);
}

#[test]
fn lender_try_reduce() {
    // Successful try_reduce
    let result: Result<Option<i32>, &str> =
        VecLender::new(vec![1, 2, 3]).try_reduce(|acc, x| Ok::<_, &str>(acc + x));
    assert_eq!(result, Ok(Some(6)));

    // Empty lender
    let result: Result<Option<i32>, &str> =
        VecLender::new(vec![]).try_reduce(|acc, x| Ok::<_, &str>(acc + x));
    assert_eq!(result, Ok(None));

    // Early exit on error
    let result: Result<Option<i32>, &str> = VecLender::new(vec![1, 2, 3]).try_reduce(|acc, x| {
        if acc + x > 4 {
            Err("too large")
        } else {
            Ok(acc + x)
        }
    });
    assert_eq!(result, Err("too large"));
}

#[test]
fn lender_partition() {
    struct I32Vec(Vec<i32>);

    impl Default for I32Vec {
        fn default() -> Self {
            I32Vec(Vec::new())
        }
    }

    impl<L: IntoLender> lender::ExtendLender<L> for I32Vec
    where
        L::Lender: for<'all> Lending<'all, Lend = i32>,
    {
        fn extend_lender(&mut self, lender: L) {
            lender.into_lender().for_each(|x| self.0.push(x));
        }

        fn extend_lender_one(&mut self, item: i32) {
            self.0.push(item);
        }
    }

    let (evens, odds): (I32Vec, I32Vec) =
        VecLender::new(vec![1, 2, 3, 4, 5]).partition::<(), _, _>(|&x| x % 2 == 0);
    assert_eq!(evens.0, vec![2, 4]);
    assert_eq!(odds.0, vec![1, 3, 5]);

    // All match predicate
    let (all, none): (I32Vec, I32Vec) =
        VecLender::new(vec![2, 4, 6]).partition::<(), _, _>(|&x| x % 2 == 0);
    assert_eq!(all.0, vec![2, 4, 6]);
    assert!(none.0.is_empty());

    // Empty lender
    let (a, b): (I32Vec, I32Vec) = VecLender::new(vec![]).partition::<(), _, _>(|&x| x > 0);
    assert!(a.0.is_empty());
    assert!(b.0.is_empty());
}

#[test]
fn lender_sum() {
    struct I32Sum(i32);

    impl lender::SumLender<VecLender> for I32Sum {
        fn sum_lender(lender: VecLender) -> Self {
            I32Sum(lender.fold(0, |acc, x| acc + x))
        }
    }

    let sum: I32Sum = VecLender::new(vec![1, 2, 3, 4]).sum();
    assert_eq!(sum.0, 10);

    let sum_empty: I32Sum = VecLender::new(vec![]).sum();
    assert_eq!(sum_empty.0, 0);
}

#[test]
fn lender_product() {
    struct I32Product(i32);

    impl lender::ProductLender<VecLender> for I32Product {
        fn product_lender(lender: VecLender) -> Self {
            I32Product(lender.fold(1, |acc, x| acc * x))
        }
    }

    let product: I32Product = VecLender::new(vec![1, 2, 3, 4]).product();
    assert_eq!(product.0, 24);

    let product_empty: I32Product = VecLender::new(vec![]).product();
    assert_eq!(product_empty.0, 1);
}

#[test]
fn lender_unzip() {
    // A lender over (i32, i32) tuples
    struct TupleLender {
        data: Vec<(i32, i32)>,
        idx: usize,
    }

    impl<'lend> Lending<'lend> for TupleLender {
        type Lend = (i32, i32);
    }

    impl Lender for TupleLender {
        check_covariance!();

        fn next(&mut self) -> Option<Lend<'_, Self>> {
            if self.idx < self.data.len() {
                let item = self.data[self.idx];
                self.idx += 1;
                Some(item)
            } else {
                None
            }
        }
    }

    // Lender::unzip requires ExtendLender impls for FirstShunt/SecondShunt,
    // so we test via owned() which delegates to Iterator::unzip.
    let (a, b): (Vec<i32>, Vec<i32>) = TupleLender {
        data: vec![(1, 4), (2, 5), (3, 6)],
        idx: 0,
    }
    .owned()
    .unzip();
    assert_eq!(a, vec![1, 2, 3]);
    assert_eq!(b, vec![4, 5, 6]);
}

// ============================================================================
// Comparison method tests (Lender)
// ============================================================================

#[test]
fn lender_cmp() {
    use core::cmp::Ordering;

    assert_eq!(
        VecLender::new(vec![1, 2, 3]).cmp(VecLender::new(vec![1, 2, 3])),
        Ordering::Equal
    );
    assert_eq!(
        VecLender::new(vec![1, 2, 3]).cmp(VecLender::new(vec![1, 2, 4])),
        Ordering::Less
    );
    assert_eq!(
        VecLender::new(vec![1, 2, 4]).cmp(VecLender::new(vec![1, 2, 3])),
        Ordering::Greater
    );
    // Different lengths
    assert_eq!(
        VecLender::new(vec![1, 2]).cmp(VecLender::new(vec![1, 2, 3])),
        Ordering::Less
    );
    assert_eq!(
        VecLender::new(vec![1, 2, 3]).cmp(VecLender::new(vec![1, 2])),
        Ordering::Greater
    );
    // Empty
    assert_eq!(
        VecLender::new(vec![]).cmp(VecLender::new(vec![])),
        Ordering::Equal
    );
}

#[test]
fn lender_cmp_by() {
    use core::cmp::Ordering;

    // Compare by absolute value
    assert_eq!(
        VecLender::new(vec![-1, 2])
            .cmp_by(VecLender::new(vec![1, -2]), |a, b| a.abs().cmp(&b.abs())),
        Ordering::Equal
    );
    assert_eq!(
        VecLender::new(vec![1]).cmp_by(VecLender::new(vec![2]), |a, b| a.cmp(&b)),
        Ordering::Less
    );
}

#[test]
fn lender_partial_cmp_by() {
    use core::cmp::Ordering;

    assert_eq!(
        VecLender::new(vec![1, 2, 3])
            .partial_cmp_by(VecLender::new(vec![1, 2, 3]), |a, b| a.partial_cmp(&b)),
        Some(Ordering::Equal)
    );
    assert_eq!(
        VecLender::new(vec![1, 2])
            .partial_cmp_by(VecLender::new(vec![1, 3]), |a, b| a.partial_cmp(&b)),
        Some(Ordering::Less)
    );
    assert_eq!(
        VecLender::new(vec![1, 3])
            .partial_cmp_by(VecLender::new(vec![1, 2]), |a, b| a.partial_cmp(&b)),
        Some(Ordering::Greater)
    );
    // Different lengths
    assert_eq!(
        VecLender::new(vec![1])
            .partial_cmp_by(VecLender::new(vec![1, 2]), |a, b| a.partial_cmp(&b)),
        Some(Ordering::Less)
    );
}

#[test]
fn lender_eq_by() {
    assert!(VecLender::new(vec![1, 2, 3]).eq_by(VecLender::new(vec![1, 2, 3]), |a, b| a == b));
    assert!(!VecLender::new(vec![1, 2, 3]).eq_by(VecLender::new(vec![1, 2, 4]), |a, b| a == b));
    assert!(!VecLender::new(vec![1, 2]).eq_by(VecLender::new(vec![1, 2, 3]), |a, b| a == b));
    assert!(VecLender::new(vec![]).eq_by(VecLender::new(vec![]), |a: i32, b: i32| a == b));
    // Equal by absolute value
    assert!(
        VecLender::new(vec![-1, 2]).eq_by(VecLender::new(vec![1, -2]), |a, b| a.abs() == b.abs())
    );
}

#[test]
fn lender_ne_via_eq_by() {
    // ne is !eq, so test via eq_by negation
    assert!(!VecLender::new(vec![1, 2, 3]).eq_by(VecLender::new(vec![1, 2, 4]), |a, b| a == b));
    assert!(VecLender::new(vec![1, 2, 3]).eq_by(VecLender::new(vec![1, 2, 3]), |a, b| a == b));
    assert!(!VecLender::new(vec![1]).eq_by(VecLender::new(vec![1, 2]), |a, b| a == b));
}

#[test]
fn lender_ordering_via_partial_cmp_by() {
    use core::cmp::Ordering;

    // lt: a < b
    assert_eq!(
        VecLender::new(vec![1, 2])
            .partial_cmp_by(VecLender::new(vec![1, 3]), |a, b| a.partial_cmp(&b)),
        Some(Ordering::Less)
    );
    // le: a <= b (equal)
    assert_eq!(
        VecLender::new(vec![1, 2])
            .partial_cmp_by(VecLender::new(vec![1, 2]), |a, b| a.partial_cmp(&b)),
        Some(Ordering::Equal)
    );
    // gt: a > b
    assert_eq!(
        VecLender::new(vec![1, 3])
            .partial_cmp_by(VecLender::new(vec![1, 2]), |a, b| a.partial_cmp(&b)),
        Some(Ordering::Greater)
    );
    // ge: a >= b (equal)
    assert_eq!(
        VecLender::new(vec![1, 2])
            .partial_cmp_by(VecLender::new(vec![1, 2]), |a, b| a.partial_cmp(&b)),
        Some(Ordering::Equal)
    );
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
