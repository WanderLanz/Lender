use core::{convert::Infallible, num::NonZeroUsize};

use crate::{
    DoubleEndedFallibleLender, DoubleEndedLender, ExactSizeFallibleLender, ExactSizeLender,
    FallibleLend, FallibleLender, FallibleLending, FusedFallibleLender, FusedLender, Lender,
    Lending,
};

/// A fallible lender that wraps a normal lender.
///
/// The error type is always [`Infallible`] since the underlying
/// lender cannot fail.
#[derive(Clone, Debug)]
#[repr(transparent)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct IntoFallible<L> {
    pub(crate) lender: L,
}

impl<L> IntoFallible<L> {
    #[inline(always)]
    pub(crate) fn new(lender: L) -> Self {
        Self { lender }
    }

    /// Returns the inner lender.
    #[inline(always)]
    pub fn into_inner(self) -> L {
        self.lender
    }
}

impl<'lend, L> FallibleLending<'lend> for IntoFallible<L>
where
    L: Lending<'lend>,
{
    type Lend = L::Lend;
}

impl<L> FallibleLender for IntoFallible<L>
where
    L: Lender,
{
    type Error = Infallible;
    // SAFETY: the lend is that of L
    crate::unsafe_assume_covariance_fallible!();

    #[inline]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        Ok(self.lender.next())
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.lender.size_hint()
    }

    #[inline]
    fn advance_by(&mut self, n: usize) -> Result<Result<(), NonZeroUsize>, Self::Error> {
        Ok(self.lender.advance_by(n))
    }

    #[inline]
    fn try_fold<B, F, R>(&mut self, init: B, mut f: F) -> Result<R, Self::Error>
    where
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: stable_try_trait_v2::Try<Output = B>,
    {
        // Since Self::Error = Infallible, f can never return Err, so we can
        // safely unwrap and delegate to the inner lender's try_fold
        Ok(self.lender.try_fold(init, |acc, value| {
            // f returns Result<R, Infallible>, which is always Ok
            match f(acc, value) {
                Ok(r) => r,
                Err(e) => match e {},
            }
        }))
    }
}

impl<L: Lender> From<L> for IntoFallible<L> {
    fn from(lender: L) -> Self {
        Self::new(lender)
    }
}

impl<L> DoubleEndedFallibleLender for IntoFallible<L>
where
    L: DoubleEndedLender,
{
    #[inline]
    fn next_back(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        Ok(self.lender.next_back())
    }

    #[inline]
    fn advance_back_by(&mut self, n: usize) -> Result<Result<(), NonZeroUsize>, Self::Error> {
        Ok(self.lender.advance_back_by(n))
    }

    #[inline]
    fn try_rfold<B, F, R>(&mut self, init: B, mut f: F) -> Result<R, Self::Error>
    where
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: stable_try_trait_v2::Try<Output = B>,
    {
        // Since Self::Error = Infallible, f can never return Err
        Ok(self
            .lender
            .try_rfold(init, |acc, value| match f(acc, value) {
                Ok(r) => r,
                Err(e) => match e {},
            }))
    }
}

impl<L> ExactSizeFallibleLender for IntoFallible<L>
where
    L: Lender + ExactSizeLender,
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

impl<L> FusedFallibleLender for IntoFallible<L> where L: FusedLender {}
