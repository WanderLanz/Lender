use crate::{
    try_trait_v2::Try, FallibleLend, FallibleLender, FallibleLending, FusedFallibleLender,
    FusedLender, Lend, Lender, Lending,
};
use core::ops::ControlFlow;

#[derive(Debug)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Chunk<'s, T> {
    lender: &'s mut T,
    len: usize,
}

impl<'s, T> Chunk<'s, T> {
    pub(crate) fn new(lender: &'s mut T, len: usize) -> Self {
        Self { lender, len }
    }

    pub fn into_inner(self) -> &'s mut T {
        self.lender
    }

    /// Returns the inner lender and the remaining chunk length.
    pub fn into_parts(self) -> (&'s mut T, usize) {
        (self.lender, self.len)
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

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Lend<'_, Self>> {
        if n >= self.len {
            // Exhaust remaining len
            for _ in 0..self.len {
                self.lender.next();
            }
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

impl<'lend, T> FallibleLending<'lend> for Chunk<'_, T>
where
    T: FallibleLender,
{
    type Lend = FallibleLend<'lend, T>;
}

impl<T> FallibleLender for Chunk<'_, T>
where
    T: FallibleLender,
{
    type Error = T::Error;
    // SAFETY: the lend is that of T
    crate::unsafe_assume_covariance_fallible!();

    #[inline]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        if self.len == 0 {
            Ok(None)
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
    fn count(self) -> Result<usize, Self::Error>
    where
        Self: Sized,
    {
        self.fold(0, |count, _| Ok(count + 1))
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        if n >= self.len {
            for _ in 0..self.len {
                self.lender.next()?;
            }
            self.len = 0;
            Ok(None)
        } else {
            self.len -= n + 1;
            self.lender.nth(n)
        }
    }

    #[inline]
    fn try_fold<B, F, R>(&mut self, init: B, mut f: F) -> Result<R, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: Try<Output = B>,
    {
        let mut acc = init;
        while self.len > 0 {
            self.len -= 1;
            match self.lender.next()? {
                Some(x) => {
                    acc = match f(acc, x)?.branch() {
                        ControlFlow::Continue(v) => v,
                        ControlFlow::Break(r) => return Ok(R::from_residual(r)),
                    };
                }
                None => break,
            }
        }
        Ok(R::from_output(acc))
    }

    #[inline]
    fn fold<B, F>(mut self, init: B, mut f: F) -> Result<B, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<B, Self::Error>,
    {
        let mut acc = init;
        while self.len > 0 {
            self.len -= 1;
            match self.lender.next()? {
                Some(x) => acc = f(acc, x)?,
                None => break,
            }
        }
        Ok(acc)
    }
}

impl<L> FusedFallibleLender for Chunk<'_, L> where L: FusedFallibleLender {}
