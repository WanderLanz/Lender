use core::ops::ControlFlow;

use crate::{
    Chunk, FallibleLend, FallibleLender, FallibleLending, FusedFallibleLender, try_trait_v2::Try,
};

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
            let _ = self.lender.advance_by(self.len)?;
            self.len = 0;
            Ok(None)
        } else {
            let result = self.lender.nth(n)?;
            self.len -= n + 1;
            Ok(result)
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
