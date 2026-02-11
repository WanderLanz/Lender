use core::ops::ControlFlow;

use crate::{
    DoubleEndedFallibleLender, ExactSizeFallibleLender, FallibleLend, FallibleLender,
    FallibleLending, FusedFallibleLender, StepBy, try_trait_v2::Try,
};

impl<'lend, L> FallibleLending<'lend> for StepBy<L>
where
    L: FallibleLender,
{
    type Lend = FallibleLend<'lend, L>;
}

impl<L> FallibleLender for StepBy<L>
where
    L: FallibleLender,
{
    type Error = L::Error;
    // SAFETY: the lend is that of L
    crate::unsafe_assume_covariance_fallible!();

    #[inline]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        if self.first_take {
            self.first_take = false;
            self.lender.next()
        } else {
            self.lender.nth(self.step)
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (low, high) = self.lender.size_hint();
        let step = self.step;
        if self.first_take {
            let f = move |n| if n == 0 { 0 } else { 1 + (n - 1) / (step + 1) };
            (f(low), high.map(f))
        } else {
            let f = move |n| n / (step + 1);
            (f(low), high.map(f))
        }
    }

    #[inline]
    fn nth(&mut self, mut n: usize) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        if self.first_take {
            self.first_take = false;
            if n == 0 {
                return self.lender.next();
            }
            n -= 1;
        }
        let mut step = self.step + 1;
        if n == usize::MAX {
            self.lender.nth(step - 1)?;
        } else {
            n += 1;
        }
        loop {
            let mul = n.checked_mul(step);
            if let Some(mul) = mul {
                return self.lender.nth(mul - 1);
            }
            let div_n = usize::MAX / n;
            let div_step = usize::MAX / step;
            let nth_n = div_n * n;
            let nth_step = div_step * step;
            let nth = if nth_n > nth_step {
                step -= div_n;
                nth_n
            } else {
                n -= div_step;
                nth_step
            };
            if self.lender.nth(nth - 1)?.is_none() {
                return Ok(None);
            }
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
        if self.first_take {
            self.first_take = false;
            match self.lender.next()? {
                None => return Ok(R::from_output(acc)),
                Some(x) => {
                    acc = match f(acc, x)?.branch() {
                        ControlFlow::Break(b) => return Ok(R::from_residual(b)),
                        ControlFlow::Continue(c) => c,
                    }
                }
            }
        }
        while let Some(x) = self.lender.nth(self.step)? {
            acc = match f(acc, x)?.branch() {
                ControlFlow::Break(b) => return Ok(R::from_residual(b)),
                ControlFlow::Continue(c) => c,
            };
        }
        Ok(R::from_output(acc))
    }

    #[inline]
    fn count(self) -> Result<usize, Self::Error> {
        self.fold(0, |count, _| Ok(count + 1))
    }

    #[inline]
    fn fold<B, F>(mut self, init: B, mut f: F) -> Result<B, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<B, Self::Error>,
    {
        let mut acc = init;
        if self.first_take {
            self.first_take = false;
            match self.lender.next()? {
                None => return Ok(acc),
                Some(x) => acc = f(acc, x)?,
            }
        }
        while let Some(x) = self.lender.nth(self.step)? {
            acc = f(acc, x)?;
        }
        Ok(acc)
    }
}

impl<L> StepBy<L>
where
    L: ExactSizeFallibleLender,
{
    #[inline]
    fn next_back_index_fallible(&self) -> usize {
        let rem = self.lender.len() % (self.step + 1);
        if self.first_take {
            if rem == 0 { self.step } else { rem - 1 }
        } else {
            rem
        }
    }
}

impl<L> DoubleEndedFallibleLender for StepBy<L>
where
    L: DoubleEndedFallibleLender + ExactSizeFallibleLender,
{
    #[inline]
    fn next_back(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        self.lender.nth_back(self.next_back_index_fallible())
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        let n = n
            .saturating_mul(self.step + 1)
            .saturating_add(self.next_back_index_fallible());
        self.lender.nth_back(n)
    }

    #[inline]
    fn try_rfold<B, F, R>(&mut self, init: B, mut f: F) -> Result<R, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: Try<Output = B>,
    {
        let mut acc = init;
        match self.next_back()? {
            None => return Ok(R::from_output(acc)),
            Some(x) => {
                acc = match f(acc, x)?.branch() {
                    ControlFlow::Break(b) => return Ok(R::from_residual(b)),
                    ControlFlow::Continue(c) => c,
                };
            }
        }
        while let Some(x) = self.lender.nth_back(self.step)? {
            acc = match f(acc, x)?.branch() {
                ControlFlow::Break(b) => return Ok(R::from_residual(b)),
                ControlFlow::Continue(c) => c,
            };
        }
        Ok(R::from_output(acc))
    }

    #[inline]
    fn rfold<B, F>(mut self, init: B, mut f: F) -> Result<B, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<B, Self::Error>,
    {
        let mut acc = init;
        match self.next_back()? {
            None => return Ok(acc),
            Some(x) => acc = f(acc, x)?,
        }
        while let Some(x) = self.lender.nth_back(self.step)? {
            acc = f(acc, x)?;
        }
        Ok(acc)
    }
}

impl<L> ExactSizeFallibleLender for StepBy<L> where L: ExactSizeFallibleLender {}

impl<L> FusedFallibleLender for StepBy<L> where L: FusedFallibleLender {}
