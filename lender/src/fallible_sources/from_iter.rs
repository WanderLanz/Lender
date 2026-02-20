use fallible_iterator::{DoubleEndedFallibleIterator, FallibleIterator, IntoFallibleIterator};

use crate::{
    CovariantFallibleLending, DoubleEndedFallibleLender, FallibleLend, FallibleLender,
    FallibleLending, IntoFallibleLender, prelude::*,
};

/// Creates a lender from a fallible iterator.
///
/// This function can be conveniently accessed using the
/// [`into_fallible_lender`][ifl] method added to
/// [`FallibleIterator`] by this crate.
///
/// [ifl]: crate::traits::FallibleIteratorExt::into_fallible_lender
///
/// Does not change the behavior of the iterator: the resulting
/// lender will yield the same items and can be adapted back into
/// an iterator.
///
/// # Examples
/// ```rust
/// # use lender::prelude::*; let data = [1, 2, 3];
/// let mut lender = data.iter().into_lender().into_fallible();
/// assert_eq!(lender.next().unwrap(), Some(&1));
/// ```
#[inline]
pub fn from_iter<I: FallibleIterator>(iter: I) -> FromIter<I> {
    FromIter { iter }
}

/// A lender that yields elements from a fallible iterator.
///
/// This `struct` is created by the
/// [`from_fallible_iter()`](crate::from_fallible_iter)
/// function.
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
    #[inline]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        self.iter.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        self.iter.nth(n)
    }

    #[inline]
    fn count(self) -> Result<usize, Self::Error>
    where
        Self: Sized,
    {
        self.iter.count()
    }

    #[inline]
    fn fold<B, F>(self, init: B, f: F) -> Result<B, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<B, Self::Error>,
    {
        self.iter.fold(init, f)
    }
}

impl<I: DoubleEndedFallibleIterator> DoubleEndedFallibleLender for FromIter<I> {
    #[inline]
    fn next_back(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        self.iter.next_back()
    }

    #[inline]
    fn rfold<B, F>(self, init: B, f: F) -> Result<B, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<B, Self::Error>,
    {
        self.iter.rfold(init, f)
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
/// [`into_into_fallible_lender`][iifl] method added to
/// [`IntoFallibleIterator`] by this crate.
///
/// [iifl]: crate::traits::IntoFallibleIteratorExt::into_into_fallible_lender
///
/// The lenders returned are obtained by applying
/// [`from_fallible_iter()`](crate::from_fallible_iter) to
/// the iterators returned by the wrapped
/// [`IntoFallibleIterator`].
///
/// # Examples
/// ```rust
/// # use lender::prelude::*;
/// # use fallible_iterator::IteratorExt;
/// let data = [1, 2, 3];
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

/// An [`IntoFallibleLender`] that returns lenders obtained
/// by applying
/// [`from_fallible_iter()`](crate::from_fallible_iter) to
/// the iterators returned by the wrapped
/// [`IntoFallibleIterator`].
///
/// This `struct` is created by the
/// [`from_into_fallible_iter()`](crate::from_into_fallible_iter)
/// function.
#[repr(transparent)]
#[derive(Clone, Debug)]
pub struct FromIntoIter<I> {
    into_iter: I,
}

impl<I: IntoFallibleIterator> IntoFallibleLender for FromIntoIter<I> {
    type Error = I::Error;

    type FallibleLender = FromIter<I::IntoFallibleIter>;

    #[inline]
    fn into_fallible_lender(self) -> <Self as IntoFallibleLender>::FallibleLender {
        self.into_iter.into_fallible_iter().into_fallible_lender()
    }
}

impl<I: IntoFallibleIterator> From<I> for FromIntoIter<I> {
    #[inline]
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
/// # use lender::prelude::*;
/// # use fallible_iterator::IteratorExt;
/// let mut data = [1, 2, 3];
///
/// // properly shortens the lifetime of non-static items and lends them
/// let mut lender = lender::lend_fallible_iter::<
///     '_, fallible_lend!(&'lend i32), _,
/// >(data.iter().into_fallible());
/// let lend: Option<&'_ i32> = lender.next().unwrap();
/// let lend: &'_ i32 = lend.unwrap();
///
/// // does not shorten the lifetime of 'static items
/// let mut lender = lender::lend_fallible_iter::<
///     '_, fallible_lend!(i32), _,
/// >([1, 2, 3].into_iter().into_fallible());
/// let item: Option<i32> = lender.next().unwrap();
/// let item: i32 = item.unwrap();
/// let item2: Option<i32> = lender.next().unwrap();
/// let item2: i32 = item2.unwrap();
/// ```
#[inline]
pub fn lend_iter<'a, L, I>(iter: I) -> LendIter<'a, L, I>
where
    L: ?Sized + CovariantFallibleLending + 'a,
    I: FallibleIterator<Item = FallibleLend<'a, L>>,
{
    crate::__check_fallible_lending_covariance::<L>();
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
/// This `struct` is created by the
/// [`lend_fallible_iter()`](crate::lend_fallible_iter)
/// function.
#[derive(Clone, Debug)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct LendIter<'a, L: ?Sized, I> {
    iter: I,
    _marker: core::marker::PhantomData<&'a L>,
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

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        let nth = self.iter.nth(n)?;
        Ok(
            // SAFETY: 'a: 'lend
            unsafe {
                core::mem::transmute::<Option<FallibleLend<'a, L>>, Option<FallibleLend<'_, L>>>(
                    nth,
                )
            },
        )
    }

    #[inline]
    fn count(self) -> Result<usize, Self::Error>
    where
        Self: Sized,
    {
        self.iter.count()
    }

    #[inline]
    fn fold<B, F>(self, init: B, mut f: F) -> Result<B, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<B, Self::Error>,
    {
        self.iter.fold(init, |acc, x| {
            // SAFETY: 'a: 'lend, and FallibleLend<'a, L> is covariant in 'a
            f(acc, unsafe {
                core::mem::transmute::<FallibleLend<'a, L>, FallibleLend<'_, L>>(x)
            })
        })
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

    #[inline]
    fn rfold<B, F>(self, init: B, mut f: F) -> Result<B, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<B, Self::Error>,
    {
        self.iter.rfold(init, |acc, x| {
            // SAFETY: 'a: 'lend, and FallibleLend<'a, L> is covariant in 'a
            f(acc, unsafe {
                core::mem::transmute::<FallibleLend<'a, L>, FallibleLend<'_, L>>(x)
            })
        })
    }
}

// Note: FusedFallibleLender and ExactSizeFallibleLender are not
// implemented for LendIter because the fallible_iterator crate
// does not expose FusedFallibleIterator or
// ExactSizeFallibleIterator marker traits.
