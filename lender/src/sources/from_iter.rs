use core::iter::FusedIterator;

use crate::{prelude::*, FusedLender};

/// Creates a lender from an iterator.
///
/// This function can be conveniently accessed using the
/// [`into_lendwe`](crate::traits::IteratorExt::into_lender) method
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
pub fn from_iter<I: Iterator>(iter: I) -> FromIter<I> { FromIter { iter } }

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

impl<'lend, I: Iterator> Lending<'lend> for FromIter<I> {
    type Lend = I::Item;
}

impl<I: Iterator> Lender for FromIter<I> {
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> { self.iter.next() }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) { self.iter.size_hint() }
}

impl<I: DoubleEndedIterator> DoubleEndedLender for FromIter<I> {
    fn next_back(&mut self) -> Option<Lend<'_, Self>> { self.iter.next_back() }
}

impl<I: ExactSizeIterator> ExactSizeLender for FromIter<I> {
    fn len(&self) -> usize { self.iter.len() }
}

impl<I: FusedIterator> FusedLender for FromIter<I> {}

impl<I: Iterator> From<I> for FromIter<I> {
    #[inline]
    fn from(iter: I) -> Self { from_iter(iter) }
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
pub fn from_into_iter<I: IntoIterator>(into_iter: I) -> FromIntoIter<I> { FromIntoIter { into_iter } }

/// A [`IntoLender`] that returns lenders obtained by applying [`from_iter`]
/// to the iterators returned by the wrapped [`IntoIterator`].
///
/// This `struct` is created by the [`from_into_iter()`] function.
///

#[repr(transparent)]
#[derive(Clone, Debug)]
pub struct FromIntoIter<I> {
    into_iter: I,
}

impl<I: IntoIterator> IntoLender for FromIntoIter<I> {
    type Lender = FromIter<I::IntoIter>;

    fn into_lender(self) -> <Self as IntoLender>::Lender { self.into_iter.into_iter().into_lender() }
}

impl<I: IntoIterator> From<I> for FromIntoIter<I> {
    fn from(into_iter: I) -> Self { from_into_iter(into_iter) }
}

/// Creates a lender from an iterator `I`, safely shortening the items' lifetimes with the given lending type `L`.
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
    LendIter { iter, _marker: core::marker::PhantomData }
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
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        // SAFETY: 'a: 'lend
        unsafe { core::mem::transmute::<Option<Lend<'a, L>>, Option<Lend<'_, L>>>(self.iter.next()) }
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) { self.iter.size_hint() }
}

impl<'a, L, I> DoubleEndedLender for LendIter<'a, L, I>
where
    L: ?Sized + for<'all> Lending<'all> + 'a,
    I: DoubleEndedIterator<Item = Lend<'a, L>>,
{
    fn next_back(&mut self) -> Option<Lend<'_, Self>> {
        // SAFETY: 'a: 'lend
        unsafe { core::mem::transmute::<Option<Lend<'a, L>>, Option<Lend<'_, L>>>(self.iter.next_back()) }
    }
}

impl<'a, L, I> ExactSizeLender for LendIter<'a, L, I>
where
    L: ?Sized + for<'all> Lending<'all> + 'a,
    I: ExactSizeIterator<Item = Lend<'a, L>>,
{
    fn len(&self) -> usize { self.iter.len() }
}

impl<'a, L, I> FusedLender for LendIter<'a, L, I>
where
    L: ?Sized + for<'all> Lending<'all> + 'a,
    I: FusedIterator<Item = Lend<'a, L>>,
{
}
