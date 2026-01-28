use core::fmt;

use crate::{
    DoubleEndedFallibleLender, DoubleEndedLender, ExactSizeFallibleLender, ExactSizeLender,
    FallibleLend, FallibleLender, FallibleLending, FusedFallibleLender, FusedLender, Lend, Lender,
    Lending, try_trait_v2::Try,
};

#[derive(Clone)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Mutate<L, F> {
    lender: L,
    f: F,
}

impl<L, F> Mutate<L, F> {
    pub(crate) fn new(lender: L, f: F) -> Mutate<L, F> {
        Mutate { lender, f }
    }

    pub fn into_inner(self) -> L {
        self.lender
    }

    pub fn into_parts(self) -> (L, F) {
        (self.lender, self.f)
    }
}

impl<L: fmt::Debug, F> fmt::Debug for Mutate<L, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Mutate")
            .field("lender", &self.lender)
            .finish()
    }
}

impl<'lend, L, F> Lending<'lend> for Mutate<L, F>
where
    L: Lender,
    F: FnMut(&mut Lend<'lend, L>),
{
    type Lend = Lend<'lend, L>;
}

impl<L, F> Lender for Mutate<L, F>
where
    L: Lender,
    F: FnMut(&mut Lend<'_, L>),
{
    // SAFETY: the lend is that of L
    crate::unsafe_assume_covariance!();
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        let mut next = self.lender.next();
        if let Some(ref mut x) = next {
            (self.f)(x);
        }
        next
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.lender.size_hint()
    }

    #[inline]
    fn try_fold<B, Fold, R>(&mut self, init: B, mut fold: Fold) -> R
    where
        Self: Sized,
        Fold: FnMut(B, Lend<'_, Self>) -> R,
        R: Try<Output = B>,
    {
        let f = &mut self.f;
        self.lender.try_fold(init, move |acc, mut x| {
            (f)(&mut x);
            fold(acc, x)
        })
    }

    #[inline]
    fn fold<B, Fold>(mut self, init: B, mut fold: Fold) -> B
    where
        Self: Sized,
        Fold: FnMut(B, Lend<'_, Self>) -> B,
    {
        self.lender.fold(init, move |acc, mut x| {
            (self.f)(&mut x);
            fold(acc, x)
        })
    }
}

impl<L, F> DoubleEndedLender for Mutate<L, F>
where
    L: DoubleEndedLender,
    F: FnMut(&mut Lend<'_, L>),
{
    #[inline]
    fn next_back(&mut self) -> Option<Lend<'_, Self>> {
        let mut next = self.lender.next_back();
        if let Some(ref mut x) = next {
            (self.f)(x);
        }
        next
    }

    #[inline]
    fn try_rfold<B, Fold, R>(&mut self, init: B, mut fold: Fold) -> R
    where
        Self: Sized,
        Fold: FnMut(B, Lend<'_, Self>) -> R,
        R: Try<Output = B>,
    {
        let f = &mut self.f;
        self.lender.try_rfold(init, move |acc, mut x| {
            (f)(&mut x);
            fold(acc, x)
        })
    }

    #[inline]
    fn rfold<B, Fold>(mut self, init: B, mut fold: Fold) -> B
    where
        Self: Sized,
        Fold: FnMut(B, Lend<'_, Self>) -> B,
    {
        self.lender.rfold(init, move |acc, mut x| {
            (self.f)(&mut x);
            fold(acc, x)
        })
    }
}

impl<L: ExactSizeLender, F> ExactSizeLender for Mutate<L, F>
where
    F: FnMut(&mut Lend<'_, L>),
{
    #[inline(always)]
    fn len(&self) -> usize {
        self.lender.len()
    }

    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.lender.is_empty()
    }
}

impl<L: FusedLender, F> FusedLender for Mutate<L, F> where F: FnMut(&mut Lend<'_, L>) {}

impl<'lend, L, F> FallibleLending<'lend> for Mutate<L, F>
where
    L: FallibleLender,
    F: FnMut(&mut FallibleLend<'lend, L>) -> Result<(), L::Error>,
{
    type Lend = FallibleLend<'lend, L>;
}

impl<L, F> FallibleLender for Mutate<L, F>
where
    L: FallibleLender,
    F: FnMut(&mut FallibleLend<'_, L>) -> Result<(), L::Error>,
{
    type Error = L::Error;
    // SAFETY: the lend is that of L
    crate::unsafe_assume_covariance_fallible!();

    #[inline]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        let mut next = self.lender.next()?;
        if let Some(ref mut x) = next {
            (self.f)(x)?;
        }
        Ok(next)
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.lender.size_hint()
    }

    #[inline]
    fn try_fold<B, Fold, R>(&mut self, init: B, mut fold: Fold) -> Result<R, Self::Error>
    where
        Self: Sized,
        Fold: FnMut(B, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: Try<Output = B>,
    {
        let f = &mut self.f;
        self.lender.try_fold(init, move |acc, mut x| {
            (f)(&mut x)?;
            fold(acc, x)
        })
    }

    #[inline]
    fn fold<B, Fold>(mut self, init: B, mut fold: Fold) -> Result<B, Self::Error>
    where
        Self: Sized,
        Fold: FnMut(B, FallibleLend<'_, Self>) -> Result<B, Self::Error>,
    {
        self.lender.fold(init, move |acc, mut x| {
            (self.f)(&mut x)?;
            fold(acc, x)
        })
    }
}

impl<L, F> DoubleEndedFallibleLender for Mutate<L, F>
where
    L: DoubleEndedFallibleLender,
    F: FnMut(&mut FallibleLend<'_, L>) -> Result<(), L::Error>,
{
    #[inline]
    fn next_back(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        let mut next = self.lender.next_back()?;
        if let Some(ref mut x) = next {
            (self.f)(x)?;
        }
        Ok(next)
    }

    #[inline]
    fn try_rfold<B, Fold, R>(&mut self, init: B, mut fold: Fold) -> Result<R, Self::Error>
    where
        Self: Sized,
        Fold: FnMut(B, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: Try<Output = B>,
    {
        let f = &mut self.f;
        self.lender.try_rfold(init, move |acc, mut x| {
            (f)(&mut x)?;
            fold(acc, x)
        })
    }

    #[inline]
    fn rfold<B, Fold>(mut self, init: B, mut fold: Fold) -> Result<B, Self::Error>
    where
        Self: Sized,
        Fold: FnMut(B, FallibleLend<'_, Self>) -> Result<B, Self::Error>,
    {
        self.lender.rfold(init, move |acc, mut x| {
            (self.f)(&mut x)?;
            fold(acc, x)
        })
    }
}

impl<L: ExactSizeFallibleLender, F> ExactSizeFallibleLender for Mutate<L, F>
where
    F: FnMut(&mut FallibleLend<'_, L>) -> Result<(), L::Error>,
{
    #[inline(always)]
    fn len(&self) -> usize {
        self.lender.len()
    }

    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.lender.is_empty()
    }
}

impl<L: FusedFallibleLender, F> FusedFallibleLender for Mutate<L, F> where
    F: FnMut(&mut FallibleLend<'_, L>) -> Result<(), L::Error>
{
}
