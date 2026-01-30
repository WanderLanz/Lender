use core::fmt;
use core::ops::ControlFlow;

use crate::{FusedLender, Lend, Lender, Lending, try_trait_v2::Try};

#[derive(Clone)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct TakeWhile<L, P> {
    pub(crate) lender: L,
    pub(crate) flag: bool,
    pub(crate) predicate: P,
}

impl<L, P> TakeWhile<L, P> {
    #[inline(always)]
    pub(crate) fn new(lender: L, predicate: P) -> TakeWhile<L, P> {
        TakeWhile {
            lender,
            flag: false,
            predicate,
        }
    }

    #[inline(always)]
    pub fn into_inner(self) -> L {
        self.lender
    }

    /// Returns the inner lender and the predicate.
    #[inline(always)]
    pub fn into_parts(self) -> (L, P) {
        (self.lender, self.predicate)
    }
}

impl<L: fmt::Debug, P> fmt::Debug for TakeWhile<L, P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TakeWhile")
            .field("lender", &self.lender)
            .finish_non_exhaustive()
    }
}

impl<'lend, L, P> Lending<'lend> for TakeWhile<L, P>
where
    P: FnMut(&Lend<'lend, L>) -> bool,
    L: Lender,
{
    type Lend = Lend<'lend, L>;
}

impl<L, P> Lender for TakeWhile<L, P>
where
    P: FnMut(&Lend<'_, L>) -> bool,
    L: Lender,
{
    // SAFETY: the lend is that of L
    crate::unsafe_assume_covariance!();
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        if !self.flag {
            let x = self.lender.next()?;
            if (self.predicate)(&x) {
                return Some(x);
            }
            self.flag = true;
        }
        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.flag {
            (0, Some(0))
        } else {
            let (_, upper) = self.lender.size_hint();
            (0, upper)
        }
    }

    #[inline]
    fn try_fold<B, F, R>(&mut self, init: B, mut f: F) -> R
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> R,
        R: Try<Output = B>,
    {
        if self.flag {
            return R::from_output(init);
        }
        let flag = &mut self.flag;
        let predicate = &mut self.predicate;
        match self.lender.try_fold(init, move |acc, x| {
            if (predicate)(&x) {
                match f(acc, x).branch() {
                    ControlFlow::Continue(b) => ControlFlow::Continue(b),
                    ControlFlow::Break(residual) => {
                        ControlFlow::Break(R::from_residual(residual))
                    }
                }
            } else {
                *flag = true;
                ControlFlow::Break(R::from_output(acc))
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
        if self.flag {
            return init;
        }
        let flag = &mut self.flag;
        let predicate = &mut self.predicate;
        match self.lender.try_fold(init, move |acc, x| {
            if (predicate)(&x) {
                ControlFlow::Continue(f(acc, x))
            } else {
                *flag = true;
                ControlFlow::Break(acc)
            }
        }) {
            ControlFlow::Continue(acc) | ControlFlow::Break(acc) => acc,
        }
    }
}

impl<L, P> FusedLender for TakeWhile<L, P>
where
    P: FnMut(&Lend<'_, L>) -> bool,
    L: Lender,
{
}
