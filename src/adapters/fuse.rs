use core::ops::ControlFlow;

use crate::{
    try_trait_v2::{FromResidual, Try},
    DoubleEndedLender, ExactSizeLender, FusedLender, Lender, Lending,
};
#[derive(Clone, Debug)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Fuse<L> {
    lender: L,
    flag: bool,
}
impl<L> Fuse<L> {
    pub(crate) fn new(lender: L) -> Fuse<L> { Fuse { lender, flag: false } }
}
impl<L> FusedLender for Fuse<L> where L: Lender {}
impl<'lend, L> Lending<'lend> for Fuse<L>
where
    L: Lender,
{
    type Lend = <L as Lending<'lend>>::Lend;
}
impl<L> Lender for Fuse<L>
where
    L: Lender,
{
    #[inline]
    fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> {
        if !self.flag {
            if let x @ Some(_) = self.lender.next() {
                return x;
            }
            self.flag = true;
        }
        None
    }
    #[inline]
    fn nth(&mut self, n: usize) -> Option<<Self as Lending<'_>>::Lend> {
        if !self.flag {
            if let x @ Some(_) = self.lender.nth(n) {
                return x;
            }
            self.flag = true;
        }
        None
    }
    #[inline]
    fn last<'call>(mut self) -> Option<<Self as Lending<'call>>::Lend>
    where
        Self: Sized + 'call,
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
        if !self.flag {
            self.lender.count()
        } else {
            0
        }
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
        F: FnMut(Acc, <Self as Lending<'_>>::Lend) -> R,
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
    fn fold<B, F>(mut self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, <Self as Lending<'_>>::Lend) -> B,
    {
        let mut acc = init;
        if !self.flag {
            acc = self.lender.fold(acc, &mut f);
            self.flag = true;
        }
        acc
    }
    #[inline]
    fn find<P>(&mut self, mut predicate: P) -> Option<<Self as Lending<'_>>::Lend>
    where
        Self: Sized,
        P: FnMut(&<Self as Lending<'_>>::Lend) -> bool,
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
    fn next_back(&mut self) -> Option<<Self as Lending<'_>>::Lend> {
        if !self.flag {
            if let x @ Some(_) = self.lender.next_back() {
                return x;
            }
            self.flag = true;
        }
        None
    }
    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<<Self as Lending<'_>>::Lend> {
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
        F: FnMut(Acc, <Self as Lending<'_>>::Lend) -> R,
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
    fn rfold<B, F>(mut self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, <Self as Lending<'_>>::Lend) -> B,
    {
        let mut acc = init;
        if !self.flag {
            acc = self.lender.rfold(acc, &mut f);
            self.flag = true;
        }
        acc
    }
    #[inline]
    fn rfind<P>(&mut self, mut predicate: P) -> Option<<Self as Lending<'_>>::Lend>
    where
        Self: Sized,
        P: FnMut(&<Self as Lending<'_>>::Lend) -> bool,
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
    fn len(&self) -> usize {
        if !self.flag {
            self.lender.len()
        } else {
            0
        }
    }
    fn is_empty(&self) -> bool {
        if !self.flag {
            self.lender.is_empty()
        } else {
            true
        }
    }
}
impl<L: Default> Default for Fuse<L> {
    fn default() -> Self { Self::new(L::default()) }
}
