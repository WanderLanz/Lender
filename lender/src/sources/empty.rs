use core::{fmt, marker};

use crate::{
    DoubleEndedFallibleLender, DoubleEndedLender, ExactSizeLender, FallibleLend, FallibleLender, FallibleLending,
    FusedFallibleLender, FusedLender, Lend, Lender, Lending,
};

/// Creates a lender that yields nothing.
///
/// similar to [`core::iter::empty()`]
///
/// # Examples
/// ```rust
/// use lender::prelude::*;
/// let mut e = lender::empty::<lend!(&'lend mut u32)>();
/// let x: Option<&'_ mut u32> = e.next();
/// assert_eq!(x, None);
/// ```
pub const fn empty<L: ?Sized + for<'all> Lending<'all>>() -> Empty<L> { Empty(marker::PhantomData) }

/// A lender that yields nothing.
///
/// This `struct` is created by the [`empty()`] function.
///
/// similar to [`core::iter::Empty`].
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Empty<L: ?Sized>(marker::PhantomData<L>);

impl<L: ?Sized> fmt::Debug for Empty<L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.debug_struct("Empty").finish() }
}

impl<'lend, L> Lending<'lend> for Empty<L>
where
    L: ?Sized + for<'all> Lending<'all>,
{
    type Lend = Lend<'lend, L>;
}
impl<L> Lender for Empty<L>
where
    L: ?Sized + for<'all> Lending<'all>,
{
    fn next(&mut self) -> Option<Lend<'_, Self>> { None }
    fn size_hint(&self) -> (usize, Option<usize>) { (0, Some(0)) }
}

impl<L> DoubleEndedLender for Empty<L>
where
    L: ?Sized + for<'all> Lending<'all>,
{
    fn next_back(&mut self) -> Option<Lend<'_, Self>> { None }
}

impl<L> ExactSizeLender for Empty<L>
where
    L: ?Sized + for<'all> Lending<'all>,
{
    fn len(&self) -> usize { 0 }
}

impl<L> FusedLender for Empty<L> where L: ?Sized + for<'all> Lending<'all> {}

impl<L: ?Sized> Clone for Empty<L> {
    fn clone(&self) -> Empty<L> { Empty(marker::PhantomData) }
}

impl<L: ?Sized> Default for Empty<L> {
    fn default() -> Empty<L> { Empty(marker::PhantomData) }
}

/// Creates a fallible lender that yields nothing.
///
/// similar to [`core::iter::empty()`]
pub const fn empty_fallible<E, L: ?Sized + for<'all> FallibleLending<'all>>() -> EmptyFallible<E, L> {
    EmptyFallible(marker::PhantomData)
}

/// A lender that yields nothing.
///
/// This `struct` is created by the [`empty()`] function.
///
/// similar to [`core::iter::Empty`].
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct EmptyFallible<E, L: ?Sized>(marker::PhantomData<(E, L)>);

impl<E, L: ?Sized> fmt::Debug for EmptyFallible<E, L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.debug_struct("EmptyFallible").finish() }
}

impl<'lend, E, L> FallibleLending<'lend> for EmptyFallible<E, L>
where
    L: ?Sized + for<'all> FallibleLending<'all>,
{
    type Lend = FallibleLend<'lend, L>;
}
impl<E, L> FallibleLender for EmptyFallible<E, L>
where
    L: ?Sized + for<'all> FallibleLending<'all>,
{
    type Error = E;

    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> { Ok(None) }
    fn size_hint(&self) -> (usize, Option<usize>) { (0, Some(0)) }
}

impl<E, L> DoubleEndedFallibleLender for EmptyFallible<E, L>
where
    L: ?Sized + for<'all> FallibleLending<'all>,
{
    fn next_back(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> { Ok(None) }
}

impl<E, L> FusedFallibleLender for EmptyFallible<E, L> where L: ?Sized + for<'all> FallibleLending<'all> {}
