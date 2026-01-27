use core::{marker::PhantomData, num::NonZeroUsize, ops::ControlFlow};

use stable_try_trait_v2::{FromResidual, Try};

use crate::{
    DoubleEndedFallibleLender, DoubleEndedLender, ExactSizeFallibleLender, ExactSizeLender, FallibleLend,
    FallibleLender, FallibleLending, FusedFallibleLender, FusedLender, Lender, Lending,
};

/// Wrapper for Try types wrapped in outer Result
#[repr(transparent)]
struct ResTry<T, E>(Result<T, E>);

impl<T, E> FromResidual<Result<T::Residual, E>> for ResTry<T, E>
where
    T: Try,
{
    fn from_residual(residual: Result<T::Residual, E>) -> Self {
        Self(residual.map(T::from_residual))
    }
}

impl<T, E> Try for ResTry<T, E>
where
    T: Try,
{
    type Output = T::Output;
    type Residual = Result<T::Residual, E>;
    fn from_output(output: Self::Output) -> Self {
        Self(Ok(T::from_output(output)))
    }
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self.0 {
            Ok(output) => output.branch().map_break(Ok),
            Err(err) => ControlFlow::Break(Err(err)),
        }
    }
}

/// A fallible lender that wraps a normal lender.
#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct IntoFallible<E, L> {
    pub(crate) lender: L,
    _marker: PhantomData<E>,
}

impl<E, L> IntoFallible<E, L> {
    pub(crate) fn new(lender: L) -> Self {
        Self { lender, _marker: PhantomData }
    }

    pub fn into_inner(self) -> L {
        self.lender
    }
}

impl<'lend, E, L> FallibleLending<'lend> for IntoFallible<E, L>
where
    L: Lending<'lend>,
{
    type Lend = L::Lend;
}
impl<E, L> FallibleLender for IntoFallible<E, L>
where
    L: Lender,
{
    type Error = E;
    // SAFETY: The underlying Lender L's covariance has been verified by its own
    // _check_covariance implementation. This adapter wraps the Lend type unchanged.
    unsafe fn _check_covariance<'long: 'short, 'short>(
        lend: *const &'short <Self as FallibleLending<'long>>::Lend,
        _: crate::Uncallable,
    ) -> *const &'short <Self as FallibleLending<'short>>::Lend {
        unsafe { core::mem::transmute(lend) }
    }

    #[inline]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        Ok(self.lender.next())
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.lender.size_hint()
    }

    #[inline]
    fn try_fold<B, F, R>(&mut self, init: B, mut f: F) -> Result<R, Self::Error>
    where
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: stable_try_trait_v2::Try<Output = B>,
    {
        self.lender.try_fold(init, |acc, value| ResTry(f(acc, value))).0
    }
}
impl<E, L: Lender> From<L> for IntoFallible<E, L> {
    fn from(lender: L) -> Self {
        Self::new(lender)
    }
}

impl<E, L> DoubleEndedFallibleLender for IntoFallible<E, L>
where
    L: DoubleEndedLender,
{
    #[inline]
    fn next_back(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        Ok(self.lender.next_back())
    }

    #[inline]
    fn advance_back_by(&mut self, n: usize) -> Result<Option<NonZeroUsize>, Self::Error> {
        Ok(self.lender.advance_back_by(n).err())
    }

    #[inline]
    fn try_rfold<B, F, R>(&mut self, init: B, mut f: F) -> Result<R, Self::Error>
    where
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: stable_try_trait_v2::Try<Output = B>,
    {
        self.lender.try_rfold(init, |acc, value| ResTry(f(acc, value))).0
    }
}

impl<E, L> ExactSizeFallibleLender for IntoFallible<E, L>
where
    L: Lender + ExactSizeLender,
{
    #[inline]
    fn len(&self) -> usize {
        self.lender.len()
    }
}

impl<E, L> FusedFallibleLender for IntoFallible<E, L> where L: FusedLender {}
