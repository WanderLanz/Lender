use core::num::NonZeroUsize;

use crate::{
    DoubleEndedFallibleLender, ExactSizeFallibleLender, FallibleLend, FallibleLender,
    FallibleLending, FusedFallibleLender, Skip, try_trait_v2::Try,
};

impl<'lend, L> FallibleLending<'lend> for Skip<L>
where
    L: FallibleLender,
{
    type Lend = FallibleLend<'lend, L>;
}

impl<L> FallibleLender for Skip<L>
where
    L: FallibleLender,
{
    type Error = L::Error;
    // SAFETY: the lend is that of L
    crate::unsafe_assume_covariance_fallible!();

    #[inline]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        if self.n > 0 {
            self.lender.nth(core::mem::take(&mut self.n))
        } else {
            self.lender.next()
        }
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        if self.n > 0 {
            let skip = core::mem::take(&mut self.n);
            let n = match skip.checked_add(n) {
                Some(nth) => nth,
                None => {
                    self.lender.nth(skip - 1)?;
                    n
                }
            };
            self.lender.nth(n)
        } else {
            self.lender.nth(n)
        }
    }

    #[inline]
    fn count(mut self) -> Result<usize, <L as FallibleLender>::Error> {
        if self.n > 0 && self.lender.nth(self.n - 1)?.is_none() {
            return Ok(0);
        }
        self.lender.count()
    }

    #[inline]
    fn last(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error>
    where
        Self: Sized,
    {
        if self.n > 0 && self.lender.nth(self.n - 1)?.is_none() {
            return Ok(None);
        }
        self.lender.last()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower, upper) = self.lender.size_hint();

        let lower = lower.saturating_sub(self.n);
        let upper = upper.map(|x| x.saturating_sub(self.n));

        (lower, upper)
    }

    #[inline]
    fn try_fold<B, F, R>(&mut self, init: B, f: F) -> Result<R, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: Try<Output = B>,
    {
        let n = self.n;
        self.n = 0;
        if n > 0 && self.lender.nth(n - 1)?.is_none() {
            return Ok(R::from_output(init));
        }
        self.lender.try_fold(init, f)
    }

    #[inline]
    fn fold<B, F>(mut self, init: B, f: F) -> Result<B, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<B, Self::Error>,
    {
        if self.n > 0 && self.lender.nth(self.n - 1)?.is_none() {
            return Ok(init);
        }
        self.lender.fold(init, f)
    }

    #[inline]
    fn advance_by(&mut self, mut n: usize) -> Result<Result<(), NonZeroUsize>, Self::Error> {
        let skip_inner = self.n;
        let skip_and_advance = skip_inner.saturating_add(n);

        let remainder = match self.lender.advance_by(skip_and_advance)? {
            Ok(()) => 0,
            Err(rem) => rem.get(),
        };
        let advanced_inner = skip_and_advance - remainder;
        n -= advanced_inner.saturating_sub(skip_inner);
        self.n = self.n.saturating_sub(advanced_inner);

        if remainder == 0 && n > 0 {
            n = match self.lender.advance_by(n)? {
                Ok(()) => 0,
                Err(rem) => rem.get(),
            }
        }

        Ok(NonZeroUsize::new(n).map_or(Ok(()), Err))
    }
}

impl<L> ExactSizeFallibleLender for Skip<L> where L: ExactSizeFallibleLender {}

impl<L> DoubleEndedFallibleLender for Skip<L>
where
    L: DoubleEndedFallibleLender + ExactSizeFallibleLender,
{
    #[inline]
    fn next_back(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        if self.len() > 0 {
            self.lender.next_back()
        } else {
            Ok(None)
        }
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        let len = self.len();
        if len > n {
            self.lender.nth_back(n)
        } else {
            if len > 0 {
                self.lender.nth_back(len - 1)?;
            }
            Ok(None)
        }
    }

    #[inline]
    fn try_rfold<B, F, R>(&mut self, init: B, mut f: F) -> Result<R, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: Try<Output = B>,
    {
        use core::ops::ControlFlow;
        let mut len = self.len();
        if len == 0 {
            Ok(R::from_output(init))
        } else {
            match self.lender.try_rfold(init, |acc, x| {
                len -= 1;
                let r = f(acc, x)?;
                if len == 0 {
                    Ok(ControlFlow::Break(r))
                } else {
                    match r.branch() {
                        ControlFlow::Continue(r) => Ok(ControlFlow::Continue(r)),
                        ControlFlow::Break(r) => Ok(ControlFlow::Break(R::from_residual(r))),
                    }
                }
            })? {
                ControlFlow::Continue(r) => Ok(R::from_output(r)),
                ControlFlow::Break(r) => Ok(r),
            }
        }
    }

    #[inline]
    fn advance_back_by(&mut self, n: usize) -> Result<Result<(), NonZeroUsize>, Self::Error> {
        let min = core::cmp::min(self.len(), n);
        let rem = self.lender.advance_back_by(min)?;
        assert!(rem.is_ok(), "ExactSizeFallibleLender contract violation");
        Ok(NonZeroUsize::new(n - min).map_or(Ok(()), Err))
    }
}

impl<L> FusedFallibleLender for Skip<L> where L: FusedFallibleLender {}
