use core::{fmt, ops::ControlFlow};

use crate::{FusedLender, Lend, Lender, Lending, try_trait_v2::Try};

/// A lender that rejects elements while a predicate returns `true`.
///
/// This `struct` is created by the [`skip_while()`](crate::Lender::skip_while)
/// method on [`Lender`].
#[derive(Clone)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct SkipWhile<L, P> {
    pub(crate) lender: L,
    pub(crate) flag: bool,
    pub(crate) predicate: P,
}

impl<L, P> SkipWhile<L, P> {
    /// Returns the inner lender.
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

impl<L: Lender, P> SkipWhile<L, P> {
    #[inline(always)]
    pub(crate) fn new(lender: L, predicate: P) -> SkipWhile<L, P> {
        crate::__check_lender_covariance::<L>();
        SkipWhile {
            lender,
            flag: false,
            predicate,
        }
    }
}

impl<L: fmt::Debug, P> fmt::Debug for SkipWhile<L, P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SkipWhile")
            .field("lender", &self.lender)
            .finish_non_exhaustive()
    }
}

impl<'lend, L, P> Lending<'lend> for SkipWhile<L, P>
where
    P: FnMut(&Lend<'lend, L>) -> bool,
    L: Lender,
{
    type Lend = Lend<'lend, L>;
}

impl<L, P> Lender for SkipWhile<L, P>
where
    P: FnMut(&Lend<'_, L>) -> bool,
    L: Lender,
{
    // SAFETY: the lend is that of L
    crate::unsafe_assume_covariance!();
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        let flag = &mut self.flag;
        let predicate = &mut self.predicate;
        self.lender.find(move |x| {
            if *flag || !(predicate)(x) {
                *flag = true;
                true
            } else {
                false
            }
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.lender.size_hint();
        (0, upper)
    }

    #[inline]
    fn try_fold<B, F, R>(&mut self, mut init: B, mut f: F) -> R
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> R,
        R: Try<Output = B>,
    {
        if !self.flag {
            match self.next() {
                Some(x) => {
                    init = match f(init, x).branch() {
                        ControlFlow::Break(x) => return R::from_residual(x),
                        ControlFlow::Continue(x) => x,
                    }
                }
                None => return R::from_output(init),
            }
        }
        self.lender.try_fold(init, f)
    }

    #[inline]
    fn fold<B, F>(mut self, mut init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> B,
    {
        if !self.flag {
            match self.next() {
                Some(x) => init = f(init, x),
                None => return init,
            }
        }
        self.lender.fold(init, f)
    }
}

impl<L, P> FusedLender for SkipWhile<L, P>
where
    P: FnMut(&Lend<'_, L>) -> bool,
    L: FusedLender,
{
}
