use core::num::NonZeroUsize;
use core::ops::ControlFlow;

use crate::{
    DoubleEndedFallibleLender, ExactSizeFallibleLender, FallibleLend, FallibleLender,
    FallibleLending, FusedFallibleLender, Take, try_trait_v2::Try,
};

impl<L: FallibleLender> Take<L> {
    #[inline]
    pub(crate) fn new_fallible(lender: L, n: usize) -> Take<L> {
        crate::__check_fallible_lender_covariance::<L>();
        Take { lender, n }
    }
}

impl<'lend, L> FallibleLending<'lend> for Take<L>
where
    L: FallibleLender,
{
    type Lend = FallibleLend<'lend, L>;
}

impl<L> FallibleLender for Take<L>
where
    L: FallibleLender,
{
    type Error = L::Error;
    // SAFETY: the lend is that of L
    crate::unsafe_assume_covariance_fallible!();

    #[inline]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        if self.n != 0 {
            self.n -= 1;
            self.lender.next()
        } else {
            Ok(None)
        }
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        if self.n > n {
            self.n -= n + 1;
            self.lender.nth(n)
        } else {
            if self.n > 0 {
                self.lender.nth(self.n - 1)?;
                self.n = 0;
            }
            Ok(None)
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.n == 0 {
            return (0, Some(0));
        }

        let (lower, upper) = self.lender.size_hint();

        let lower = lower.min(self.n);

        let upper = match upper {
            Some(x) if x < self.n => Some(x),
            _ => Some(self.n),
        };

        (lower, upper)
    }

    #[inline]
    fn advance_by(&mut self, n: usize) -> Result<Result<(), NonZeroUsize>, Self::Error> {
        let min = self.n.min(n);
        let rem = match self.lender.advance_by(min)? {
            Ok(()) => 0,
            Err(rem) => rem.get(),
        };
        let advanced = min - rem;
        self.n -= advanced;
        Ok(NonZeroUsize::new(n - advanced).map_or(Ok(()), Err))
    }

    #[inline]
    fn try_fold<B, F, R>(&mut self, init: B, mut f: F) -> Result<R, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: Try<Output = B>,
    {
        if self.n == 0 {
            return Ok(R::from_output(init));
        }
        let n = &mut self.n;
        match self.lender.try_fold(init, move |acc, x| {
            *n -= 1;
            let r = f(acc, x)?;
            if *n == 0 {
                Ok(ControlFlow::Break(r))
            } else {
                match r.branch() {
                    ControlFlow::Continue(b) => Ok(ControlFlow::Continue(b)),
                    ControlFlow::Break(residual) => {
                        Ok(ControlFlow::Break(R::from_residual(residual)))
                    }
                }
            }
        })? {
            ControlFlow::Continue(acc) => Ok(R::from_output(acc)),
            ControlFlow::Break(r) => Ok(r),
        }
    }

    #[inline]
    fn fold<B, F>(mut self, init: B, mut f: F) -> Result<B, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<B, Self::Error>,
    {
        if self.n == 0 {
            return Ok(init);
        }
        let n = &mut self.n;
        match self.lender.try_fold(init, move |acc, x| {
            *n -= 1;
            let acc = f(acc, x)?;
            if *n == 0 {
                Ok(ControlFlow::Break(acc))
            } else {
                Ok(ControlFlow::Continue(acc))
            }
        })? {
            ControlFlow::Continue(acc) | ControlFlow::Break(acc) => Ok(acc),
        }
    }
}

impl<L> DoubleEndedFallibleLender for Take<L>
where
    L: DoubleEndedFallibleLender + ExactSizeFallibleLender,
{
    #[inline]
    fn next_back(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        if self.n != 0 {
            let n = self.n;
            self.n -= 1;
            self.lender.nth_back(self.lender.len().saturating_sub(n))
        } else {
            Ok(None)
        }
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        let len = self.lender.len();
        if self.n > n {
            let m = len.saturating_sub(self.n) + n;
            self.n -= n + 1;
            self.lender.nth_back(m)
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
        if self.n == 0 {
            Ok(R::from_output(init))
        } else {
            let len = self.lender.len();
            if len > self.n && self.lender.nth_back(len - self.n - 1)?.is_none() {
                Ok(R::from_output(init))
            } else {
                let n = &mut self.n;
                self.lender.try_rfold(init, |acc, x| {
                    *n -= 1;
                    f(acc, x)
                })
            }
        }
    }

    #[inline]
    fn rfold<B, F>(mut self, init: B, f: F) -> Result<B, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<B, Self::Error>,
    {
        if self.n == 0 {
            Ok(init)
        } else {
            let len = self.lender.len();
            if len > self.n && self.lender.nth_back(len - self.n - 1)?.is_none() {
                Ok(init)
            } else {
                self.lender.rfold(init, f)
            }
        }
    }

    #[inline]
    fn advance_back_by(&mut self, n: usize) -> Result<Result<(), NonZeroUsize>, Self::Error> {
        let trim_inner = self.lender.len().saturating_sub(self.n);
        let advance_by = trim_inner.saturating_add(n);
        let remainder = match self.lender.advance_back_by(advance_by)? {
            Ok(()) => 0,
            Err(rem) => rem.get(),
        };
        let advanced_by_inner = advance_by - remainder;
        let advanced_by = advanced_by_inner - trim_inner;
        self.n -= advanced_by;
        Ok(NonZeroUsize::new(n - advanced_by).map_or(Ok(()), Err))
    }
}

impl<L> ExactSizeFallibleLender for Take<L> where L: ExactSizeFallibleLender {}

impl<L> FusedFallibleLender for Take<L> where L: FusedFallibleLender {}
