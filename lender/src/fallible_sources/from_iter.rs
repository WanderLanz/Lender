use fallible_iterator::{DoubleEndedFallibleIterator, FallibleIterator, IntoFallibleIterator};

use crate::{
    CovariantFallibleLending, DoubleEndedFallibleLender, FallibleLend,
    FallibleLender, FallibleLending, IntoFallibleLender, prelude::*,
};

/// Creates a lender from a fallible iterator.
///
/// This function can be conveniently accessed using the
/// [`into_fallible_lender`](crate::traits::FallibleIteratorExt::into_fallible_lender)
/// method added to [`FallibleIterator`] by this crate.
///
/// Does not change the behavior of the iterator: the resulting
/// lender will yield the same items and can be adapted back into
/// an iterator.
///
/// # Examples
/// ```rust
/// use fallible_iterator::IteratorExt as _;
/// use lender::prelude::*;
/// let data = vec![1i32, 2, 3];
/// let mut lender = lender::from_fallible_iter(data.iter().into_fallible());
/// assert_eq!(lender.next().unwrap(), Some(&1));
/// ```
#[inline]
pub fn from_iter<I: FallibleIterator>(iter: I) -> FromIter<I> {
    FromIter { iter }
}

/// A lender that yields elements from a fallible iterator.
///
/// This `struct` is created by the [`from_iter()`] function.

#[derive(Clone, Debug)]
#[repr(transparent)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct FromIter<I> {
    iter: I,
}

impl<I: FallibleIterator> FallibleLending<'_> for FromIter<I> {
    type Lend = I::Item;
}

impl<I: FallibleIterator> FallibleLender for FromIter<I> {
    type Error = I::Error;
    crate::check_covariance_fallible!();
    #[inline(always)]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        self.iter.next()
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<I: DoubleEndedFallibleIterator> DoubleEndedFallibleLender for FromIter<I> {
    #[inline(always)]
    fn next_back(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        self.iter.next_back()
    }
}

// Note: FusedFallibleLender and ExactSizeFallibleLender are not
// implemented for FromIter because the fallible_iterator crate
// does not expose FusedFallibleIterator or
// ExactSizeFallibleIterator marker traits.

impl<I: FallibleIterator> From<I> for FromIter<I> {
    #[inline]
    fn from(iter: I) -> Self {
        from_iter(iter)
    }
}

/// Creates an [`IntoFallibleLender`] from an
/// [`IntoFallibleIterator`].
///
/// This function can be conveniently accessed using the
/// [`into_into_fallible_lender`](crate::traits::IntoFallibleIteratorExt::into_into_fallible_lender)
/// method added to [`IntoFallibleIterator`] by this crate.
///
/// The lenders returned are obtained by applying
/// [`from_iter()`] to the iterators returned by the
/// wrapped [`IntoFallibleIterator`].
///
/// # Examples
/// ```rust
/// use fallible_iterator::IteratorExt as _;
/// use lender::prelude::*;
/// let data = vec![1i32, 2, 3];
/// let into_lender = lender::from_into_fallible_iter(
///     data.iter().into_fallible(),
/// );
/// let mut lender = into_lender.into_fallible_lender();
/// assert_eq!(lender.next().unwrap(), Some(&1));
/// ```
#[inline]
pub fn from_into_iter<I: IntoFallibleIterator>(into_iter: I) -> FromIntoIter<I> {
    FromIntoIter { into_iter }
}

/// A [`IntoFallibleLender`] that returns lenders obtained by
/// applying [`from_iter()`] to the iterators returned by the
/// wrapped [`IntoFallibleIterator`].
///
/// This `struct` is created by the [`from_into_iter()`]
/// function.
#[repr(transparent)]
#[derive(Clone, Debug)]
pub struct FromIntoIter<I> {
    into_iter: I,
}

impl<I: IntoFallibleIterator> IntoFallibleLender for FromIntoIter<I> {
    type Error = I::Error;

    type FallibleLender = FromIter<I::IntoFallibleIter>;

    fn into_fallible_lender(self) -> <Self as IntoFallibleLender>::FallibleLender {
        self.into_iter.into_fallible_iter().into_fallible_lender()
    }
}

impl<I: IntoFallibleIterator> From<I> for FromIntoIter<I> {
    fn from(into_iter: I) -> Self {
        from_into_iter(into_iter)
    }
}

/// Creates a fallible lender from a fallible iterator `I`,
/// safely shortening the items' lifetimes with the given
/// lending type `L`.
///
/// If `I::Item` is 'static, behaves like [`from_iter()`].
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
pub fn lend_iter<'a, L, I>(iter: I) -> LendIter<'a, L, I>
where
    L: ?Sized + CovariantFallibleLending + 'a,
    I: FallibleIterator<Item = FallibleLend<'a, L>>,
{
    LendIter {
        iter,
        _marker: core::marker::PhantomData,
    }
}

/// A lender that lends elements from a fallible iterator by
/// shortening their lifetime.
///
/// If `I::Item` is 'static, behaves like [`FromIter`].
///
/// This `struct` is created by the [`lend_iter()`] function.

#[derive(Clone, Debug)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct LendIter<'a, L: ?Sized, I> {
    iter: I,
    _marker: core::marker::PhantomData<fn() -> &'a L>,
}

impl<'a, 'lend, L, I> FallibleLending<'lend> for LendIter<'a, L, I>
where
    L: ?Sized + CovariantFallibleLending + 'a,
    I: FallibleIterator<Item = FallibleLend<'a, L>>,
{
    type Lend = FallibleLend<'lend, L>;
}

impl<'a, L, I> FallibleLender for LendIter<'a, L, I>
where
    L: ?Sized + CovariantFallibleLending + 'a,
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

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, L, I> DoubleEndedFallibleLender for LendIter<'a, L, I>
where
    L: ?Sized + CovariantFallibleLending + 'a,
    I: DoubleEndedFallibleIterator<Item = FallibleLend<'a, L>>,
{
    #[inline]
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

// Note: FusedFallibleLender and ExactSizeFallibleLender are not
// implemented for LendIter because the fallible_iterator crate
// does not expose FusedFallibleIterator or
// ExactSizeFallibleIterator marker traits.
