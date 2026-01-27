use core::ops::ControlFlow;

use crate::{
    try_trait_v2::{FromResidual, Try},
    DoubleEndedFallibleLender, DoubleEndedLender, ExactSizeFallibleLender, ExactSizeLender, FallibleLend, FallibleLender,
    FallibleLending, FusedFallibleLender, FusedLender, Lend, Lender, Lending,
};
#[derive(Clone, Debug)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Fuse<L> {
    lender: L,
    flag: bool,
}
impl<L> Fuse<L> {
    pub(crate) fn new(lender: L) -> Fuse<L> {
        Fuse { lender, flag: false }
    }
    pub fn into_inner(self) -> L {
        self.lender
    }
}
impl<L> FusedLender for Fuse<L> where L: Lender {}
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
    crate::inherit_covariance!(L);
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
    fn fold<B, F>(mut self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> B,
    {
        let mut acc = init;
        if !self.flag {
            acc = self.lender.fold(acc, &mut f);
            self.flag = true;
        }
        acc
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
    fn rfold<B, F>(mut self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> B,
    {
        let mut acc = init;
        if !self.flag {
            acc = self.lender.rfold(acc, &mut f);
            self.flag = true;
        }
        acc
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
    fn default() -> Self {
        Self::new(L::default())
    }
}

impl<'lend, L> FallibleLending<'lend> for Fuse<L>
where
    L: FallibleLender,
{
    type Lend = FallibleLend<'lend, L>;
}
impl<L> FallibleLender for Fuse<L>
where
    L: FallibleLender,
{
    type Error = L::Error;
    crate::inherit_covariance_fallible!(L);

    #[inline]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        if self.flag {
            Ok(None)
        } else {
            match self.lender.next() {
                Ok(Some(next)) => Ok(Some(next)),
                res @ (Ok(None) | Err(_)) => {
                    self.flag = true;
                    res
                }
            }
        }
    }
    #[inline]
    fn nth(&mut self, n: usize) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        if self.flag {
            Ok(None)
        } else {
            match self.lender.nth(n) {
                Ok(Some(value)) => Ok(Some(value)),
                res @ (Ok(None) | Err(_)) => {
                    self.flag = true;
                    res
                }
            }
        }
    }
    #[inline]
    fn last(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error>
    where
        Self: Sized,
    {
        if self.flag {
            Ok(None)
        } else {
            match self.lender.last() {
                Ok(Some(value)) => Ok(Some(value)),
                res @ (Ok(None) | Err(_)) => {
                    self.flag = true;
                    res
                }
            }
        }
    }
    #[inline]
    fn count(self) -> Result<usize, Self::Error> {
        if !self.flag {
            self.lender.count()
        } else {
            Ok(0)
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
    fn try_fold<Acc, F, R>(&mut self, mut acc: Acc, mut f: F) -> Result<R, Self::Error>
    where
        F: FnMut(Acc, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: Try<Output = Acc>,
    {
        if !self.flag {
            acc = match self.lender.try_fold(acc, &mut f).inspect_err(|_| self.flag = true)?.branch() {
                ControlFlow::Continue(x) => x,
                ControlFlow::Break(x) => return Ok(FromResidual::from_residual(x)),
            };
            self.flag = true;
        }
        Ok(Try::from_output(acc))
    }
    #[inline]
    fn fold<B, F>(mut self, init: B, mut f: F) -> Result<B, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<B, Self::Error>,
    {
        let mut acc = init;
        if !self.flag {
            acc = self.lender.fold(acc, &mut f).inspect_err(|_| self.flag = true)?;
            self.flag = true;
        }
        Ok(acc)
    }
    #[inline]
    fn find<P>(&mut self, mut predicate: P) -> Result<Option<FallibleLend<'_, Self>>, Self::Error>
    where
        P: FnMut(&FallibleLend<'_, Self>) -> Result<bool, Self::Error>,
    {
        if !self.flag {
            if let x @ Some(_) = self.lender.find(&mut predicate).inspect_err(|_| self.flag = true)? {
                return Ok(x);
            }
            self.flag = true;
        }
        Ok(None)
    }
}
impl<L> DoubleEndedFallibleLender for Fuse<L>
where
    L: DoubleEndedFallibleLender,
{
    #[inline]
    fn next_back(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        if !self.flag {
            if let x @ Some(_) = self.lender.next_back().inspect_err(|_| self.flag = true)? {
                return Ok(x);
            }
            self.flag = true;
        }
        Ok(None)
    }
    #[inline]
    fn nth_back(&mut self, n: usize) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        if !self.flag {
            if let x @ Some(_) = self.lender.nth_back(n).inspect_err(|_| self.flag = true)? {
                return Ok(x);
            }
            self.flag = true;
        }
        Ok(None)
    }
    #[inline]
    fn try_rfold<Acc, F, R>(&mut self, mut acc: Acc, mut f: F) -> Result<R, Self::Error>
    where
        Self: Sized,
        F: FnMut(Acc, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: Try<Output = Acc>,
    {
        if !self.flag {
            acc = match self.lender.try_rfold(acc, &mut f).inspect_err(|_| self.flag = true)?.branch() {
                ControlFlow::Continue(x) => x,
                ControlFlow::Break(x) => return Ok(FromResidual::from_residual(x)),
            };
            self.flag = true;
        }
        Ok(Try::from_output(acc))
    }
    #[inline]
    fn rfold<B, F>(mut self, init: B, mut f: F) -> Result<B, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<B, Self::Error>,
    {
        let mut acc = init;
        if !self.flag {
            acc = self.lender.rfold(acc, &mut f).inspect_err(|_| self.flag = true)?;
            self.flag = true;
        }
        Ok(acc)
    }
    #[inline]
    fn rfind<P>(&mut self, mut predicate: P) -> Result<Option<FallibleLend<'_, Self>>, Self::Error>
    where
        Self: Sized,
        P: FnMut(&FallibleLend<'_, Self>) -> Result<bool, Self::Error>,
    {
        if !self.flag {
            if let x @ Some(_) = self.lender.rfind(&mut predicate).inspect_err(|_| self.flag = true)? {
                return Ok(x);
            }
            self.flag = true;
        }
        Ok(None)
    }
}
impl<L> ExactSizeFallibleLender for Fuse<L>
where
    L: ExactSizeFallibleLender,
{
    fn len(&self) -> usize {
        if self.flag {
            0
        } else {
            self.lender.len()
        }
    }
    fn is_empty(&self) -> bool {
        self.flag || self.lender.is_empty()
    }
}
impl<L> FusedFallibleLender for Fuse<L> where L: FallibleLender {}
