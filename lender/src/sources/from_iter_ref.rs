use core::iter::FusedIterator;
use core::num::NonZeroUsize;
use core::ops::ControlFlow;

use crate::try_trait_v2::Try;
use crate::{FusedLender, prelude::*};

/// Creates a lender that stores each element from an iterator
/// and lends a reference to it.
///
/// This function can be conveniently accessed using the
/// [`into_ref_lender`](crate::traits::IteratorRefExt::into_ref_lender)
/// extension method.
///
/// Unlike [`from_iter`](crate::from_iter), which passes items
/// through transparently, this source stores each element
/// internally and lends a reference to it, turning an
/// `Iterator<Item = T>` into a `Lender` with
/// `Lend<'lend> = &'lend T`.
///
/// # Examples
/// ```rust
/// # use lender::prelude::*;
/// let mut lender = lender::from_iter_ref([1, 2, 3].into_iter());
/// let item: &i32 = lender.next().unwrap();
/// assert_eq!(*item, 1);
/// ```
#[inline]
pub fn from_iter_ref<I: Iterator>(iter: I) -> FromIterRef<I> {
    FromIterRef {
        iter,
        current: None,
    }
}

/// A lender that stores each element from an iterator and
/// lends a reference to it.
///
/// This `struct` is created by the [`from_iter_ref()`]
/// function.
#[derive(Clone, Debug)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct FromIterRef<I: Iterator> {
    iter: I,
    current: Option<I::Item>,
}

impl<'lend, I: Iterator> Lending<'lend> for FromIterRef<I> {
    type Lend = &'lend I::Item;
}

impl<I: Iterator> Lender for FromIterRef<I> {
    crate::check_covariance!();

    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        self.current = self.iter.next();
        self.current.as_ref()
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Lend<'_, Self>> {
        self.current = self.iter.nth(n);
        self.current.as_ref()
    }

    #[inline(always)]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.iter.count()
    }

    #[inline]
    fn advance_by(&mut self, n: usize) -> Result<(), NonZeroUsize> {
        for i in 0..n {
            if self.iter.next().is_none() {
                // SAFETY: `i` is always less than `n`.
                return Err(unsafe { NonZeroUsize::new_unchecked(n - i) });
            }
        }
        Ok(())
    }

    #[inline]
    fn last<'call>(&'call mut self) -> Option<Lend<'call, Self>>
    where
        Self: Sized,
    {
        self.current = None;
        for x in self.iter.by_ref() {
            self.current = Some(x);
        }
        self.current.as_ref()
    }

    #[inline]
    fn try_fold<B, F, R>(&mut self, init: B, mut f: F) -> R
    where
        F: FnMut(B, Lend<'_, Self>) -> R,
        R: Try<Output = B>,
    {
        let mut acc = init;
        for item in self.iter.by_ref() {
            acc = match f(acc, &item).branch() {
                ControlFlow::Break(x) => return R::from_residual(x),
                ControlFlow::Continue(x) => x,
            };
        }
        R::from_output(acc)
    }

    #[inline]
    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> B,
    {
        self.iter.fold(init, |acc, item| f(acc, &item))
    }
}

impl<I: DoubleEndedIterator> DoubleEndedLender for FromIterRef<I> {
    #[inline]
    fn next_back(&mut self) -> Option<Lend<'_, Self>> {
        self.current = self.iter.next_back();
        self.current.as_ref()
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Lend<'_, Self>> {
        self.current = self.iter.nth_back(n);
        self.current.as_ref()
    }

    #[inline]
    fn advance_back_by(&mut self, n: usize) -> Result<(), NonZeroUsize> {
        for i in 0..n {
            if self.iter.next_back().is_none() {
                // SAFETY: `i` is always less than `n`.
                return Err(unsafe { NonZeroUsize::new_unchecked(n - i) });
            }
        }
        Ok(())
    }

    #[inline]
    fn try_rfold<B, F, R>(&mut self, init: B, mut f: F) -> R
    where
        F: FnMut(B, Lend<'_, Self>) -> R,
        R: Try<Output = B>,
    {
        let mut acc = init;
        while let Some(item) = self.iter.next_back() {
            acc = match f(acc, &item).branch() {
                ControlFlow::Break(x) => return R::from_residual(x),
                ControlFlow::Continue(x) => x,
            };
        }
        R::from_output(acc)
    }

    #[inline]
    fn rfold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> B,
    {
        self.iter.rfold(init, |acc, item| f(acc, &item))
    }
}

impl<I: ExactSizeIterator> ExactSizeLender for FromIterRef<I> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<I: FusedIterator> FusedLender for FromIterRef<I> {}

impl<I: Iterator> From<I> for FromIterRef<I> {
    #[inline(always)]
    fn from(iter: I) -> Self {
        from_iter_ref(iter)
    }
}
