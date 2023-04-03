use core::ops::ControlFlow;

use crate::{
    try_trait_v2::{FromResidual, Try},
    Lender, Lending,
};
#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct Enumerate<L> {
    lender: L,
    count: usize,
}
impl<L> Enumerate<L> {
    pub(crate) fn new(lender: L) -> Enumerate<L> { Enumerate { lender, count: 0 } }
}
impl<'lend, L> Lending<'lend> for Enumerate<L>
where
    L: Lender,
{
    type Lend = (usize, <L as Lending<'lend>>::Lend);
}
impl<L> Lender for Enumerate<L>
where
    L: Lender,
{
    #[inline]
    fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> {
        self.lender.next().map(|x| {
            let count = self.count;
            self.count += 1;
            (count, x)
        })
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) { self.lender.size_hint() }
    #[inline]
    fn nth(&mut self, n: usize) -> Option<<Self as Lending<'_>>::Lend> {
        let a = self.lender.nth(n)?;
        let i = self.count + n;
        self.count = i + 1;
        Some((i, a))
    }
    #[inline]
    fn count(self) -> usize { self.lender.count() }
    #[inline]
    fn try_fold<B, F, R>(&mut self, init: B, mut f: F) -> R
    where
        Self: Sized,
        F: FnMut(B, <Self as Lending<'_>>::Lend) -> R,
        R: Try<Output = B>,
    {
        let count = &mut self.count;
        self.lender.try_fold(init, |acc, x| {
            let elt = (*count, x);
            *count += 1;
            f(acc, elt)
        })
    }
    #[inline]
    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, <Self as Lending<'_>>::Lend) -> B,
    {
        let mut count = self.count;
        self.lender.fold(init, |acc, x| {
            let elt = (count, x);
            count += 1;
            f(acc, elt)
        })
    }
    #[inline]
    fn advance_by(&mut self, n: usize) -> Result<(), core::num::NonZeroUsize> {
        let remaining = self.lender.advance_by(n);
        let advanced = match remaining {
            Ok(()) => n,
            Err(rem) => n - rem.get(),
        };
        self.count += advanced;
        remaining
    }
}

// TODO DoubleEndedLender Impl
