use core::marker::PhantomData;

use stable_try_trait_v2::Try;

use crate::{
    DoubleEndedFallibleLender, DoubleEndedLender, ExactSizeFallibleLender, ExactSizeLender,
    FallibleLend, FallibleLender, FallibleLending, FusedFallibleLender, FusedLender, ImplBound,
    Lender, Lending, Ref,
};

trait LendingResult<'lend, E, __ImplBound: ImplBound = Ref<'lend, Self>>:
    Lending<'lend, __ImplBound, Lend = Result<Self::Item, E>>
{
    type Item: 'lend;
}

impl<'a, Bound, T, E, I> LendingResult<'a, E, Bound> for I
where
    Bound: ImplBound,
    T: 'a,
    I: Lending<'a, Bound, Lend = Result<T, E>>,
{
    type Item = T;
}

trait LenderResult<E>: Lender + for<'all> LendingResult<'all, E> {}

impl<E, I> LenderResult<E> for I where I: Lender + for<'all> LendingResult<'all, E> {}

/// A fallible lending iterator that wraps a normal lending iterator over [`Result`]s.
///
/// This struct is created by [`Lender::convert`](crate::Lender::convert).
#[derive(Clone, Debug)]
#[repr(transparent)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Convert<E, I> {
    iter: I,
    _marker: PhantomData<E>,
}

impl<E, I> Convert<E, I> {
    pub(crate) fn new(iter: I) -> Self {
        Self {
            iter,
            _marker: PhantomData,
        }
    }

    pub fn into_inner(self) -> I {
        self.iter
    }
}

impl<'lend, E, I> FallibleLending<'lend> for Convert<E, I>
where
    I: LendingResult<'lend, E>,
{
    type Lend = <<I as Lending<'lend>>::Lend as Try>::Output;
}

impl<E, I> FallibleLender for Convert<E, I>
where
    I: LenderResult<E>,
{
    type Error = E;
    // SAFETY: the lend is that of I
    crate::unsafe_assume_covariance_fallible!();

    #[inline]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, E> {
        match self.iter.next() {
            Some(Ok(i)) => Ok(Some(i)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<E, I> DoubleEndedFallibleLender for Convert<E, I>
where
    I: DoubleEndedLender + LenderResult<E>,
{
    #[inline]
    fn next_back(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        match self.iter.next_back() {
            Some(Ok(i)) => Ok(Some(i)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }
}

impl<E, I> ExactSizeFallibleLender for Convert<E, I>
where
    I: ExactSizeLender + LenderResult<E>,
{
    #[inline(always)]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<E, I> FusedFallibleLender for Convert<E, I> where I: FusedLender + LenderResult<E> {}
