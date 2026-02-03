use core::{fmt, marker::PhantomData, num::NonZeroUsize};

use crate::{
    DoubleEndedFallibleLender, ExactSizeFallibleLender, FallibleLend, FallibleLender,
    FallibleLending, FusedFallibleLender,
};

/// A fallible lender that maps the errors of the underlying lender with a closure.
///
/// This `struct` is created by the [`map_err()`](crate::FallibleLender::map_err) method on
/// [`FallibleLender`]. See its documentation for more.
#[derive(Clone)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct MapErr<E, L, F> {
    pub(crate) lender: L,
    f: F,
    _marker: PhantomData<E>,
}

impl<E, L, F> MapErr<E, L, F> {
    #[inline(always)]
    pub(crate) fn new(lender: L, f: F) -> Self {
        Self {
            lender,
            f,
            _marker: PhantomData,
        }
    }

    /// Returns the inner lender.
    #[inline(always)]
    pub fn into_inner(self) -> L {
        self.lender
    }

    /// Returns the inner lender and the error-mapping function.
    #[inline(always)]
    pub fn into_parts(self) -> (L, F) {
        (self.lender, self.f)
    }
}

impl<E, L: fmt::Debug, F> fmt::Debug for MapErr<E, L, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MapErr")
            .field("lender", &self.lender)
            .finish()
    }
}

impl<'lend, E, L, F> FallibleLending<'lend> for MapErr<E, L, F>
where
    L: FallibleLender,
{
    type Lend = FallibleLend<'lend, L>;
}

impl<E, L, F> FallibleLender for MapErr<E, L, F>
where
    F: FnMut(L::Error) -> E,
    L: FallibleLender,
{
    type Error = E;
    // SAFETY: the lend is that of L
    crate::unsafe_assume_covariance_fallible!();

    #[inline(always)]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        self.lender.next().map_err(&mut self.f)
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.lender.size_hint()
    }

    #[inline]
    fn count(mut self) -> Result<usize, Self::Error>
    where
        Self: Sized,
    {
        self.lender.count().map_err(&mut self.f)
    }

    #[inline]
    fn advance_by(&mut self, n: usize) -> Result<Result<(), NonZeroUsize>, Self::Error> {
        self.lender.advance_by(n).map_err(&mut self.f)
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        self.lender.nth(n).map_err(&mut self.f)
    }

    #[inline]
    fn last<'call>(&'call mut self) -> Result<Option<FallibleLend<'call, Self>>, Self::Error>
    where
        Self: Sized,
    {
        self.lender.last().map_err(&mut self.f)
    }
}

impl<E, L: DoubleEndedFallibleLender, F> DoubleEndedFallibleLender for MapErr<E, L, F>
where
    F: FnMut(L::Error) -> E,
{
    #[inline(always)]
    fn next_back(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        self.lender.next_back().map_err(&mut self.f)
    }

    #[inline]
    fn advance_back_by(&mut self, n: usize) -> Result<Result<(), NonZeroUsize>, Self::Error> {
        self.lender.advance_back_by(n).map_err(&mut self.f)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        self.lender.nth_back(n).map_err(&mut self.f)
    }
}

impl<E, L, F> FusedFallibleLender for MapErr<E, L, F>
where
    F: FnMut(L::Error) -> E,
    L: FusedFallibleLender,
{
}

impl<E, L, F> ExactSizeFallibleLender for MapErr<E, L, F>
where
    F: FnMut(L::Error) -> E,
    L: ExactSizeFallibleLender,
{
    #[inline(always)]
    fn len(&self) -> usize {
        self.lender.len()
    }
}
