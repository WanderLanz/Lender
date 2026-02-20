use core::num::NonZeroUsize;
use core::ops::ControlFlow;

use crate::{
    DoubleEndedLender, ExactSizeLender, FusedLender, Lend, Lender, Lending, try_trait_v2::Try,
};

/// A lender that only yields the first `n` elements of the underlying lender.
///
/// This `struct` is created by the [`take()`](crate::Lender::take) method on
/// [`Lender`].
#[derive(Clone, Debug)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Take<L> {
    pub(crate) lender: L,
    pub(crate) n: usize,
}

impl<L> Take<L> {
    /// Returns the inner lender.
    #[inline]
    pub fn into_inner(self) -> L {
        self.lender
    }

    /// Returns the inner lender and the remaining number of elements to take.
    #[inline]
    pub fn into_parts(self) -> (L, usize) {
        (self.lender, self.n)
    }
}

impl<L: Lender> Take<L> {
    #[inline]
    pub(crate) fn new(lender: L, n: usize) -> Take<L> {
        crate::__check_lender_covariance::<L>();
        Take { lender, n }
    }
}

impl<'lend, L> Lending<'lend> for Take<L>
where
    L: Lender,
{
    type Lend = Lend<'lend, L>;
}

impl<L> Lender for Take<L>
where
    L: Lender,
{
    // SAFETY: the lend is that of L
    crate::unsafe_assume_covariance!();
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        if self.n != 0 {
            self.n -= 1;
            self.lender.next()
        } else {
            None
        }
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Lend<'_, Self>> {
        if self.n > n {
            self.n -= n + 1;
            self.lender.nth(n)
        } else {
            if self.n > 0 {
                self.lender.nth(self.n - 1);
                self.n = 0;
            }
            None
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
    fn advance_by(&mut self, n: usize) -> Result<(), NonZeroUsize> {
        let min = self.n.min(n);
        let rem = match self.lender.advance_by(min) {
            Ok(()) => 0,
            Err(rem) => rem.get(),
        };
        let advanced = min - rem;
        self.n -= advanced;
        NonZeroUsize::new(n - advanced).map_or(Ok(()), Err)
    }

    #[inline]
    fn try_fold<B, F, R>(&mut self, init: B, mut f: F) -> R
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> R,
        R: Try<Output = B>,
    {
        if self.n == 0 {
            return R::from_output(init);
        }
        let n = &mut self.n;
        match self.lender.try_fold(init, move |acc, x| {
            *n -= 1;
            let r = f(acc, x);
            if *n == 0 {
                ControlFlow::Break(r)
            } else {
                match r.branch() {
                    ControlFlow::Continue(b) => ControlFlow::Continue(b),
                    ControlFlow::Break(residual) => ControlFlow::Break(R::from_residual(residual)),
                }
            }
        }) {
            ControlFlow::Continue(acc) => R::from_output(acc),
            ControlFlow::Break(r) => r,
        }
    }

    #[inline]
    fn fold<B, F>(mut self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> B,
    {
        if self.n == 0 {
            return init;
        }
        let n = &mut self.n;
        match self.lender.try_fold(init, move |acc, x| {
            *n -= 1;
            let acc = f(acc, x);
            if *n == 0 {
                ControlFlow::Break(acc)
            } else {
                ControlFlow::Continue(acc)
            }
        }) {
            ControlFlow::Continue(acc) | ControlFlow::Break(acc) => acc,
        }
    }
}

impl<L> DoubleEndedLender for Take<L>
where
    L: DoubleEndedLender + ExactSizeLender,
{
    #[inline]
    fn next_back(&mut self) -> Option<Lend<'_, Self>> {
        if self.n != 0 {
            let n = self.n;
            self.n -= 1;
            self.lender.nth_back(self.lender.len().saturating_sub(n))
        } else {
            None
        }
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Lend<'_, Self>> {
        let len = self.lender.len();
        if self.n > n {
            let m = len.saturating_sub(self.n) + n;
            self.n -= n + 1;
            self.lender.nth_back(m)
        } else {
            if len > 0 {
                self.lender.nth_back(len - 1);
            }
            None
        }
    }

    #[inline]
    fn try_rfold<B, F, R>(&mut self, init: B, mut f: F) -> R
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> R,
        R: Try<Output = B>,
    {
        if self.n == 0 {
            R::from_output(init)
        } else {
            let len = self.lender.len();
            if len > self.n && self.lender.nth_back(len - self.n - 1).is_none() {
                R::from_output(init)
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
    fn rfold<B, F>(mut self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> B,
    {
        if self.n == 0 {
            init
        } else {
            let len = self.lender.len();
            if len > self.n && self.lender.nth_back(len - self.n - 1).is_none() {
                init
            } else {
                self.lender.rfold(init, f)
            }
        }
    }

    #[inline]
    fn advance_back_by(&mut self, n: usize) -> Result<(), NonZeroUsize> {
        // Relies on ExactSizeLender: if len() is inaccurate,
        // advanced_by_inner - trim_inner may underflow.
        let trim_inner = self.lender.len().saturating_sub(self.n);
        let advance_by = trim_inner.saturating_add(n);
        let remainder = match self.lender.advance_back_by(advance_by) {
            Ok(()) => 0,
            Err(rem) => rem.get(),
        };
        let advanced_by_inner = advance_by - remainder;
        let advanced_by = advanced_by_inner - trim_inner;
        self.n -= advanced_by;
        NonZeroUsize::new(n - advanced_by).map_or(Ok(()), Err)
    }
}

impl<L> ExactSizeLender for Take<L> where L: ExactSizeLender {}

impl<L> FusedLender for Take<L> where L: FusedLender {}
