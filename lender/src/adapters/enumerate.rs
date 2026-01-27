use core::num::NonZeroUsize;

use crate::{
    try_trait_v2::Try, DoubleEndedFallibleLender, DoubleEndedLender, ExactSizeFallibleLender, ExactSizeLender, FallibleLend,
    FallibleLender, FallibleLending, FusedFallibleLender, FusedLender, Lend, Lender, Lending,
};
#[derive(Clone, Debug)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Enumerate<L> {
    lender: L,
    count: usize,
}
impl<L> Enumerate<L> {
    pub(crate) fn new(lender: L) -> Enumerate<L> {
        Enumerate { lender, count: 0 }
    }
    pub fn into_inner(self) -> L {
        self.lender
    }
}
impl<'lend, L> Lending<'lend> for Enumerate<L>
where
    L: Lender,
{
    type Lend = (usize, Lend<'lend, L>);
}
impl<L> Lender for Enumerate<L>
where
    L: Lender,
{
    crate::unsafe_assume_covariance!();
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        self.lender.next().map(|x| {
            let count = self.count;
            self.count += 1;
            (count, x)
        })
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.lender.size_hint()
    }
    #[inline]
    fn nth(&mut self, n: usize) -> Option<Lend<'_, Self>> {
        let a = self.lender.nth(n)?;
        let i = self.count + n;
        self.count = i + 1;
        Some((i, a))
    }
    #[inline]
    fn count(self) -> usize {
        self.lender.count()
    }
    #[inline]
    fn try_fold<B, F, R>(&mut self, init: B, mut f: F) -> R
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> R,
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
        F: FnMut(B, Lend<'_, Self>) -> B,
    {
        let mut count = self.count;
        self.lender.fold(init, |acc, x| {
            let elt = (count, x);
            count += 1;
            f(acc, elt)
        })
    }
    #[inline]
    fn advance_by(&mut self, n: usize) -> Result<(), NonZeroUsize> {
        let remaining = self.lender.advance_by(n);
        let advanced = match remaining {
            Ok(()) => n,
            Err(rem) => n - rem.get(),
        };
        self.count += advanced;
        remaining
    }
}
impl<L> DoubleEndedLender for Enumerate<L>
where
    L: ExactSizeLender + DoubleEndedLender,
{
    #[inline]
    fn next_back(&mut self) -> Option<Lend<'_, Self>> {
        let len = self.lender.len();
        let x = self.lender.next_back()?;
        Some((self.count + len - 1, x))
    }
    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Lend<'_, Self>> {
        let len = self.lender.len();
        let x = self.lender.nth_back(n)?;
        Some((self.count + len - n - 1, x))
    }
    #[inline]
    fn try_rfold<B, F, R>(&mut self, init: B, mut f: F) -> R
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> R,
        R: Try<Output = B>,
    {
        let mut count = self.count + self.lender.len();
        self.lender.try_rfold(init, move |acc, x| {
            count -= 1;
            f(acc, (count, x))
        })
    }
    #[inline]
    fn rfold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> B,
    {
        let mut count = self.count + self.lender.len();
        self.lender.rfold(init, move |acc, x| {
            count -= 1;
            f(acc, (count, x))
        })
    }
    #[inline]
    fn advance_back_by(&mut self, n: usize) -> Result<(), NonZeroUsize> {
        self.lender.advance_back_by(n)
    }
}
impl<L> ExactSizeLender for Enumerate<L>
where
    L: ExactSizeLender,
{
    fn len(&self) -> usize {
        self.lender.len()
    }
    fn is_empty(&self) -> bool {
        self.lender.is_empty()
    }
}
impl<L> FusedLender for Enumerate<L> where L: FusedLender {}
impl<L: Default> Default for Enumerate<L> {
    fn default() -> Self {
        Enumerate::new(Default::default())
    }
}

impl<'lend, L> FallibleLending<'lend> for Enumerate<L>
where
    L: FallibleLender,
{
    type Lend = (usize, FallibleLend<'lend, L>);
}
impl<L> FallibleLender for Enumerate<L>
where
    L: FallibleLender,
{
    type Error = L::Error;
    crate::unsafe_assume_covariance_fallible!();

    #[inline]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        Ok(self.lender.next()?.map(|x| {
            let count = self.count;
            self.count += 1;
            (count, x)
        }))
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.lender.size_hint()
    }
    #[inline]
    fn nth(&mut self, n: usize) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        let a = match self.lender.nth(n)? {
            Some(a) => a,
            None => return Ok(None),
        };
        let i = self.count + n;
        self.count = i + 1;
        Ok(Some((i, a)))
    }
    #[inline]
    fn count(self) -> Result<usize, Self::Error> {
        self.lender.count()
    }
    #[inline]
    fn try_fold<B, F, R>(&mut self, init: B, mut f: F) -> Result<R, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
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
    fn fold<B, F>(self, init: B, mut f: F) -> Result<B, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<B, Self::Error>,
    {
        let mut count = self.count;
        self.lender.fold(init, |acc, x| {
            let elt = (count, x);
            count += 1;
            f(acc, elt)
        })
    }
    #[inline]
    fn advance_by(&mut self, n: usize) -> Result<Option<NonZeroUsize>, Self::Error> {
        let remaining = self.lender.advance_by(n)?;
        let advanced = match remaining {
            None => n,
            Some(rem) => n - rem.get(),
        };
        self.count += advanced;
        Ok(remaining)
    }
}
impl<L> DoubleEndedFallibleLender for Enumerate<L>
where
    L: ExactSizeFallibleLender + DoubleEndedFallibleLender,
{
    #[inline]
    fn next_back(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        let len = self.lender.len();
        let x = match self.lender.next_back()? {
            Some(x) => x,
            None => return Ok(None),
        };
        Ok(Some((self.count + len - 1, x)))
    }
    #[inline]
    fn nth_back(&mut self, n: usize) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        let len = self.lender.len();
        let x = match self.lender.nth_back(n)? {
            Some(x) => x,
            None => return Ok(None),
        };
        Ok(Some((self.count + len - n - 1, x)))
    }
    #[inline]
    fn try_rfold<B, F, R>(&mut self, init: B, mut f: F) -> Result<R, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: Try<Output = B>,
    {
        let mut count = self.count + self.lender.len();
        self.lender.try_rfold(init, move |acc, x| {
            count -= 1;
            f(acc, (count, x))
        })
    }
    #[inline]
    fn rfold<B, F>(self, init: B, mut f: F) -> Result<B, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<B, Self::Error>,
    {
        let mut count = self.count + self.lender.len();
        self.lender.rfold(init, move |acc, x| {
            count -= 1;
            f(acc, (count, x))
        })
    }
    #[inline]
    fn advance_back_by(&mut self, n: usize) -> Result<Option<core::num::NonZeroUsize>, Self::Error> {
        self.lender.advance_back_by(n)
    }
}
impl<L> ExactSizeFallibleLender for Enumerate<L>
where
    L: ExactSizeFallibleLender,
{
    fn len(&self) -> usize {
        self.lender.len()
    }
    fn is_empty(&self) -> bool {
        self.lender.is_empty()
    }
}
impl<L> FusedFallibleLender for Enumerate<L> where L: FusedFallibleLender {}
