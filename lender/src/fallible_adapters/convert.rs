use core::marker::PhantomData;

use stable_try_trait_v2::Try;

use crate::{
    DoubleEndedFallibleLender, DoubleEndedLender, ExactSizeFallibleLender, ExactSizeLender, FallibleLend, FallibleLender,
    FallibleLending, FusedFallibleLender, FusedLender, ImplBound, Lender, Lending, Ref,
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
#[repr(transparent)]
pub struct Convert<E, I> {
    iter: I,
    _marker: PhantomData<E>,
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
    crate::unsafe_assume_covariance_fallible!();

    #[inline]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, E> {
        match self.iter.next() {
            Some(Ok(i)) => Ok(Some(i)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }

    #[inline]
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
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<E, I> FusedFallibleLender for Convert<E, I> where I: FusedLender + LenderResult<E> {}
