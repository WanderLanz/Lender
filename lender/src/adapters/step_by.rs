use core::ops::ControlFlow;

use crate::{
    DoubleEndedLender, ExactSizeLender, FusedLender, Lend, Lender, Lending, try_trait_v2::Try,
};

/// A lender for stepping the underlying lender by a custom amount.
///
/// This `struct` is created by the [`step_by()`](crate::Lender::step_by)
/// method on [`Lender`].
#[derive(Clone, Debug)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct StepBy<L> {
    pub(crate) lender: L,
    pub(crate) step: usize,
    pub(crate) first_take: bool,
}

impl<L> StepBy<L> {
    #[inline]
    pub(crate) fn new(lender: L, step: usize) -> Self {
        assert_ne!(step, 0);
        StepBy {
            lender,
            step: step - 1,
            first_take: true,
        }
    }

    /// Returns the inner lender.
    #[inline(always)]
    pub fn into_inner(self) -> L {
        self.lender
    }

    /// Returns the inner lender and the step size.
    #[inline(always)]
    pub fn into_parts(self) -> (L, usize) {
        (self.lender, self.step + 1)
    }
}

impl<'lend, L> Lending<'lend> for StepBy<L>
where
    L: Lender,
{
    type Lend = Lend<'lend, L>;
}

impl<L> Lender for StepBy<L>
where
    L: Lender,
{
    // SAFETY: the lend is that of L
    crate::unsafe_assume_covariance!();
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
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
    fn nth(&mut self, mut n: usize) -> Option<Lend<'_, Self>> {
        if self.first_take {
            self.first_take = false;
            if n == 0 {
                return self.lender.next();
            }
            n -= 1;
        }
        let mut step = self.step + 1;
        if n == usize::MAX {
            self.lender.nth(step - 1);
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
            self.lender.nth(nth - 1)?;
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
        if self.first_take {
            self.first_take = false;
            match self.lender.next() {
                None => return R::from_output(acc),
                Some(x) => {
                    acc = match f(acc, x).branch() {
                        ControlFlow::Break(b) => return R::from_residual(b),
                        ControlFlow::Continue(c) => c,
                    }
                }
            }
        }
        while let Some(x) = self.lender.nth(self.step) {
            acc = match f(acc, x).branch() {
                ControlFlow::Break(b) => return R::from_residual(b),
                ControlFlow::Continue(c) => c,
            };
        }
        R::from_output(acc)
    }

    #[inline]
    fn count(self) -> usize {
        self.fold(0, |count, _| count + 1)
    }

    #[inline]
    fn fold<B, F>(mut self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> B,
    {
        let mut acc = init;
        if self.first_take {
            self.first_take = false;
            match self.lender.next() {
                None => return acc,
                Some(x) => acc = f(acc, x),
            }
        }
        while let Some(x) = self.lender.nth(self.step) {
            acc = f(acc, x);
        }
        acc
    }
}

impl<L> StepBy<L>
where
    L: ExactSizeLender,
{
    #[inline]
    fn next_back_index(&self) -> usize {
        let rem = self.lender.len() % (self.step + 1);
        if self.first_take {
            if rem == 0 { self.step } else { rem - 1 }
        } else {
            rem
        }
    }
}

impl<L> DoubleEndedLender for StepBy<L>
where
    L: DoubleEndedLender + ExactSizeLender,
{
    #[inline]
    fn next_back(&mut self) -> Option<Lend<'_, Self>> {
        self.lender.nth_back(self.next_back_index())
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Lend<'_, Self>> {
        let n = n
            .saturating_mul(self.step + 1)
            .saturating_add(self.next_back_index());
        self.lender.nth_back(n)
    }

    #[inline]
    fn try_rfold<B, F, R>(&mut self, init: B, mut f: F) -> R
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> R,
        R: Try<Output = B>,
    {
        let mut acc = init;
        match self.next_back() {
            None => return R::from_output(acc),
            Some(x) => {
                acc = match f(acc, x).branch() {
                    ControlFlow::Break(b) => return R::from_residual(b),
                    ControlFlow::Continue(c) => c,
                };
            }
        }
        while let Some(x) = self.lender.nth_back(self.step) {
            acc = match f(acc, x).branch() {
                ControlFlow::Break(b) => return R::from_residual(b),
                ControlFlow::Continue(c) => c,
            };
        }
        R::from_output(acc)
    }

    #[inline]
    fn rfold<B, F>(mut self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> B,
    {
        let mut acc = init;
        match self.next_back() {
            None => return acc,
            Some(x) => acc = f(acc, x),
        }
        while let Some(x) = self.lender.nth_back(self.step) {
            acc = f(acc, x);
        }
        acc
    }
}

impl<L> ExactSizeLender for StepBy<L> where L: ExactSizeLender {}

impl<L> FusedLender for StepBy<L> where L: FusedLender {}
