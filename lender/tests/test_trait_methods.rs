mod common;
use ::lender::prelude::*;
use common::*;

// ============================================================================
// Misc tests
// ============================================================================

#[test]
fn test_lines_str() {
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
fn test_simple_lender() {
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
    let mut x = 0;
    let mut bar: MyLender<'_, i32> = MyLender(&mut x);
    let _ = bar.next();
    let _ = bar.next();
    let mut bar = bar
        .into_lender()
        .mutate(|y| **y += 1)
        .map(covar_mut!(for<'lend> |x: &'lend mut i32| -> i32 { *x + 1 }))
        .iter();
    let _ = bar.find_map(|x| if x > 0 { Some(vec![1, 2, 3]) } else { None });
}

#[test]
fn test_from_lender() {
    let mut vec = vec![1, 2, 3, 4, 5];
    let windows = WindowsMut {
        slice: &mut vec,
        begin: 0,
        len: 3,
    };
    let vec = MyVec::<Vec<i32>>::from_lender(windows);
    assert_eq!(vec.0, vec![&[1, 2, 3][..], &[2, 3, 4][..], &[3, 4, 5][..]]);

    struct MyVec<T>(Vec<T>);
    impl<L: Lender> FromLender<L> for MyVec<Vec<i32>>
    where
        for<'all> L: Lending<'all, Lend = &'all mut [i32]>,
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
fn test_try_collect() {
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
fn test_lender_advance_by() {
    use core::num::NonZeroUsize;

    let mut lender = VecLender::new(vec![1, 2, 3, 4, 5]);

    // advance_by(2) skips 2 elements
    assert_eq!(lender.advance_by(2), Ok(()));
    assert_eq!(lender.next(), Some(&3));

    // advance_by with remaining elements
    assert_eq!(lender.advance_by(1), Ok(()));
    assert_eq!(lender.next(), Some(&5));

    // advance_by past end returns Err with remaining count
    assert_eq!(lender.advance_by(5), Err(NonZeroUsize::new(5).unwrap()));
}

#[test]
fn test_lender_count() {
    let lender = VecLender::new(vec![1, 2, 3, 4, 5]);
    assert_eq!(lender.count(), 5);

    let empty = VecLender::new(vec![]);
    assert_eq!(empty.count(), 0);
}

#[test]
fn test_lender_last() {
    let mut lender = VecLender::new(vec![1, 2, 3]);
    assert_eq!(lender.last(), Some(&3));

    let mut empty = VecLender::new(vec![]);
    assert_eq!(empty.last(), None);
}

#[test]
fn test_lender_for_each() {
    let mut collected = Vec::new();
    VecLender::new(vec![1, 2, 3]).for_each(|x| collected.push(*x));
    assert_eq!(collected, vec![1, 2, 3]);
}

#[test]
fn test_lender_all() {
    assert!(VecLender::new(vec![2, 4, 6]).all(|x| *x % 2 == 0));
    assert!(!VecLender::new(vec![2, 3, 6]).all(|x| *x % 2 == 0));
    assert!(VecLender::new(vec![]).all(|_x: &i32| false)); // vacuously true
}

#[test]
fn test_lender_any() {
    assert!(VecLender::new(vec![1, 2, 3]).any(|x| *x == 2));
    assert!(!VecLender::new(vec![1, 2, 3]).any(|x| *x == 10));
    assert!(!VecLender::new(vec![]).any(|_x: &i32| true)); // vacuously false
}

#[test]
fn test_lender_find() {
    let mut lender = VecLender::new(vec![1, 2, 3, 4, 5]);
    assert_eq!(lender.find(|&x| *x > 2), Some(&3));
    assert_eq!(lender.find(|&x| *x > 10), None);
}

#[test]
fn test_lender_find_map() {
    let mut lender = VecLender::new(vec![1, 2, 3, 4, 5]);
    let result = lender.find_map(covar_mut!(for<'all> |x: &'all i32| -> Option<i32> {
        if *x > 2 { Some(*x * 10) } else { None }
    }));
    assert_eq!(result, Some(30));
}

#[test]
fn test_lender_position() {
    let mut lender = VecLender::new(vec![1, 2, 3, 4, 5]);
    assert_eq!(lender.position(|x| *x == 3), Some(2));

    let mut lender2 = VecLender::new(vec![1, 2, 3]);
    assert_eq!(lender2.position(|x| *x == 10), None);
}

#[test]
fn test_lender_max() {
    // max/min/reduce/etc. require `for<'all> Lend<'all, Self>: ToOwned<Owned = T>`.
    // For a reference-yielding lender (Lend = &'lend i32), the HRTB cannot be
    // satisfied because T would need to depend on 'lend. We test via from_iter
    // wrapping a standard iterator that yields owned i32 values.
    assert_eq!(vec![1, 5, 3, 2, 4].into_iter().into_lender().max(), Some(5));
    assert_eq!(Vec::<i32>::new().into_iter().into_lender().max(), None);
    // Per Iterator::max docs: "If several elements are equally maximum, the last element is returned."
    assert_eq!(vec![1, 3, 3, 1].into_iter().into_lender().max(), Some(3));
}

#[test]
fn test_lender_min() {
    assert_eq!(vec![3, 1, 5, 2, 4].into_iter().into_lender().min(), Some(1));
    assert_eq!(Vec::<i32>::new().into_iter().into_lender().min(), None);
    // Per Iterator::min docs: "If several elements are equally minimum, the first element is returned."
    assert_eq!(vec![3, 1, 1, 3].into_iter().into_lender().min(), Some(1));
}

#[test]
fn test_lender_max_by_key() {
    assert_eq!(
        vec![-3, 0, 1, 5, -2]
            .into_iter()
            .into_lender()
            .max_by_key(|x: &i32| x.abs()),
        Some(5)
    );
}

#[test]
fn test_lender_min_by_key() {
    assert_eq!(
        vec![-3, 0, 1, 5, -2]
            .into_iter()
            .into_lender()
            .min_by_key(|x: &i32| x.abs()),
        Some(0)
    );
}

#[test]
fn test_lender_max_by() {
    assert_eq!(
        vec![1, 5, 3]
            .into_iter()
            .into_lender()
            .max_by(|a, b| a.cmp(b)),
        Some(5)
    );
    // Per Iterator::max_by docs: "If several elements are equally maximum, the last element is returned."
    assert_eq!(
        vec![-3, 1, 3]
            .into_iter()
            .into_lender()
            .max_by(|a: &i32, b: &i32| a.abs().cmp(&b.abs())),
        Some(3)
    );
}

#[test]
fn test_lender_min_by() {
    assert_eq!(
        vec![3, 1, 5]
            .into_iter()
            .into_lender()
            .min_by(|a, b| a.cmp(b)),
        Some(1)
    );
    // Per Iterator::min_by docs: "If several elements are equally minimum, the first element is returned."
    assert_eq!(
        vec![3, -1, 1]
            .into_iter()
            .into_lender()
            .min_by(|a: &i32, b: &i32| a.abs().cmp(&b.abs())),
        Some(-1)
    );
}

#[test]
fn test_lender_is_sorted() {
    assert!(vec![1, 2, 3, 4].into_iter().into_lender().is_sorted());
    assert!(vec![1, 1, 2, 2].into_iter().into_lender().is_sorted());
    assert!(!vec![1, 3, 2].into_iter().into_lender().is_sorted());
    assert!(Vec::<i32>::new().into_iter().into_lender().is_sorted());
    assert!(vec![1].into_iter().into_lender().is_sorted());
}

#[test]
fn test_lender_is_sorted_by() {
    // Sorted in reverse order
    assert!(
        vec![4, 3, 2, 1]
            .into_iter()
            .into_lender()
            .is_sorted_by(|a, b| Some(b.cmp(a)))
    );
}

#[test]
fn test_lender_is_sorted_by_key() {
    // Sorted by absolute value
    assert!(
        vec![0, -1, 2, -3]
            .into_iter()
            .into_lender()
            .is_sorted_by_key(|x: i32| x.abs())
    );
}

// ============================================================================
// Reduce, partition, sum, product, unzip tests (Lender)
// ============================================================================

#[test]
fn test_lender_reduce() {
    // reduce requires `for<'all> Lend<'all, Self>: ToOwned<Owned = T>`, use from_iter
    assert_eq!(
        vec![1, 2, 3, 4]
            .into_iter()
            .into_lender()
            .reduce(|acc, x| acc + x),
        Some(10)
    );
    // Single element
    assert_eq!(
        vec![42].into_iter().into_lender().reduce(|acc, x| acc + x),
        Some(42)
    );
    // Empty
    assert_eq!(
        Vec::<i32>::new()
            .into_iter()
            .into_lender()
            .reduce(|acc, x| acc + x),
        None
    );
}

#[test]
fn test_lender_try_reduce() {
    // try_reduce requires ToOwned, so use from_iter with owned values.
    // Return type is ChangeOutputType<R, Option<T>> = Result<Option<i32>, &str>.
    let result: Result<Option<i32>, &str> = vec![1, 2, 3]
        .into_iter()
        .into_lender()
        .try_reduce(|acc, x| Ok(acc + x));
    assert_eq!(result, Ok(Some(6)));

    // Single element (closure is never called)
    let result: Result<Option<i32>, &str> = vec![42]
        .into_iter()
        .into_lender()
        .try_reduce(|acc, x| Ok(acc + x));
    assert_eq!(result, Ok(Some(42)));

    // Empty lender
    let result: Result<Option<i32>, &str> = Vec::<i32>::new()
        .into_iter()
        .into_lender()
        .try_reduce(|acc, x| Ok(acc + x));
    assert_eq!(result, Ok(None));

    // Early exit on error
    let result: Result<Option<i32>, &str> = vec![1, 2, 3, 4, 5]
        .into_iter()
        .into_lender()
        .try_reduce(|acc, x| {
            if acc + x > 6 {
                Err("too large")
            } else {
                Ok(acc + x)
            }
        });
    assert_eq!(result, Err("too large"));
}

#[test]
fn test_lender_partition() {
    #[derive(Default)]
    struct I32Vec(Vec<i32>);

    impl<L: IntoLender> lender::ExtendLender<L> for I32Vec
    where
        L::Lender: for<'all> Lending<'all, Lend = &'all i32>,
    {
        fn extend_lender(&mut self, lender: L) {
            lender.into_lender().for_each(|x| self.0.push(*x));
        }

        fn extend_lender_one(&mut self, item: &i32) {
            self.0.push(*item);
        }
    }

    let (evens, odds): (I32Vec, I32Vec) =
        VecLender::new(vec![1, 2, 3, 4, 5]).partition::<_, _>(|&x| x % 2 == 0);
    assert_eq!(evens.0, vec![2, 4]);
    assert_eq!(odds.0, vec![1, 3, 5]);

    // All match predicate
    let (all, none): (I32Vec, I32Vec) =
        VecLender::new(vec![2, 4, 6]).partition::<_, _>(|&x| x % 2 == 0);
    assert_eq!(all.0, vec![2, 4, 6]);
    assert!(none.0.is_empty());

    // Empty lender
    let (a, b): (I32Vec, I32Vec) = VecLender::new(vec![]).partition::<_, _>(|&x| *x > 0);
    assert!(a.0.is_empty());
    assert!(b.0.is_empty());
}

#[test]
fn test_lender_sum() {
    struct I32Sum(i32);

    impl lender::SumLender<VecLender> for I32Sum {
        fn sum_lender(lender: VecLender) -> Self {
            I32Sum(lender.fold(0, |acc, x| acc + *x))
        }
    }

    let sum: I32Sum = VecLender::new(vec![1, 2, 3, 4]).sum();
    assert_eq!(sum.0, 10);

    let sum_empty: I32Sum = VecLender::new(vec![]).sum();
    assert_eq!(sum_empty.0, 0);
}

#[test]
fn test_lender_product() {
    struct I32Product(i32);

    impl lender::ProductLender<VecLender> for I32Product {
        fn product_lender(lender: VecLender) -> Self {
            I32Product(lender.fold(1, |acc, x| acc * *x))
        }
    }

    let product: I32Product = VecLender::new(vec![1, 2, 3, 4]).product();
    assert_eq!(product.0, 24);

    let product_empty: I32Product = VecLender::new(vec![]).product();
    assert_eq!(product_empty.0, 1);
}

#[test]
fn test_lender_unzip() {
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
fn test_lender_cmp() {
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
fn test_lender_cmp_by() {
    use core::cmp::Ordering;

    // Compare by absolute value
    assert_eq!(
        VecLender::new(vec![-1, 2])
            .cmp_by(VecLender::new(vec![1, -2]), |a, b| a.abs().cmp(&b.abs())),
        Ordering::Equal
    );
    assert_eq!(
        VecLender::new(vec![1]).cmp_by(VecLender::new(vec![2]), |a, b| a.cmp(b)),
        Ordering::Less
    );
}

#[test]
fn test_lender_partial_cmp_by() {
    use core::cmp::Ordering;

    assert_eq!(
        VecLender::new(vec![1, 2, 3])
            .partial_cmp_by(VecLender::new(vec![1, 2, 3]), |a, b| a.partial_cmp(b)),
        Some(Ordering::Equal)
    );
    assert_eq!(
        VecLender::new(vec![1, 2])
            .partial_cmp_by(VecLender::new(vec![1, 3]), |a, b| a.partial_cmp(b)),
        Some(Ordering::Less)
    );
    assert_eq!(
        VecLender::new(vec![1, 3])
            .partial_cmp_by(VecLender::new(vec![1, 2]), |a, b| a.partial_cmp(b)),
        Some(Ordering::Greater)
    );
    // Different lengths
    assert_eq!(
        VecLender::new(vec![1]).partial_cmp_by(VecLender::new(vec![1, 2]), |a, b| a.partial_cmp(b)),
        Some(Ordering::Less)
    );
}

#[test]
fn test_lender_eq_by() {
    assert!(VecLender::new(vec![1, 2, 3]).eq_by(VecLender::new(vec![1, 2, 3]), |a, b| a == b));
    assert!(!VecLender::new(vec![1, 2, 3]).eq_by(VecLender::new(vec![1, 2, 4]), |a, b| a == b));
    assert!(!VecLender::new(vec![1, 2]).eq_by(VecLender::new(vec![1, 2, 3]), |a, b| a == b));
    assert!(VecLender::new(vec![]).eq_by(VecLender::new(vec![]), |a: &i32, b: &i32| a == b));
    // Equal by absolute value
    assert!(
        VecLender::new(vec![-1, 2]).eq_by(VecLender::new(vec![1, -2]), |a, b| a.abs() == b.abs())
    );
}

// ============================================================================
// Try operations tests
// ============================================================================

#[test]
fn test_lender_try_for_each() {
    let mut sum = 0;
    let result: Result<(), &str> = VecLender::new(vec![1, 2, 3]).try_for_each(|x| {
        sum += *x;
        Ok(())
    });
    assert!(result.is_ok());
    assert_eq!(sum, 6);
}

#[test]
fn test_lender_try_for_each_early_exit() {
    let mut sum = 0;
    let result: Result<(), &str> = VecLender::new(vec![1, 2, 3, 4, 5]).try_for_each(|x| {
        if *x > 3 {
            Err("too big")
        } else {
            sum += *x;
            Ok(())
        }
    });
    assert_eq!(result, Err("too big"));
    assert_eq!(sum, 6); // 1 + 2 + 3
}

#[test]
fn test_lender_try_fold() {
    let result: Result<i32, &str> =
        VecLender::new(vec![1, 2, 3]).try_fold(0, |acc, x| Ok(acc + *x));
    assert_eq!(result, Ok(6));
}

#[test]
fn test_lender_try_fold_early_exit() {
    let result: Result<i32, &str> = VecLender::new(vec![1, 2, 3, 4, 5]).try_fold(0, |acc, x| {
        if *x > 3 { Err("too big") } else { Ok(acc + *x) }
    });
    assert_eq!(result, Err("too big"));
}

// ============================================================================
// try_find tests
// ============================================================================

#[test]
fn test_lender_try_find_found() {
    let mut lender = VecLender::new(vec![1, 2, 3, 4, 5]);
    let result: Result<Option<&i32>, String> = lender.try_find(|x| Ok(**x == 3));
    assert_eq!(result, Ok(Some(&3)));
}

#[test]
fn test_lender_try_find_not_found() {
    let mut lender = VecLender::new(vec![1, 2, 3]);
    let result: Result<Option<&i32>, String> = lender.try_find(|x| Ok(**x == 99));
    assert_eq!(result, Ok(None));
}

#[test]
fn test_lender_try_find_short_circuit() {
    let mut lender = VecLender::new(vec![1, 2, 3, 4, 5]);
    let result: Result<Option<&i32>, String> = lender.try_find(|x| {
        if **x == 3 {
            Err("hit 3".to_string())
        } else {
            Ok(**x > 10)
        }
    });
    assert_eq!(result, Err("hit 3".to_string()));
}

#[test]
fn test_lender_try_find_empty() {
    let mut lender = VecLender::new(vec![]);
    let result: Result<Option<&i32>, String> = lender.try_find(|_| Ok(true));
    assert_eq!(result, Ok(None));
}

// ============================================================================
// is_partitioned and collect_into tests
// ============================================================================

#[test]
fn test_lender_is_partitioned_true() {
    // All true elements come before all false elements
    let lender = VecLender::new(vec![2, 4, 6, 1, 3, 5]);
    assert!(lender.is_partitioned(|x| *x % 2 == 0));
}

#[test]
fn test_lender_is_partitioned_all_true() {
    let lender = VecLender::new(vec![2, 4, 6]);
    assert!(lender.is_partitioned(|x| *x % 2 == 0));
}

#[test]
fn test_lender_is_partitioned_all_false() {
    let lender = VecLender::new(vec![1, 3, 5]);
    assert!(lender.is_partitioned(|x| *x % 2 == 0));
}

#[test]
fn test_lender_is_partitioned_false() {
    // false, true, false — not partitioned
    let lender = VecLender::new(vec![1, 2, 3]);
    assert!(!lender.is_partitioned(|x| *x % 2 == 0));
}

#[test]
fn test_lender_is_partitioned_empty() {
    let lender = VecLender::new(vec![]);
    assert!(lender.is_partitioned(|_: &i32| true));
}

#[test]
fn test_lender_collect_into() {
    let lender = VecLender::new(vec![1, 2, 3, 4, 5]);
    let mut result = I32Collector(Vec::new());
    lender.collect_into(&mut result);
    assert_eq!(result.0, vec![1, 2, 3, 4, 5]);
}

#[test]
fn test_lender_collect_into_existing() {
    let lender = VecLender::new(vec![4, 5, 6]);
    let mut result = I32Collector(vec![1, 2, 3]);
    lender.collect_into(&mut result);
    assert_eq!(result.0, vec![1, 2, 3, 4, 5, 6]);
}

// ============================================================================
// by_ref tests
// ============================================================================
//
// Note: Infallible comparison methods (partial_cmp, eq, ne, lt, le, gt, ge)
// cannot be tested due to Rust compiler limitations with HRTB trait bounds.
// These methods require `for<'all> Lend<'all, Self>: PartialOrd<Lend<'all, L::Lender>>`
// or `PartialEq` bounds, which the trait solver cannot currently satisfy even
// with owned types. Use the `_by` variants (partial_cmp_by, eq_by, etc.)
// instead - these are tested in the "Comparison method tests" section above.
//
// ============================================================================

#[test]
fn test_lender_by_ref() {
    let mut lender = VecLender::new(vec![1, 2, 3, 4, 5]);
    // Use by_ref to take 2 elements without consuming the lender
    {
        let by_ref = lender.by_ref();
        let mut taken = by_ref.take(2);
        assert_eq!(taken.next(), Some(&1));
        assert_eq!(taken.next(), Some(&2));
        assert_eq!(taken.next(), None);
    }
    // Remaining elements still available
    assert_eq!(lender.next(), Some(&3));
    assert_eq!(lender.next(), Some(&4));
    assert_eq!(lender.next(), Some(&5));
    assert_eq!(lender.next(), None);
}

#[test]
fn test_lender_by_ref_with_skip() {
    let mut lender = VecLender::new(vec![1, 2, 3, 4, 5, 6]);
    // Skip 2 via by_ref
    {
        let by_ref = lender.by_ref();
        let _ = by_ref.skip(2).next(); // consumes 1, 2, returns 3
    }
    // After skip(2).next(), 1, 2, 3 consumed
    assert_eq!(lender.next(), Some(&4));
}
