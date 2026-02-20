use core::ops::ControlFlow;

use crate::{FusedLender, Lend, Lender, Lending, try_trait_v2::Try};

/// A sub-lender over elements of a chunk in a [`Chunky`](crate::Chunky) lender.
///
/// This `struct` is created by the [`Chunky`](crate::Chunky) lender during
/// iteration.
#[derive(Debug)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Chunk<'s, T> {
    pub(crate) lender: &'s mut T,
    pub(crate) len: usize,
}

impl<'s, T> Chunk<'s, T> {
    /// Returns the inner lender.
    #[inline(always)]
    pub fn into_inner(self) -> &'s mut T {
        self.lender
    }

    /// Returns the inner lender and the remaining chunk length.
    #[inline(always)]
    pub fn into_parts(self) -> (&'s mut T, usize) {
        (self.lender, self.len)
    }
}

impl<'s, T: Lender> Chunk<'s, T> {
    #[inline(always)]
    pub(crate) fn new(lender: &'s mut T, len: usize) -> Self {
        crate::__check_lender_covariance::<T>();
        Self { lender, len }
    }
}

impl<'lend, T> Lending<'lend> for Chunk<'_, T>
where
    T: Lender,
{
    type Lend = Lend<'lend, T>;
}

impl<T> Lender for Chunk<'_, T>
where
    T: Lender,
{
    // SAFETY: the lend is that of T
    crate::unsafe_assume_covariance!();
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            self.lender.next()
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower, upper) = self.lender.size_hint();
        (lower.min(self.len), upper.map(|x| x.min(self.len)))
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.fold(0, |count, _| count + 1)
    }

    /// Returns the `n`th element of this chunk.
    ///
    /// If `n` is greater than or equal to the remaining length of the chunk,
    /// the remaining chunk capacity is exhausted from the underlying lender,
    /// and `None` is returned.
    #[inline]
    fn nth(&mut self, n: usize) -> Option<Lend<'_, Self>> {
        if n >= self.len {
            let _ = self.lender.advance_by(self.len);
            self.len = 0;
            None
        } else {
            self.len -= n + 1;
            self.lender.nth(n)
        }
    }

    #[inline]
    fn try_fold<B, F, R>(&mut self, init: B, mut f: F) -> R
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> R,
        R: Try<Output = B>,
    {
        let mut acc = init;
        while self.len > 0 {
            self.len -= 1;
            match self.lender.next() {
                Some(x) => {
                    acc = match f(acc, x).branch() {
                        ControlFlow::Continue(v) => v,
                        ControlFlow::Break(r) => return R::from_residual(r),
                    };
                }
                None => break,
            }
        }
        R::from_output(acc)
    }

    #[inline]
    fn fold<B, F>(mut self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> B,
    {
        let mut acc = init;
        while self.len > 0 {
            self.len -= 1;
            match self.lender.next() {
                Some(x) => acc = f(acc, x),
                None => break,
            }
        }
        acc
    }
}

impl<L> FusedLender for Chunk<'_, L> where L: FusedLender {}
