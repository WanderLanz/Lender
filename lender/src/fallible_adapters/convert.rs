use core::{marker::PhantomData, ops::ControlFlow};

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

/// A fallible lending iterator that wraps a normal lending
/// iterator over [`Result`]s.
///
/// This struct is created by [`Lender::convert`].
#[derive(Clone, Debug)]
#[repr(transparent)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Convert<E, I> {
    iter: I,
    _marker: PhantomData<fn() -> E>,
}

impl<E, I: crate::Lender> Convert<E, I> {
    #[inline]
    pub(crate) fn new(iter: I) -> Self {
        crate::__check_lender_covariance::<I>();
        Self {
            iter,
            _marker: PhantomData,
        }
    }

    /// Returns the inner lender.
    #[inline]
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

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    #[inline]
    fn try_fold<B, F, R>(&mut self, init: B, mut f: F) -> Result<R, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: Try<Output = B>,
    {
        match self.iter.try_fold(init, |acc, item| match item {
            Ok(v) => super::try_fold_with(f(acc, v)),
            Err(e) => ControlFlow::Break(Err(e)),
        }) {
            ControlFlow::Continue(acc) => Ok(R::from_output(acc)),
            ControlFlow::Break(r) => r,
        }
    }

    #[inline]
    fn fold<B, F>(mut self, init: B, mut f: F) -> Result<B, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<B, Self::Error>,
    {
        // Delegate to the inner lender's try_fold using
        // ControlFlow to propagate item errors.
        match self.iter.try_fold(init, |acc, item| match item {
            Ok(v) => match f(acc, v) {
                Ok(b) => ControlFlow::Continue(b),
                Err(e) => ControlFlow::Break(e),
            },
            Err(e) => ControlFlow::Break(e),
        }) {
            ControlFlow::Continue(acc) => Ok(acc),
            ControlFlow::Break(e) => Err(e),
        }
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

    #[inline]
    fn try_rfold<B, F, R>(&mut self, init: B, mut f: F) -> Result<R, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: Try<Output = B>,
    {
        match self.iter.try_rfold(init, |acc, item| match item {
            Ok(v) => super::try_fold_with(f(acc, v)),
            Err(e) => ControlFlow::Break(Err(e)),
        }) {
            ControlFlow::Continue(acc) => Ok(R::from_output(acc)),
            ControlFlow::Break(r) => r,
        }
    }

    #[inline]
    fn rfold<B, F>(mut self, init: B, mut f: F) -> Result<B, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<B, Self::Error>,
    {
        match self.iter.try_rfold(init, |acc, item| match item {
            Ok(v) => match f(acc, v) {
                Ok(b) => ControlFlow::Continue(b),
                Err(e) => ControlFlow::Break(e),
            },
            Err(e) => ControlFlow::Break(e),
        }) {
            ControlFlow::Continue(acc) => Ok(acc),
            ControlFlow::Break(e) => Err(e),
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

    #[inline]
    fn is_empty(&self) -> bool {
        self.iter.is_empty()
    }
}

impl<E, I> FusedFallibleLender for Convert<E, I> where I: FusedLender + LenderResult<E> {}
