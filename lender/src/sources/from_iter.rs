use core::iter::FusedIterator;

use fallible_iterator::{DoubleEndedFallibleIterator, FallibleIterator, IntoFallibleIterator};

use crate::{
    prelude::*, DoubleEndedFallibleLender, FallibleLend, FallibleLender, FallibleLending,
    FusedLender, IntoFallibleLender,
};

/// Creates a lender from an iterator.
///
/// This function can be conveniently accessed using the
/// [`into_lender`](crate::traits::IteratorExt::into_lender) method
/// added to [`Iterator`] by this crate.
///
/// Does not change the behavior of the iterator, the resulting lender
/// will yield the same items and can be adapted back into an iterator.
///
/// # Examples
/// ```rust
/// use lender::prelude::*;
/// let mut data = [1u8, 2, 3];
///
/// let mut lender = lender::from_iter([1, 2, 3].iter());
/// let item: &'_ u8 = lender.next().unwrap();
/// let item2: &'_ u8 = lender.next().unwrap();
/// let x: u8 = *item + *item2; // == 3
///
/// let mut lender = [1, 2, 3].iter().into_lender();
/// let item: &'_ u8 = lender.next().unwrap();
/// let item2: &'_ u8 = lender.next().unwrap();
/// let x: u8 = *item + *item2; // == 3
/// ```
#[inline]
pub fn from_iter<I: Iterator>(iter: I) -> FromIter<I> {
    FromIter { iter }
}

/// A lender that yields elements from an iterator.
///
/// This `struct` is created by the [`from_iter()`] function.
///

#[derive(Clone, Debug)]
#[repr(transparent)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct FromIter<I> {
    iter: I,
}

impl<I: Iterator> Lending<'_> for FromIter<I> {
    type Lend = I::Item;
}

impl<I: Iterator> Lender for FromIter<I> {
    crate::check_covariance!();
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        self.iter.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<I: DoubleEndedIterator> DoubleEndedLender for FromIter<I> {
    fn next_back(&mut self) -> Option<Lend<'_, Self>> {
        self.iter.next_back()
    }
}

impl<I: ExactSizeIterator> ExactSizeLender for FromIter<I> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<I: FusedIterator> FusedLender for FromIter<I> {}

impl<I: Iterator> From<I> for FromIter<I> {
    #[inline]
    fn from(iter: I) -> Self {
        from_iter(iter)
    }
}

/// Creates an [`IntoLender`] from an [`IntoIterator`].
///
/// This function can be conveniently accessed using the
/// [`into_into_lender`](crate::traits::IntoIteratorExt::into_into_lender) method
/// added to [`IntoIterator`] by this crate.
///
/// The lenders returned are obtained by applying [`from_iter`]
/// to the iterators returned by the wrapped [`IntoIterator`].
///
#[inline]
pub fn from_into_iter<I: IntoIterator>(into_iter: I) -> FromIntoIter<I> {
    FromIntoIter { into_iter }
}

/// A [`IntoLender`] that returns lenders obtained by applying [`from_iter`]
/// to the iterators returned by the wrapped [`IntoIterator`].
///
/// This `struct` is created by the [`from_into_iter()`] function.
#[repr(transparent)]
#[derive(Clone, Debug)]
pub struct FromIntoIter<I> {
    into_iter: I,
}

impl<I: IntoIterator> IntoLender for FromIntoIter<I> {
    type Lender = FromIter<I::IntoIter>;

    fn into_lender(self) -> <Self as IntoLender>::Lender {
        self.into_iter.into_iter().into_lender()
    }
}

impl<I: IntoIterator> From<I> for FromIntoIter<I> {
    fn from(into_iter: I) -> Self {
        from_into_iter(into_iter)
    }
}

/// Creates a lender from an iterator `I`, safely shortening the items' lifetimes with the given
/// lending type `L`.
///
/// If `I::Item` is 'static, behaves like [`from_iter`].
/// # Examples
/// ```rust
/// use lender::prelude::*;
/// let mut data = [1u8, 2, 3];
///
/// // properly shortens the lifetime of non-static items and lends them
/// let mut lender = lender::lend_iter::<'_, lend!(&'lend u8), _>(data.iter());
/// let lend: &'_ u8 = lender.next().unwrap();
///
/// // does not shorten the lifetime of 'static items, behaves like `from_iter`
/// let mut lender = lender::lend_iter::<'_, lend!(u8), _>([1, 2, 3].into_iter());
/// let item: u8 = lender.next().unwrap();
/// let item2: u8 = lender.next().unwrap();
/// ```
#[inline]
pub fn lend_iter<'a, L, I>(iter: I) -> LendIter<'a, L, I>
where
    L: ?Sized + for<'all> Lending<'all> + 'a,
    I: Iterator<Item = Lend<'a, L>>,
{
    LendIter {
        iter,
        _marker: core::marker::PhantomData,
    }
}

/// A lender that lends elements from an iterator by shortening their lifetime.
///
/// If `I::Item` is 'static, behaves like [`FromIter`].
///
/// This `struct` is created by the [`lend_iter()`] function.
///

#[derive(Clone, Debug)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct LendIter<'a, L: ?Sized, I> {
    iter: I,
    _marker: core::marker::PhantomData<fn() -> &'a L>,
}

impl<'a, 'lend, L, I> Lending<'lend> for LendIter<'a, L, I>
where
    L: ?Sized + for<'all> Lending<'all> + 'a,
    I: Iterator<Item = Lend<'a, L>>,
{
    type Lend = Lend<'lend, L>;
}

impl<'a, L, I> Lender for LendIter<'a, L, I>
where
    L: ?Sized + for<'all> Lending<'all> + 'a,
    I: Iterator<Item = Lend<'a, L>>,
{
    // SAFETY: the lend is the type parameter L
    crate::unsafe_assume_covariance!();
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        // SAFETY: 'a: 'lend, and caller must ensure Lend<'a, L> is covariant in 'a
        unsafe {
            core::mem::transmute::<Option<Lend<'a, L>>, Option<Lend<'_, L>>>(self.iter.next())
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, L, I> DoubleEndedLender for LendIter<'a, L, I>
where
    L: ?Sized + for<'all> Lending<'all> + 'a,
    I: DoubleEndedIterator<Item = Lend<'a, L>>,
{
    fn next_back(&mut self) -> Option<Lend<'_, Self>> {
        // SAFETY: 'a: 'lend, and caller must ensure Lend<'a, L> is covariant in 'a
        unsafe {
            core::mem::transmute::<Option<Lend<'a, L>>, Option<Lend<'_, L>>>(self.iter.next_back())
        }
    }
}

impl<'a, L, I> ExactSizeLender for LendIter<'a, L, I>
where
    L: ?Sized + for<'all> Lending<'all> + 'a,
    I: ExactSizeIterator<Item = Lend<'a, L>>,
{
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<'a, L, I> FusedLender for LendIter<'a, L, I>
where
    L: ?Sized + for<'all> Lending<'all> + 'a,
    I: FusedIterator<Item = Lend<'a, L>>,
{
}

/// Creates a lender from a fallible iterator.
///
/// This function can be conveniently accessed using the
/// [`into_fallible_lender`](crate::traits::FallibleIteratorExt::into_fallible_lender) method
/// added to [`FallibleIterator`] by this crate.
///
/// Does not change the behavior of the iterator, the resulting lender
/// will yield the same items and can be adapted back into an iterator.
#[inline]
pub fn from_fallible_iter<I: FallibleIterator>(iter: I) -> FromFallibleIter<I> {
    FromFallibleIter { iter }
}

/// A lender that yields elements from a fallible iterator.
///
/// This `struct` is created by the [`from_fallible_iter()`] function.

#[derive(Clone, Debug)]
#[repr(transparent)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct FromFallibleIter<I> {
    iter: I,
}

impl<I: FallibleIterator> FallibleLending<'_> for FromFallibleIter<I> {
    type Lend = I::Item;
}

impl<I: FallibleIterator> FallibleLender for FromFallibleIter<I> {
    type Error = I::Error;
    crate::check_covariance_fallible!();
    #[inline]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        self.iter.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<I: DoubleEndedFallibleIterator> DoubleEndedFallibleLender for FromFallibleIter<I> {
    fn next_back(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        self.iter.next_back()
    }
}

impl<I: FallibleIterator> From<I> for FromFallibleIter<I> {
    #[inline]
    fn from(iter: I) -> Self {
        from_fallible_iter(iter)
    }
}

/// Creates an [`IntoFallibleLender`] from an [`IntoFallibleIterator`].
///
/// This function can be conveniently accessed using the
/// [`into_into_lender`](crate::traits::IntoIteratorExt::into_into_lender) method
/// added to [`IntoIterator`] by this crate.
///
/// The lenders returned are obtained by applying [`from_iter`]
/// to the iterators returned by the wrapped [`IntoIterator`].
///
#[inline]
pub fn from_into_fallible_iter<I: IntoFallibleIterator>(into_iter: I) -> FromIntoFallibleIter<I> {
    FromIntoFallibleIter { into_iter }
}

/// A [`IntoFallibleLender`] that returns lenders obtained by applying [`from_iter`]
/// to the iterators returned by the wrapped [`IntoFallibleIterator`].
///
/// This `struct` is created by the [`from_into_fallible_iter()`] function.
#[repr(transparent)]
#[derive(Clone, Debug)]
pub struct FromIntoFallibleIter<I> {
    into_iter: I,
}

impl<I: IntoFallibleIterator> IntoFallibleLender for FromIntoFallibleIter<I> {
    type Error = I::Error;

    type FallibleLender = FromFallibleIter<I::IntoFallibleIter>;

    fn into_fallible_lender(self) -> <Self as IntoFallibleLender>::FallibleLender {
        self.into_iter.into_fallible_iter().into_fallible_lender()
    }
}

impl<I: IntoFallibleIterator> From<I> for FromIntoFallibleIter<I> {
    fn from(into_iter: I) -> Self {
        from_into_fallible_iter(into_iter)
    }
}

/// Creates a fallible lender from a fallible iterator `I`, safely shortening
/// the items' lifetimes with the given lending type `L`.
///
/// If `I::Item` is 'static, behaves like [`from_fallible_iter`].
///
/// # Examples
/// ```rust
/// use fallible_iterator::IteratorExt as _;
/// use lender::prelude::*;
/// let mut data = [1u8, 2, 3];
///
/// // properly shortens the lifetime of non-static items and lends them
/// let mut lender = lender::lend_fallible_iter::<'_, fallible_lend!(&'lend u8), _>(data.iter().into_fallible());
/// let lend: Option<&'_ u8> = lender.next().unwrap();
/// let lend: &'_ u8 = lend.unwrap();
///
/// // does not shorten the lifetime of 'static items, behaves like `from_iter`
/// let mut lender = lender::lend_fallible_iter::<'_, fallible_lend!(u8), _>([1, 2, 3].into_iter().into_fallible());
/// let item: Option<u8> = lender.next().unwrap();
/// let item: u8 = item.unwrap();
/// let item2: Option<u8> = lender.next().unwrap();
/// let item2: u8 = item2.unwrap();
/// ```
#[inline]
pub fn lend_fallible_iter<'a, L, I>(iter: I) -> LendFallibleIter<'a, L, I>
where
    L: ?Sized + for<'all> FallibleLending<'all> + 'a,
    I: FallibleIterator<Item = FallibleLend<'a, L>>,
{
    LendFallibleIter {
        iter,
        _marker: core::marker::PhantomData,
    }
}

/// A lender that lends elements from a fallible iterator by shortening their lifetime.
///
/// If `I::Item` is 'static, behaves like [`FromFallibleIter`].
///
/// This `struct` is created by the [`lend_fallible_iter()`] function.

#[derive(Clone, Debug)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct LendFallibleIter<'a, L: ?Sized, I> {
    iter: I,
    _marker: core::marker::PhantomData<fn() -> &'a L>,
}

impl<'a, 'lend, L, I> FallibleLending<'lend> for LendFallibleIter<'a, L, I>
where
    L: ?Sized + for<'all> FallibleLending<'all> + 'a,
    I: FallibleIterator<Item = FallibleLend<'a, L>>,
{
    type Lend = FallibleLend<'lend, L>;
}

impl<'a, L, I> FallibleLender for LendFallibleIter<'a, L, I>
where
    L: ?Sized + for<'all> FallibleLending<'all> + 'a,
    I: FallibleIterator<Item = FallibleLend<'a, L>>,
{
    type Error = I::Error;
    // SAFETY: the lend is the type parameter L
    crate::unsafe_assume_covariance_fallible!();

    #[inline]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        let next = self.iter.next()?;
        Ok(
            // SAFETY: 'a: 'lend
            unsafe {
                core::mem::transmute::<Option<FallibleLend<'a, L>>, Option<FallibleLend<'_, L>>>(
                    next,
                )
            },
        )
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, L, I> DoubleEndedFallibleLender for LendFallibleIter<'a, L, I>
where
    L: ?Sized + for<'all> FallibleLending<'all> + 'a,
    I: DoubleEndedFallibleIterator<Item = FallibleLend<'a, L>>,
{
    fn next_back(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        let next = self.iter.next_back()?;
        Ok(
            // SAFETY: 'a: 'lend
            unsafe {
                core::mem::transmute::<Option<FallibleLend<'a, L>>, Option<FallibleLend<'_, L>>>(
                    next,
                )
            },
        )
    }
}
