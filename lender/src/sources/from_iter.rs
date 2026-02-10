use core::iter::FusedIterator;

use crate::{CovariantLending, FusedLender, IntoLender, prelude::*};

/// Creates a lender from an iterator.
///
/// This function can be conveniently accessed using the
/// [`into_lender`](crate::traits::IteratorExt::into_lender)
/// method added to [`Iterator`] by this crate.
///
/// Does not change the behavior of the iterator: the resulting
/// lender will yield the same items and can be adapted back
/// into an iterator.
///
/// # Examples
/// ```rust
/// # use lender::prelude::*;
/// let mut lender = [1, 2, 3].iter().into_lender();
/// let item: &'_ i32 = lender.next().unwrap();
/// let item2: &'_ i32 = lender.next().unwrap();
/// assert_eq!(*item + *item2, 3);
/// ```
#[inline]
pub fn from_iter<I: Iterator>(iter: I) -> FromIter<I> {
    FromIter { iter }
}

/// A lender that yields elements from an iterator.
///
/// This `struct` is created by the [`from_iter()`] function.
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
    #[inline(always)]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        self.iter.next()
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    #[inline(always)]
    fn nth(&mut self, n: usize) -> Option<Lend<'_, Self>> {
        self.iter.nth(n)
    }

    #[inline(always)]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.iter.count()
    }

    #[inline(always)]
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> B,
    {
        self.iter.fold(init, f)
    }
}

impl<I: DoubleEndedIterator> DoubleEndedLender for FromIter<I> {
    #[inline(always)]
    fn next_back(&mut self) -> Option<Lend<'_, Self>> {
        self.iter.next_back()
    }

    #[inline(always)]
    fn nth_back(&mut self, n: usize) -> Option<Lend<'_, Self>> {
        self.iter.nth_back(n)
    }

    #[inline(always)]
    fn rfold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> B,
    {
        self.iter.rfold(init, f)
    }
}

impl<I: ExactSizeIterator> ExactSizeLender for FromIter<I> {
    #[inline(always)]
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
/// [`into_into_lender`](crate::traits::IntoIteratorExt::into_into_lender)
/// method added to [`IntoIterator`] by this crate.
///
/// The lenders returned are obtained by applying [`from_iter`]
/// to the iterators returned by the wrapped [`IntoIterator`].
///
/// # Examples
/// ```rust
/// # use lender::prelude::*; let data = vec![1, 2, 3];
/// let into_lender = lender::from_into_iter(&data);
/// let mut lender = into_lender.into_lender();
/// assert_eq!(lender.next(), Some(&1));
/// ```
#[inline]
pub fn from_into_iter<I: IntoIterator>(into_iter: I) -> FromIntoIter<I> {
    FromIntoIter { into_iter }
}

/// A [`IntoLender`] that returns lenders obtained by applying
/// [`from_iter`] to the iterators returned by the wrapped
/// [`IntoIterator`].
///
/// This `struct` is created by the [`from_into_iter()`]
/// function.
#[repr(transparent)]
#[derive(Clone, Debug)]
pub struct FromIntoIter<I> {
    into_iter: I,
}

impl<I: IntoIterator> IntoLender for FromIntoIter<I> {
    type Lender = FromIter<I::IntoIter>;

    #[inline(always)]
    fn into_lender(self) -> <Self as IntoLender>::Lender {
        self.into_iter.into_iter().into_lender()
    }
}

impl<I: IntoIterator> From<I> for FromIntoIter<I> {
    fn from(into_iter: I) -> Self {
        from_into_iter(into_iter)
    }
}

/// Creates a lender from an iterator `I`, safely shortening
/// the items' lifetimes with the given lending type `L`.
///
/// If `I::Item` is 'static, behaves like [`from_iter`].
/// # Examples
/// ```rust
/// # use lender::prelude::*; let mut data = [1, 2, 3];
///
/// // properly shortens the lifetime of non-static items and lends them
/// let mut lender = lender::lend_iter::<'_, lend!(&'lend i32), _>(data.iter());
/// let lend: &'_ i32 = lender.next().unwrap();
///
/// // does not shorten the lifetime of 'static items, behaves like `from_iter`
/// let mut lender = lender::lend_iter::<'_, lend!(i32), _>([1, 2, 3].into_iter());
/// let item: i32 = lender.next().unwrap();
/// let item2: i32 = lender.next().unwrap();
/// ```
#[inline]
pub fn lend_iter<'a, L, I>(iter: I) -> LendIter<'a, L, I>
where
    L: ?Sized + CovariantLending + 'a,
    I: Iterator<Item = Lend<'a, L>>,
{
    LendIter {
        iter,
        _marker: core::marker::PhantomData,
    }
}

/// A lender that lends elements from an iterator by shortening
/// their lifetime.
///
/// If `I::Item` is 'static, behaves like [`FromIter`].
///
/// This `struct` is created by the [`lend_iter()`] function.
#[derive(Clone, Debug)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct LendIter<'a, L: ?Sized, I> {
    iter: I,
    _marker: core::marker::PhantomData<&'a L>,
}

impl<'a, 'lend, L, I> Lending<'lend> for LendIter<'a, L, I>
where
    L: ?Sized + CovariantLending + 'a,
    I: Iterator<Item = Lend<'a, L>>,
{
    type Lend = Lend<'lend, L>;
}

impl<'a, L, I> Lender for LendIter<'a, L, I>
where
    L: ?Sized + CovariantLending + 'a,
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

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Lend<'_, Self>> {
        // SAFETY: 'a: 'lend, and Lend<'a, L> is covariant in 'a
        unsafe {
            core::mem::transmute::<Option<Lend<'a, L>>, Option<Lend<'_, L>>>(self.iter.nth(n))
        }
    }

    #[inline(always)]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.iter.count()
    }

    #[inline]
    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> B,
    {
        self.iter.fold(init, |acc, x| {
            // SAFETY: 'a: 'lend, and Lend<'a, L> is covariant in 'a
            f(acc, unsafe {
                core::mem::transmute::<Lend<'a, L>, Lend<'_, L>>(x)
            })
        })
    }
}

impl<'a, L, I> DoubleEndedLender for LendIter<'a, L, I>
where
    L: ?Sized + CovariantLending + 'a,
    I: DoubleEndedIterator<Item = Lend<'a, L>>,
{
    #[inline]
    fn next_back(&mut self) -> Option<Lend<'_, Self>> {
        // SAFETY: 'a: 'lend, and caller must ensure Lend<'a, L> is covariant in 'a
        unsafe {
            core::mem::transmute::<Option<Lend<'a, L>>, Option<Lend<'_, L>>>(self.iter.next_back())
        }
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Lend<'_, Self>> {
        // SAFETY: 'a: 'lend, and Lend<'a, L> is covariant in 'a
        unsafe {
            core::mem::transmute::<Option<Lend<'a, L>>, Option<Lend<'_, L>>>(
                self.iter.nth_back(n),
            )
        }
    }

    #[inline]
    fn rfold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> B,
    {
        self.iter.rfold(init, |acc, x| {
            // SAFETY: 'a: 'lend, and Lend<'a, L> is covariant in 'a
            f(acc, unsafe {
                core::mem::transmute::<Lend<'a, L>, Lend<'_, L>>(x)
            })
        })
    }
}

impl<'a, L, I> ExactSizeLender for LendIter<'a, L, I>
where
    L: ?Sized + CovariantLending + 'a,
    I: ExactSizeIterator<Item = Lend<'a, L>>,
{
    #[inline(always)]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<'a, L, I> FusedLender for LendIter<'a, L, I>
where
    L: ?Sized + CovariantLending + 'a,
    I: FusedIterator<Item = Lend<'a, L>>,
{
}
