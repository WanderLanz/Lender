use core::ops::ControlFlow;

use crate::{
    DoubleEndedLender, ExactSizeLender, FusedLender, Lend, Lender, Lending,
    try_trait_v2::{FromResidual, Try},
};

/// A lender that yields [`None`] forever after the underlying lender yields
/// [`None`] once.
///
/// This `struct` is created by the
/// [`fuse()`](crate::Lender::fuse) method on [`Lender`].
#[derive(Clone, Debug)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Fuse<L> {
    pub(crate) lender: L,
    pub(crate) flag: bool,
}

impl<L> Fuse<L> {
    /// Returns the inner lender.
    #[inline(always)]
    pub fn into_inner(self) -> L {
        self.lender
    }
}

impl<L: Lender> Fuse<L> {
    #[inline(always)]
    pub(crate) fn new(lender: L) -> Fuse<L> {
        crate::__check_lender_covariance::<L>();
        Fuse {
            lender,
            flag: false,
        }
    }
}

impl<'lend, L> Lending<'lend> for Fuse<L>
where
    L: Lender,
{
    type Lend = Lend<'lend, L>;
}

impl<L> Lender for Fuse<L>
where
    L: Lender,
{
    // SAFETY: the lend is that of L
    crate::unsafe_assume_covariance!();
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        if !self.flag {
            if let x @ Some(_) = self.lender.next() {
                return x;
            }
            self.flag = true;
        }
        None
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Lend<'_, Self>> {
        if !self.flag {
            if let x @ Some(_) = self.lender.nth(n) {
                return x;
            }
            self.flag = true;
        }
        None
    }

    #[inline]
    fn last(&mut self) -> Option<Lend<'_, Self>>
    where
        Self: Sized,
    {
        if !self.flag {
            if let x @ Some(_) = self.lender.last() {
                return x;
            }
            self.flag = true;
        }
        None
    }

    #[inline]
    fn count(self) -> usize {
        if !self.flag { self.lender.count() } else { 0 }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        if !self.flag {
            self.lender.size_hint()
        } else {
            (0, Some(0))
        }
    }

    #[inline]
    fn try_fold<Acc, F, R>(&mut self, mut acc: Acc, mut f: F) -> R
    where
        Self: Sized,
        F: FnMut(Acc, Lend<'_, Self>) -> R,
        R: Try<Output = Acc>,
    {
        if !self.flag {
            acc = match self.lender.try_fold(acc, &mut f).branch() {
                ControlFlow::Continue(x) => x,
                ControlFlow::Break(x) => return FromResidual::from_residual(x),
            };
            self.flag = true;
        }
        Try::from_output(acc)
    }

    #[inline]
    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> B,
    {
        if !self.flag {
            self.lender.fold(init, &mut f)
        } else {
            init
        }
    }

    #[inline]
    fn find<P>(&mut self, mut predicate: P) -> Option<Lend<'_, Self>>
    where
        Self: Sized,
        P: FnMut(&Lend<'_, Self>) -> bool,
    {
        if !self.flag {
            if let x @ Some(_) = self.lender.find(&mut predicate) {
                return x;
            }
            self.flag = true;
        }
        None
    }
}

impl<L> DoubleEndedLender for Fuse<L>
where
    L: DoubleEndedLender,
{
    #[inline]
    fn next_back(&mut self) -> Option<Lend<'_, Self>> {
        if !self.flag {
            if let x @ Some(_) = self.lender.next_back() {
                return x;
            }
            self.flag = true;
        }
        None
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Lend<'_, Self>> {
        if !self.flag {
            if let x @ Some(_) = self.lender.nth_back(n) {
                return x;
            }
            self.flag = true;
        }
        None
    }

    #[inline]
    fn try_rfold<Acc, F, R>(&mut self, mut acc: Acc, mut f: F) -> R
    where
        Self: Sized,
        F: FnMut(Acc, Lend<'_, Self>) -> R,
        R: Try<Output = Acc>,
    {
        if !self.flag {
            acc = match self.lender.try_rfold(acc, &mut f).branch() {
                ControlFlow::Continue(x) => x,
                ControlFlow::Break(x) => return FromResidual::from_residual(x),
            };
            self.flag = true;
        }
        Try::from_output(acc)
    }

    #[inline]
    fn rfold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> B,
    {
        if !self.flag {
            self.lender.rfold(init, &mut f)
        } else {
            init
        }
    }

    #[inline]
    fn rfind<P>(&mut self, mut predicate: P) -> Option<Lend<'_, Self>>
    where
        Self: Sized,
        P: FnMut(&Lend<'_, Self>) -> bool,
    {
        if !self.flag {
            if let x @ Some(_) = self.lender.rfind(&mut predicate) {
                return x;
            }
            self.flag = true;
        }
        None
    }
}

impl<L> ExactSizeLender for Fuse<L>
where
    L: ExactSizeLender,
{
    #[inline]
    fn len(&self) -> usize {
        if !self.flag { self.lender.len() } else { 0 }
    }

    #[inline]
    fn is_empty(&self) -> bool {
        if !self.flag {
            self.lender.is_empty()
        } else {
            true
        }
    }
}

impl<L> FusedLender for Fuse<L> where L: Lender {}

impl<L: Default + Lender> Default for Fuse<L> {
    #[inline(always)]
    fn default() -> Self {
        Fuse::new(L::default())
    }
}
