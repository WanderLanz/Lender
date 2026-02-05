use alloc::borrow::ToOwned;
use core::{cmp::Ordering, num::NonZeroUsize, ops::ControlFlow};

use stable_try_trait_v2::{
    ChangeOutputType, FromResidual, Residual, Try, internal::NeverShortCircuit,
};

use crate::{
    Chain, Chunk, Chunky, Cloned, Copied, Covar, Cycle, DoubleEndedFallibleLender, Enumerate,
    ExactSizeFallibleLender, ExtendLender, FallibleFlatMap, FallibleFlatten, FallibleIntersperse,
    FallibleIntersperseWith, FalliblePeekable, FallibleTryShuntAdapter, Filter, FilterMap,
    FirstShunt, FromLender, Fuse, ImplBound, Inspect, Iter, Map, MapErr, MapIntoIter, MapWhile,
    Mutate, NonFallibleAdapter, Owned, ProductFallibleLender, Ref, Rev, Scan, SecondShunt, Skip,
    SkipWhile, StepBy, SumFallibleLender, Take, TakeWhile, TupleLend, Zip, fallible_unzip,
    higher_order::{FnMutHKARes, FnMutHKAResOpt},
    non_fallible_adapter,
    traits::collect::IntoFallibleLender,
};

/// The fallible counterpart of [`Lending`](crate::Lending). See its documentation
/// for details on the HRTB implicit-bound technique used here.
///
/// Must be defined for any type that implements [`FallibleLender`].
pub trait FallibleLending<'lend, __ImplBound: ImplBound = Ref<'lend, Self>> {
    /// The type being lent.
    type Lend: 'lend;
}

/// A readable shorthand for the type of the items of a [`FallibleLender`] `L`.
pub type FallibleLend<'lend, L> = <L as FallibleLending<'lend>>::Lend;

/// A trait for dealing with fallible lending iterators.
///
/// This is the main fallible lender trait. For more about the concept of lenders
/// generally, please see the [crate documentation](crate).
///
/// For more about the concept of iterators
/// generally, please see [`core::iter`].
pub trait FallibleLender: for<'all /* where Self: 'all */> FallibleLending<'all> {
    /// The error type.
    type Error;

    /// Internal method for compile-time covariance checking.
    /// 
    /// This method should rarely be implemented directly. Instead, use the
    /// provided macros:
    ///
    /// - When implementing source lenders (lenders with concrete
    ///   [`Lend`](FallibleLending::Lend) types), users should invoke
    ///   [`check_covariance_fallible!`](crate::check_covariance_fallible) in
    ///   their [`FallibleLender`] impl. The macro implements the method as `{
    ///   lend }`, which only compiles if the [`Lend`](FallibleLending::Lend) type
    ///   is covariant in its lifetime.
    ///
    /// - In all other cases (e.g., when implementing adapters), use
    ///   [`unsafe_assume_covariance_fallible!`](crate::unsafe_assume_covariance_fallible)
    ///   in the [`FallibleLender`] impl. The macro implements the method
    ///   as `{ unsafe { core::mem::transmute(lend) } }`, which is a no-op.
    ///   This is
    ///   unsafe because it is up to the implementor to guarantee that the
    ///   [`Lend`](FallibleLending::Lend) type is covariant in its lifetime.
    fn __check_covariance<'long: 'short, 'short>(
        lend: *const &'short <Self as FallibleLending<'long>>::Lend, _: crate::Uncallable,
    ) -> *const &'short <Self as FallibleLending<'short>>::Lend;

    /// Yields the next lend, if any, of the lender, or `Ok(None)` when iteration
    /// is finished.
    ///
    /// The behavior of calling this method after a previous call has returned
    /// `Ok(None)` or `Err` is implementation defined. See
    /// [`FusedFallibleLender`](crate::FusedFallibleLender) for lenders that
    /// guarantee `Ok(None)` is repeated after `Ok(None)`.
    ///
    /// Every lend is only guaranteed to be valid one at a time for any kind of
    /// lender.
    /// 
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>(
    ///     [1, 2, 3].iter().into_fallible(),
    /// );
    /// assert_eq!(lender.next(), Ok(Some(&1)));
    /// assert_eq!(lender.next(), Ok(Some(&2)));
    /// assert_eq!(lender.next(), Ok(Some(&3)));
    /// assert_eq!(lender.next(), Ok(None));
    /// ```
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error>;

    /// Takes the next `len` lends of the lender with temporary lender
    /// [`Chunk`]. This is equivalent to cloning the lender and calling
    /// [`take(len)`](FallibleLender::take) on it.
    /// 
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>(
    ///     [1, 2, 3].iter().into_fallible(),
    /// );
    /// let mut chunk_lender = lender.next_chunk(2);
    /// assert_eq!(chunk_lender.next(), Ok(Some(&1)));
    /// assert_eq!(chunk_lender.next(), Ok(Some(&2)));
    /// assert_eq!(chunk_lender.next(), Ok(None));
    /// ```
    #[inline]
    fn next_chunk(&mut self, chunk_size: usize) -> Chunk<'_, Self>
    where
        Self: Sized,
    {
        Chunk::new(self, chunk_size)
    }

    /// Gets the estimated minimum and maximum length of the lender. Both
    /// bounds assume that all remaining calls to
    /// [`next()`](FallibleLender::next) succeed. That is,
    /// [`next()`](FallibleLender::next) could return an [`Err`] in fewer
    /// calls than specified by the lower bound.
    /// 
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>(
    ///     [1, 2, 3].iter().into_fallible(),
    /// );
    /// assert_eq!(lender.size_hint(), (3, Some(3)));
    /// ```
    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) { (0, None) }

    /// Counts the number of lends in the lender by consuming it until the
    /// lender yields `Ok(None)` or `Err(_)`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>(
    ///     [1, 2, 3].iter().into_fallible(),
    /// );
    /// assert_eq!(lender.count(), Ok(3));
    /// ```
    #[inline]
    fn count(self) -> Result<usize, Self::Error>
    where Self: Sized
    {
        self.fold(0, |n, _| Ok(n + 1))
    }

    /// Returns the last element of the lender.
    #[inline]
    fn last<'call>(&'call mut self) -> Result<Option<FallibleLend<'call, Self>>, Self::Error>
    where
        Self: Sized,
    {
        let mut last = None;
        while let Some(x) = self.next()? {
            // SAFETY: polonius return
            last = Some(unsafe {
                core::mem::transmute::<
                    FallibleLend<'_, Self>,
                    FallibleLend<'call, Self>
                >(x)
            });
        }
        Ok(last)
    }

    /// Advances the lender by `n` lends.
    ///
    /// Returns `Ok(Ok(()))` if the lender was successfully advanced by `n` elements.
    /// Returns `Ok(Err(k))` if the lender did not have enough elements, where `k` is
    /// the number of elements that could not be skipped.
    /// Returns `Err(e)` if an error occurred while advancing.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>(
    ///     [1, 2, 3u8].iter().into_fallible(),
    /// );
    /// assert_eq!(lender.advance_by(2), Ok(Ok(())));
    /// assert_eq!(lender.next(), Ok(Some(&3)));
    /// ```
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// # use core::num::NonZeroUsize;
    /// let mut lender = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>(
    ///     [1, 2u8].iter().into_fallible(),
    /// );
    /// // Trying to advance by 5 when only 2 elements remain
    /// assert_eq!(lender.advance_by(5), Ok(Err(NonZeroUsize::new(3).unwrap())));
    /// ```
    #[inline]
    fn advance_by(&mut self, n: usize) -> Result<Result<(), NonZeroUsize>, Self::Error> {
        for i in 0..n {
            if self.next()?.is_none() {
                // SAFETY: `i` is always less than `n`.
                return Ok(Err(unsafe { NonZeroUsize::new_unchecked(n - i) }));
            }
        }
        Ok(Ok(()))
    }

    /// Yields the nth lend of the lender, if any, by
    /// consuming it. If the lender does not have enough
    /// lends, returns [`None`].
    ///
    /// `n` is zero-indexed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lender::prelude::*;
    /// let data = [1, 2, 3, 4, 5];
    /// let mut lender =
    ///     data.iter().into_lender().into_fallible();
    /// assert_eq!(lender.nth(2), Ok(Some(&3)));
    /// assert_eq!(lender.nth(0), Ok(Some(&4)));
    /// assert_eq!(lender.nth(5), Ok(None));
    /// ```
    #[inline]
    fn nth(&mut self, n: usize) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        if self.advance_by(n)?.is_err() {
            return Ok(None);
        }
        self.next()
    }

    /// Skips `step - 1` lends between each lend of the lender.
    ///
    /// # Panics
    ///
    /// Panics if `step` is zero.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let lender = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>(
    ///     [1, 2, 3, 4, 5, 6, 7, 8, 9, 10].iter().into_fallible(),
    /// );
    /// let mut step_lender = lender.step_by(2);
    /// assert_eq!(step_lender.next(), Ok(Some(&1)));
    /// assert_eq!(step_lender.next(), Ok(Some(&3)));
    /// assert_eq!(step_lender.next(), Ok(Some(&5)));
    /// assert_eq!(step_lender.next(), Ok(Some(&7)));
    /// assert_eq!(step_lender.next(), Ok(Some(&9)));
    /// assert_eq!(step_lender.next(), Ok(None));
    /// ```
    #[inline]
    fn step_by(self, step: usize) -> StepBy<Self>
    where
        Self: Sized,
    {
        StepBy::new(self, step)
    }

    /// Chains the lender with another lender of the same type.
    ///
    /// The resulting lender will first yield all lends from `self`, then
    /// all lends from `other`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let a = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>(
    ///     [1, 2].iter().into_fallible(),
    /// );
    /// let b = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>(
    ///     [3, 4].iter().into_fallible(),
    /// );
    /// let mut chained = a.chain(b);
    /// assert_eq!(chained.next(), Ok(Some(&1)));
    /// assert_eq!(chained.next(), Ok(Some(&2)));
    /// assert_eq!(chained.next(), Ok(Some(&3)));
    /// assert_eq!(chained.next(), Ok(Some(&4)));
    /// assert_eq!(chained.next(), Ok(None));
    /// ```
    #[inline]
    fn chain<U>(self, other: U) -> Chain<Self, <U as IntoFallibleLender>::FallibleLender>
        where
            Self: Sized,
            U: IntoFallibleLender + for<'all> FallibleLending<'all, Lend = FallibleLend<'all, Self>>,
    {
        Chain::new(self, other.into_fallible_lender())
    }

    /// Zips the lender with another lender of the same or different type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let a = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>(
    ///     [1, 2].iter().into_fallible(),
    /// );
    /// let b = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>(
    ///     [3, 4].iter().into_fallible(),
    /// );
    /// let mut zipped = a.zip(b);
    /// assert_eq!(zipped.next(), Ok(Some((&1, &3))));
    /// assert_eq!(zipped.next(), Ok(Some((&2, &4))));
    /// assert_eq!(zipped.next(), Ok(None));
    /// ```
    #[inline]
    fn zip<U: IntoFallibleLender>(self, other: U) -> Zip<Self, <U as IntoFallibleLender>::FallibleLender>
    where
        Self: Sized,
    {
        Zip::new(self, other.into_fallible_lender())
    }

    /// Intersperses each lend of this lender with the given separator.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let sep = 0;
    /// let lender = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>(
    ///     [1, 2, 3].iter().into_fallible(),
    /// );
    /// let mut inter = lender.intersperse(&sep);
    /// assert_eq!(inter.next(), Ok(Some(&1)));
    /// assert_eq!(inter.next(), Ok(Some(&0)));
    /// assert_eq!(inter.next(), Ok(Some(&2)));
    /// assert_eq!(inter.next(), Ok(Some(&0)));
    /// assert_eq!(inter.next(), Ok(Some(&3)));
    /// assert_eq!(inter.next(), Ok(None));
    /// ```
    #[inline]
    fn intersperse<'call>(self, separator: FallibleLend<'call, Self>) -> FallibleIntersperse<'call, Self>
    where
        Self: Sized,
        for<'all> FallibleLend<'all, Self>: Clone,
    {
        FallibleIntersperse::new(self, separator)
    }

    /// Intersperses each lend of this lender with the separator produced by
    /// the given function.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let sep = 0;
    /// let lender = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>(
    ///     [1, 2, 3].iter().into_fallible(),
    /// );
    /// let mut inter = lender.intersperse_with(|| Ok(&sep));
    /// assert_eq!(inter.next(), Ok(Some(&1)));
    /// assert_eq!(inter.next(), Ok(Some(&0)));
    /// assert_eq!(inter.next(), Ok(Some(&2)));
    /// assert_eq!(inter.next(), Ok(Some(&0)));
    /// assert_eq!(inter.next(), Ok(Some(&3)));
    /// assert_eq!(inter.next(), Ok(None));
    /// ```
    #[inline]
    fn intersperse_with<'call, G>(self, separator: G) -> FallibleIntersperseWith<'call, Self, G>
    where
        Self: Sized,
        G: FnMut() -> Result<FallibleLend<'call, Self>, Self::Error>,
    {
        FallibleIntersperseWith::new(self, separator)
    }

    /// Maps each lend of this lender using the given function.
    ///
    /// Note that functions passed to this method must be built using the
    /// [`covar!`](crate::covar) or [`covar_mut!`](crate::covar_mut) macros, which also
    /// checks for covariance of the returned type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// # use core::convert::Infallible;
    /// let mut data = [1, 2];
    /// let lender = lender::lend_fallible_iter::<fallible_lend!(&'lend mut u8), _>(
    ///     data.iter_mut().into_fallible(),
    /// );
    /// let mut mapped = lender.map(covar_mut!(
    ///     for<'all> |a: &'all mut u8| -> Result<&'all u8, Infallible> {
    ///         *a += 1;
    ///         Ok(&*a)
    ///     }
    /// ));
    /// assert_eq!(mapped.next().unwrap(), Some(&2));
    /// ```
    #[inline]
    fn map<F>(self, f: Covar<F>) -> Map<Self, F>
    where
        Self: Sized,
        F: for<'all> FnMutHKARes<'all, FallibleLend<'all, Self>, Self::Error>,
    {
        Map::new(self, f)
    }

    /// Maps the error of this lender using the given function.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lender::prelude::*;
    /// let data = [Ok(1), Ok(2), Err("oops")];
    /// let mut lender = data.into_iter().into_lender()
    ///     .convert::<&str>()
    ///     .map_err(|e| format!("error: {e}"));
    /// assert_eq!(lender.next(), Ok(Some(1)));
    /// assert_eq!(lender.next(), Ok(Some(2)));
    /// assert_eq!(lender.next(), Err(String::from("error: oops")));
    /// ```
    #[inline]
    fn map_err<E, F>(self, f: F) -> MapErr<E, Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Error) -> E,
    {
        MapErr::new(self, f)
    }

    /// Maps each lend of this lender into an owned value using the given function.
    ///
    /// This is a weaker version of [`FallibleLender::map`] that returns a
    /// [`FallibleIterator`](fallible_iterator::FallibleIterator) instead
    /// of a [`FallibleLender`]. However, this behavior is very common, and so
    /// this method is included for convenience. The main advantage is better
    /// type inference for the mapping function.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::{IteratorExt as _, FallibleIterator as _};
    /// # use lender::prelude::*;
    /// let mut data = [1, 2u8];
    /// let lender = lender::lend_fallible_iter::<fallible_lend!(&'lend mut u8), _>(
    ///     data.iter_mut().into_fallible(),
    /// );
    /// let mut mapped_into_iter = lender.map_into_iter(|a| { Ok(*a + 10) });
    /// assert_eq!(mapped_into_iter.next(), Ok(Some(11)));
    /// assert_eq!(mapped_into_iter.next(), Ok(Some(12)));
    /// assert_eq!(mapped_into_iter.next(), Ok(None));
    /// ```
    #[inline]
    fn map_into_iter<O, F>(self, f: F) -> MapIntoIter<Self, O, F>
    where
        Self: Sized,
        F: FnMut(FallibleLend<'_, Self>) -> Result<O, Self::Error>
    {
        MapIntoIter::new(self, f)
    }

    /// Calls the given function with each lend of this lender.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>(
    ///     [1, 2, 3].iter().into_fallible(),
    /// );
    /// let mut sum = 0u8;
    /// lender.for_each(|&x| { sum += x; Ok(()) }).unwrap();
    /// assert_eq!(sum, 6);
    /// ```
    #[inline]
    fn for_each<F>(self, mut f: F) -> Result<(), Self::Error>
    where
        Self: Sized,
        F: FnMut(FallibleLend<'_, Self>) -> Result<(), Self::Error>,
    {
        self.fold((), |(), item| {
            f(item)?;
            Ok(())
        })
    }

    /// Filters this lender using the given predicate.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>(
    ///     [1, 2, 3].iter().into_fallible(),
    /// );
    /// let mut filtered = lender.filter(|&&x| Ok(x > 1));
    /// assert_eq!(filtered.next(), Ok(Some(&2)));
    /// assert_eq!(filtered.next(), Ok(Some(&3)));
    /// assert_eq!(filtered.next(), Ok(None));
    /// ```
    #[inline]
    fn filter<P>(self, predicate: P) -> Filter<Self, P>
    where
        Self: Sized,
        P: FnMut(&FallibleLend<'_, Self>) -> Result<bool, Self::Error>,
    {
        Filter::new(self, predicate)
    }

    /// Filters and maps this lender using the given function.
    ///
    /// Note that functions passed to this method must be built using the
    /// [`covar!`](crate::covar) or [`covar_mut!`](crate::covar_mut) macros, which also
    /// checks for covariance of the returned type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// # use core::convert::Infallible;
    /// let mut data = [1, 2];
    /// let lender = lender::lend_fallible_iter::<fallible_lend!(&'lend mut u8), _>(
    ///     data.iter_mut().into_fallible(),
    /// );
    /// let mut filtered = lender.filter_map(covar_mut!(
    ///     for<'all> |a: &'all mut u8| -> Result<Option<&'all u8>, Infallible> {
    ///         if *a > 1 { Ok(Some(&*a)) } else { Ok(None) }
    ///     }
    /// ));
    /// assert_eq!(filtered.next().unwrap(), Some(&2));
    /// ```
    #[inline]
    fn filter_map<F>(self, f: Covar<F>) -> FilterMap<Self, F>
    where
        Self: Sized,
        F: for<'all> FnMutHKAResOpt<'all, FallibleLend<'all, Self>, Self::Error>,
    {
        FilterMap::new(self, f)
    }

    /// Enumerates this lender. Each lend is paired with its zero-based
    /// index.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>(
    ///     [1, 2, 3].iter().into_fallible(),
    /// );
    /// let mut enumerated = lender.enumerate();
    /// assert_eq!(enumerated.next(), Ok(Some((0, &1))));
    /// assert_eq!(enumerated.next(), Ok(Some((1, &2))));
    /// assert_eq!(enumerated.next(), Ok(Some((2, &3))));
    /// assert_eq!(enumerated.next(), Ok(None));
    /// ```
    #[inline]
    fn enumerate(self) -> Enumerate<Self>
    where
        Self: Sized,
    {
        Enumerate::new(self)
    }

    /// Makes this lender peekable, so that it is possible to peek at the next
    /// lend without consuming it.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>(
    ///     [1, 2].iter().into_fallible(),
    /// );
    /// let mut peekable = lender.peekable();
    /// assert_eq!(peekable.peek(), Ok(Some(&&1)));
    /// assert_eq!(peekable.next(), Ok(Some(&1)));
    /// assert_eq!(peekable.peek(), Ok(Some(&&2)));
    /// assert_eq!(peekable.next(), Ok(Some(&2)));
    /// assert_eq!(peekable.peek(), Ok(None));
    /// ```
    #[inline]
    fn peekable<'call>(self) -> FalliblePeekable<'call, Self>
    where
        Self: Sized,
    {
        FalliblePeekable::new(self)
    }

    /// Skips the first contiguous sequence of lends of this lender that
    /// satisfy the given predicate.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>(
    ///     [1, 2, 3].iter().into_fallible(),
    /// );
    /// let mut skipped = lender.skip_while(|&&x| Ok(x < 3));
    /// assert_eq!(skipped.next(), Ok(Some(&3)));
    /// assert_eq!(skipped.next(), Ok(None));
    /// ```
    #[inline]
    fn skip_while<P>(self, predicate: P) -> SkipWhile<Self, P>
    where
        Self: Sized,
        P: FnMut(&FallibleLend<'_, Self>) -> Result<bool, Self::Error>,
    {
        SkipWhile::new(self, predicate)
    }

    /// Takes the first contiguous sequence of lends of this lender that
    /// satisfy the given predicate.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>(
    ///     [1, 2, 3].iter().into_fallible(),
    /// );
    /// let mut taken = lender.take_while(|&&x| Ok(x < 3));
    /// assert_eq!(taken.next(), Ok(Some(&1)));
    /// assert_eq!(taken.next(), Ok(Some(&2)));
    /// assert_eq!(taken.next(), Ok(None));
    /// ```
    #[inline]
    fn take_while<P>(self, predicate: P) -> TakeWhile<Self, P>
    where
        Self: Sized,
        P: FnMut(&FallibleLend<'_, Self>) -> Result<bool, Self::Error>,
    {
        TakeWhile::new(self, predicate)
    }

    /// Maps this lender using the given function while it returns [`Some`].
    ///
    /// Note that functions passed to this method must be built using the
    /// [`covar!`](crate::covar) or [`covar_mut!`](crate::covar_mut) macros, which also
    /// checks for covariance of the returned type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// # use core::convert::Infallible;
    /// let mut data = [1, 2];
    /// let lender = lender::lend_fallible_iter::<fallible_lend!(&'lend mut u8), _>(
    ///     data.iter_mut().into_fallible(),
    /// );
    /// let mut mapped = lender.map_while(covar_mut!(
    ///     for<'all> |a: &'all mut u8| -> Result<Option<&'all u8>, Infallible> {
    ///         if *a < 2 { Ok(Some(&*a)) } else { Ok(None) }
    ///     }
    /// ));
    /// assert_eq!(mapped.next().unwrap(), Some(&1));
    /// assert_eq!(mapped.next().unwrap(), None);
    /// ```
    #[inline]
    fn map_while<P>(self, predicate: Covar<P>) -> MapWhile<Self, P>
    where
        Self: Sized,
        P: for<'all> FnMutHKAResOpt<'all, FallibleLend<'all, Self>, Self::Error>,
    {
        MapWhile::new(self, predicate)
    }

    /// Skips the first `n` lends of this lender.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>(
    ///     [1, 2, 3].iter().into_fallible(),
    /// );
    /// let mut skipped = lender.skip(2);
    /// assert_eq!(skipped.next(), Ok(Some(&3)));
    /// assert_eq!(skipped.next(), Ok(None));
    /// ```
    #[inline]
    fn skip(self, n: usize) -> Skip<Self>
    where
        Self: Sized,
    {
        Skip::new(self, n)
    }

    /// Takes the first `n` lends of this lender.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>(
    ///     [1, 2, 3].iter().into_fallible(),
    /// );
    /// let mut taken = lender.take(2);
    /// assert_eq!(taken.next(), Ok(Some(&1)));
    /// assert_eq!(taken.next(), Ok(Some(&2)));
    /// assert_eq!(taken.next(), Ok(None));
    /// ```
    #[inline]
    fn take(self, n: usize) -> Take<Self>
    where
        Self: Sized,
    {
        Take::new(self, n)
    }

    /// The [`FallibleLender`] version of [`Iterator::scan`].
    ///
    /// Note that functions passed to this method must be built using the
    /// [`covar!`](crate::covar) or [`covar_mut!`](crate::covar_mut) macros, which also
    /// checks for covariance of the returned type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// # use core::convert::Infallible;
    /// let lender = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>(
    ///     [1, 2, 3].iter().into_fallible(),
    /// );
    /// let mut scanned = lender.scan(
    ///     0,
    ///     covar_mut!(
    ///         for<'all> |args: (&'all mut u8, &'all u8)|
    ///             -> Result<Option<&'all u8>, Infallible>
    ///         {
    ///             *args.0 += *args.1;
    ///             Ok(Some(args.1))
    ///         }
    ///     ),
    /// );
    /// assert_eq!(scanned.next().unwrap(), Some(&1));
    /// assert_eq!(scanned.next().unwrap(), Some(&2));
    /// ```
    #[inline]
    fn scan<St, F>(self, initial_state: St, f: Covar<F>) -> Scan<Self, St, F>
    where
        Self: Sized,
        F: for<'all> FnMutHKAResOpt<'all, (&'all mut St, FallibleLend<'all, Self>), Self::Error>,
    {
        Scan::new(self, initial_state, f)
    }

    /// The [`FallibleLender`] version of [`Iterator::flat_map`].
    ///
    /// Note that functions passed to this method must be built using the
    /// [`covar!`](crate::covar) or [`covar_mut!`](crate::covar_mut) macros, which also
    /// checks for covariance of the returned type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lender::prelude::*;
    /// # use core::convert::Infallible;
    /// // Define a wrapper that implements FallibleLender
    /// struct VecLender(Vec<i32>);
    ///
    /// impl<'lend> FallibleLending<'lend> for VecLender {
    ///     type Lend = i32;
    /// }
    ///
    /// impl FallibleLender for VecLender {
    ///     type Error = Infallible;
    ///     check_covariance_fallible!();
    ///     fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
    ///         Ok(if self.0.is_empty() { None } else { Some(self.0.remove(0)) })
    ///     }
    /// }
    ///
    /// let data = [1, 2, 3];
    /// let mut flat = data.into_iter().into_lender().into_fallible().flat_map(
    ///     covar_mut!(for<'lend> |x: i32| -> Result<VecLender, Infallible> {
    ///         Ok(VecLender(vec![x, x * 10]))
    ///     })
    /// );
    /// assert_eq!(flat.next().unwrap(), Some(1));
    /// assert_eq!(flat.next().unwrap(), Some(10));
    /// assert_eq!(flat.next().unwrap(), Some(2));
    /// ```
    #[inline]
    fn flat_map<'call, F>(self, f: Covar<F>) -> FallibleFlatMap<'call, Self, F>
    where
        Self: Sized,
        F: for<'all> FnMutHKARes<'all, FallibleLend<'all, Self>, Self::Error>,
        for<'all> <F as FnMutHKARes<'all, FallibleLend<'all, Self>, Self::Error>>::B: IntoFallibleLender<Error = Self::Error>,
    {
        FallibleFlatMap::new(self, f)
    }

    /// The [`FallibleLender`] version of [`Iterator::flatten`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lender::prelude::*;
    /// # use core::convert::Infallible;
    /// let data = [vec![1, 2], vec![3, 4]];
    /// let lender = data.into_iter().into_lender()
    ///     .into_fallible();
    /// let mut flat = lender.map(covar!(
    ///     for<'all> |v: Vec<i32>|
    ///         -> Result<lender::IntoFallible<lender::FromIter<
    ///             std::vec::IntoIter<i32>
    ///         >>, Infallible>
    ///     {
    ///         Ok(v.into_iter().into_lender().into_fallible())
    ///     }
    /// )).flatten();
    /// assert_eq!(flat.next().unwrap(), Some(1));
    /// assert_eq!(flat.next().unwrap(), Some(2));
    /// assert_eq!(flat.next().unwrap(), Some(3));
    /// assert_eq!(flat.next().unwrap(), Some(4));
    /// assert_eq!(flat.next().unwrap(), None);
    /// ```
    #[inline]
    fn flatten<'call>(self) -> FallibleFlatten<'call, Self>
    where
        Self: Sized,
        for<'all> FallibleLend<'all, Self>: IntoFallibleLender<Error = Self::Error>,
    {
        FallibleFlatten::new(self)
    }

    /// The [`FallibleLender`] version of [`Iterator::fuse`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>(
    ///     [1, 2].iter().into_fallible(),
    /// );
    /// let mut fused = lender.fuse();
    /// assert_eq!(fused.next(), Ok(Some(&1)));
    /// assert_eq!(fused.next(), Ok(Some(&2)));
    /// assert_eq!(fused.next(), Ok(None));
    /// assert_eq!(fused.next(), Ok(None));
    /// ```
    #[inline]
    fn fuse(self) -> Fuse<Self>
    where
        Self: Sized,
    {
        Fuse::new(self)
    }

    /// The [`FallibleLender`] version of [`Iterator::inspect`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>(
    ///     [1, 2, 3].iter().into_fallible(),
    /// );
    /// let mut sum = 0;
    /// let mut inspected = lender.inspect(|&x| { sum += x; Ok(()) });
    /// assert_eq!(inspected.next(), Ok(Some(&1)));
    /// assert_eq!(inspected.next(), Ok(Some(&2)));
    /// assert_eq!(inspected.next(), Ok(Some(&3)));
    /// assert_eq!(inspected.next(), Ok(None));
    /// assert_eq!(sum, 6);
    /// ```
    #[inline]
    fn inspect<F>(self, f: F) -> Inspect<Self, F>
    where
        Self: Sized,
        F: FnMut(&FallibleLend<'_, Self>) -> Result<(), Self::Error>,
    {
        Inspect::new(self, f)
    }

    // not std::iter
    /// Mutates each lend with the given function.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let mut data = [1, 2, 3];
    /// let lender = lender::lend_fallible_iter::<fallible_lend!(&'lend mut u8), _>(
    ///     data.iter_mut().into_fallible(),
    /// );
    /// let mut mutated = lender.mutate(|x| { **x += 10; Ok(()) });
    /// assert_eq!(mutated.next(), Ok(Some(&mut 11)));
    /// assert_eq!(mutated.next(), Ok(Some(&mut 12)));
    /// assert_eq!(mutated.next(), Ok(Some(&mut 13)));
    /// assert_eq!(mutated.next(), Ok(None));
    /// ```
    #[inline]
    fn mutate<F>(self, f: F) -> Mutate<Self, F>
    where
        Self: Sized,
        F: FnMut(&mut FallibleLend<'_, Self>) -> Result<(), Self::Error>,
    {
        Mutate::new(self, f)
    }

    /// The [`FallibleLender`] version of
    /// [`Iterator::by_ref`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lender::prelude::*;
    /// let data = [1, 2, 3, 4, 5];
    /// let mut lender =
    ///     data.iter().into_lender().into_fallible();
    /// // Consume two elements through by_ref.
    /// assert_eq!(
    ///     lender.by_ref().next(),
    ///     Ok(Some(&1))
    /// );
    /// assert_eq!(
    ///     lender.by_ref().next(),
    ///     Ok(Some(&2))
    /// );
    /// // The original lender continues from where
    /// // we left off.
    /// assert_eq!(lender.next(), Ok(Some(&3)));
    /// ```
    #[inline(always)]
    fn by_ref(&mut self) -> &mut Self
    where
        Self: Sized,
    {
        self
    }

    /// Transforms the fallible lender into a collection.
    /// If any invocation of next returns Err, returns the collection built
    /// from values yielded successfully, together with the error.
    ///
    /// # Error Handling
    ///
    /// The return type `Result<B, (B, Self::Error)>` preserves partial
    /// results even on error: the `B` inside the `Err` variant contains
    /// whatever was accumulated before the error occurred.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::{IteratorExt as _, FallibleIterator as _};
    /// # use lender::prelude::*;
    /// let lender = [1, 2, 3].into_iter().into_lender().into_fallible();
    /// let collected: Result<Vec<i32>, _> = lender.owned().collect();
    /// assert_eq!(collected, Ok(vec![1, 2, 3]));
    /// ```
    #[inline]
    fn collect<B>(self) -> Result<B, (B, Self::Error)>
    where
        Self: Sized,
        for<'all> B: FromLender<NonFallibleAdapter<'all, Self>>,
    {
        non_fallible_adapter::process(self, |lender| B::from_lender(lender))
    }

    /// Collects the lends of a fallible lender that are themselves [`Try`] types
    /// into a collection of their outputs, short-circuiting on both lender errors
    /// and `Try` failures.
    ///
    /// The [`FallibleLender`] version of
    /// [`Lender::try_collect`](crate::Lender::try_collect).
    ///
    /// On error, returns the collection (with partial results) together with
    /// the error.
    #[inline]
    fn try_collect<'a, B, T>(&'a mut self) -> Result<T, (T, Self::Error)>
    where
        Self: Sized,
        for<'all> FallibleLend<'all, Self>: Try,
        for<'all> <FallibleLend<'all, Self> as Try>::Residual: Residual<B, TryType = T>,
        for<'b, 'c, 'd> B: FromLender<FallibleTryShuntAdapter<'b, 'c, 'd, 'a, Self>>,
    {
        non_fallible_adapter::process(self, |mut lender| {
            crate::Lender::try_collect(&mut lender)
        })
    }

    /// Extends an existing collection with lends from this fallible lender.
    ///
    /// The [`FallibleLender`] version of
    /// [`Lender::collect_into`](crate::Lender::collect_into).
    /// On error, returns the collection (with partial results) together with
    /// the error.
    #[inline]
    fn collect_into<E>(self, collection: &mut E) -> Result<&mut E, (&mut E, Self::Error)>
    where
        Self: Sized,
        for<'all> E: ExtendLender<NonFallibleAdapter<'all, Self>>,
    {
        match non_fallible_adapter::process(self, |lender|
            collection.extend_lender(lender)
        ) {
            Ok(()) => Ok(collection),
            Err(((), err)) => Err((collection, err))
        }
    }

    /// The [`FallibleLender`] version of [`Iterator::partition`].
    ///
    /// On error, returns the partial results accumulated so far together
    /// with the error, matching the convention used by
    /// [`collect`](FallibleLender::collect).
    #[inline]
    #[allow(clippy::type_complexity)]
    fn partition<'this, E, F>(mut self, mut f: F) -> Result<(E, E), ((E, E), Self::Error)>
    where
        Self: Sized + 'this,
        E: Default + ExtendLender<NonFallibleAdapter<'this, Self>>,
        F: FnMut(&FallibleLend<'_, Self>) -> Result<bool, Self::Error>
    {
        let mut left = E::default();
        let mut right = E::default();
        loop {
            match self.next() {
                Ok(Some(x)) => match f(&x) {
                    Ok(true) => left.extend_lender_one(x),
                    Ok(false) => right.extend_lender_one(x),
                    Err(e) => return Err(((left, right), e)),
                },
                Ok(None) => return Ok((left, right)),
                Err(e) => return Err(((left, right), e)),
            }
        }
    }

    /// The [`FallibleLender`] version of [`Iterator::is_partitioned`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// // [1, 3, 2, 4] is partitioned: all odd elements come before all even.
    /// let mut lender = lender::lend_fallible_iter::<fallible_lend!(&'lend i32), _>(
    ///     [1, 3, 2, 4].iter().into_fallible(),
    /// );
    /// assert!(lender.is_partitioned(|&x| Ok(x % 2 != 0)).unwrap());
    /// ```
    #[inline]
    #[allow(clippy::wrong_self_convention)]
    fn is_partitioned<P>(mut self, mut predicate: P) -> Result<bool, Self::Error>
    where
        Self: Sized,
        P: FnMut(FallibleLend<'_, Self>) -> Result<bool, Self::Error>,
    {
        Ok(self.all(&mut predicate)? || !self.any(predicate)?)
    }

    /// The [`FallibleLender`] version of [`Iterator::try_fold`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>(
    ///     [1, 2, 3].iter().into_fallible(),
    /// );
    /// let sum = lender.try_fold(0u8, |acc, &x| Ok(acc.checked_add(x)));
    /// assert_eq!(sum, Ok(Some(6)));
    /// ```
    #[inline]
    fn try_fold<B, F, R>(&mut self, mut init: B, mut f: F) -> Result<R, Self::Error>
    where
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: Try<Output = B>
    {
        while let Some(v) = self.next()? {
            match f(init, v)?.branch() {
                ControlFlow::Break(residual) => return Ok(R::from_residual(residual)),
                ControlFlow::Continue(output) => init = output
            }
        }
        Ok(R::from_output(init))
    }

    /// The [`FallibleLender`] version of [`Iterator::try_for_each`].
    #[inline]
    fn try_for_each<F, R>(&mut self, mut f: F) -> Result<R, Self::Error>
    where
        F: FnMut(FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: Try<Output = ()>,
    {
        self.try_fold((), move |(), x| f(x))
    }

    /// The [`FallibleLender`] version of [`Iterator::fold`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>(
    ///     [1, 2, 3].iter().into_fallible(),
    /// );
    /// let sum = lender.fold(0, |acc, &x| Ok(acc + x));
    /// assert_eq!(sum, Ok(6));
    /// ```
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// // Example concatenating strings
    /// let data = ["hello", " ", "world"];
    /// let lender = lender::lend_fallible_iter::<fallible_lend!(&&'lend str), _>(
    ///     data.iter().into_fallible(),
    /// );
    /// let result = lender.fold(String::new(), |mut acc, &s| {
    ///     acc.push_str(s);
    ///     Ok(acc)
    /// });
    /// assert_eq!(result, Ok(String::from("hello world")));
    /// ```
    #[inline]
    fn fold<B, F>(mut self, init: B, mut f: F) -> Result<B, Self::Error>
    where
        Self: Sized,
        F: FnMut(B, FallibleLend<'_, Self>) -> Result<B, Self::Error>,
    {
        self.try_fold(init, |acc, item| f(acc, item).map(NeverShortCircuit))
            .map(|res| res.0)
    }

    /// The [`FallibleLender`] version of [`Iterator::reduce`].
    ///
    /// # Error Handling
    ///
    /// If the closure or the underlying lender returns an error, `reduce`
    /// propagates it immediately. Unlike [`collect`](FallibleLender::collect),
    /// the partial accumulation is not preserved on error.
    #[inline]
    fn reduce<T, F>(mut self, f: F) -> Result<Option<T>, Self::Error>
    where
        Self: Sized,
        for<'all> FallibleLend<'all, Self>: ToOwned<Owned = T>,
        F: FnMut(T, FallibleLend<'_, Self>) -> Result<T, Self::Error>
    {
        let Some(first) = self.next()?.map(|first| first.to_owned()) else {
            return Ok(None)
        };
        self.fold(first, f).map(Some)
    }

    /// The [`FallibleLender`] version of [`Iterator::try_reduce`].
    #[inline]
    fn try_reduce<T, F, R>(mut self, f: F) -> Result<ChangeOutputType<R, Option<T>>, Self::Error>
    where
        Self: Sized,
        for<'all> FallibleLend<'all, Self>: ToOwned<Owned = T>,
        F: FnMut(T, FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: Try<Output = T>,
        R::Residual: Residual<Option<T>>,
    {
        let first = match self.next()? {
            Some(ref x) => x.to_owned(),
            None => return Ok(Try::from_output(None)),
        };
        match self.try_fold(first, f)?.branch() {
            ControlFlow::Break(x) => Ok(FromResidual::from_residual(x)),
            ControlFlow::Continue(x) => Ok(Try::from_output(Some(x))),
        }
    }

    /// The [`FallibleLender`] version of [`Iterator::all`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>(
    ///     [1, 2, 3].iter().into_fallible(),
    /// );
    /// assert_eq!(lender.all(|&x| Ok(x > 0)), Ok(true));
    /// ```
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>(
    ///     [1, 2, 3].iter().into_fallible(),
    /// );
    /// assert_eq!(lender.all(|&x| Ok(x > 2)), Ok(false));
    /// ```
    #[inline]
    fn all<F>(&mut self, mut f: F) -> Result<bool, Self::Error>
    where
        F: FnMut(FallibleLend<'_, Self>) -> Result<bool, Self::Error>,
    {
        while let Some(x) = self.next()? {
            if !f(x)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// The [`FallibleLender`] version of [`Iterator::any`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>(
    ///     [1, 2, 3].iter().into_fallible(),
    /// );
    /// assert_eq!(lender.any(|&x| Ok(x == 2)), Ok(true));
    /// ```
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>(
    ///     [1, 2, 3].iter().into_fallible(),
    /// );
    /// assert_eq!(lender.any(|&x| Ok(x > 5)), Ok(false));
    /// ```
    #[inline]
    fn any<F>(&mut self, mut f: F) -> Result<bool, Self::Error>
    where
        F: FnMut(FallibleLend<'_, Self>) -> Result<bool, Self::Error>,
    {
        while let Some(x) = self.next()? {
            if f(x)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// The [`FallibleLender`] version of [`Iterator::find`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>(
    ///     [1, 2, 3].iter().into_fallible(),
    /// );
    /// assert_eq!(lender.find(|&&x| Ok(x == 2)), Ok(Some(&2)));
    /// ```
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>(
    ///     [1, 2, 3].iter().into_fallible(),
    /// );
    /// assert_eq!(lender.find(|&&x| Ok(x > 10)), Ok(None));
    /// ```
    #[inline]
    fn find<P>(&mut self, mut predicate: P) -> Result<Option<FallibleLend<'_, Self>>, Self::Error>
    where
        P: FnMut(&FallibleLend<'_, Self>) -> Result<bool, Self::Error>,
    {
        while let Some(x) = self.next()? {
            if predicate(&x)? {
                // SAFETY: polonius return
                return Ok(Some(unsafe {
                    core::mem::transmute::<
                        FallibleLend<'_, Self>,
                        FallibleLend<'_, Self>
                    >(x)
                }));
            }
        }
        Ok(None)
    }

    /// The [`FallibleLender`] version of [`Iterator::find_map`].
    #[allow(clippy::type_complexity)]
    #[inline]
    fn find_map<'a, F>(&'a mut self, mut f: F) -> Result<Option<<F as FnMutHKAResOpt<'a, FallibleLend<'a, Self>, Self::Error>>::B>, Self::Error>
    where
        Self: Sized,
        F: for<'all> FnMutHKAResOpt<'all, FallibleLend<'all, Self>, Self::Error>,
    {
        while let Some(x) = self.next()? {
            if let Some(y) = f(x)? {
                return Ok(Some(
                    // SAFETY: polonius return
                    unsafe {
                        core::mem::transmute::<
                            <F as FnMutHKAResOpt<'_, FallibleLend<'_, Self>, Self::Error>>::B,
                            <F as FnMutHKAResOpt<'a, FallibleLend<'a, Self>, Self::Error>>::B
                        >(y)
                    }
                ));
            }
        }
        Ok(None)
    }

    /// The [`FallibleLender`] version of [`Iterator::try_find`].
    #[inline]
    fn try_find<F, R>(&mut self, mut f: F) -> Result<ChangeOutputType<R, Option<FallibleLend<'_, Self>>>, Self::Error>
    where
        Self: Sized,
        F: FnMut(&FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: Try<Output = bool>,
        for<'all> R::Residual: Residual<Option<FallibleLend<'all, Self>>>,
    {
        while let Some(x) = self.next()? {
            match f(&x)?.branch() {
                ControlFlow::Break(x) => return Ok(<ChangeOutputType<R, Option<FallibleLend<'_, Self>>>>::from_residual(x)),
                ControlFlow::Continue(cond) => {
                    if cond {
                        return Ok(<ChangeOutputType<R, Option<FallibleLend<'_, Self>>>>::from_output(
                            Some(
                                // SAFETY: polonius return
                                unsafe {
                                    core::mem::transmute::<
                                        FallibleLend<'_, Self>,
                                        FallibleLend<'_, Self>
                                    >(x)
                                }
                            )
                        ));
                    }
                }
            }
        }
        Ok(<ChangeOutputType<R, Option<FallibleLend<'_, Self>>>>::from_output(None))
    }

    /// The [`FallibleLender`] version of [`Iterator::position`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>(
    ///     [1, 2, 3].iter().into_fallible(),
    /// );
    /// assert_eq!(lender.position(|&x| Ok(x == 2)), Ok(Some(1)));
    /// ```
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>(
    ///     [1, 2, 3].iter().into_fallible(),
    /// );
    /// assert_eq!(lender.position(|&x| Ok(x > 10)), Ok(None));
    /// ```
    #[inline]
    fn position<P>(&mut self, mut predicate: P) -> Result<Option<usize>, Self::Error>
    where
        Self: Sized,
        P: FnMut(FallibleLend<'_, Self>) -> Result<bool, Self::Error>,
    {
        let mut i = 0;
        while let Some(x) = self.next()? {
            if predicate(x)? {
                return Ok(Some(i));
            }
            i += 1;
        }
        Ok(None)
    }

    /// The [`FallibleLender`] version of [`Iterator::rposition`].
    #[inline]
    fn rposition<P>(&mut self, mut predicate: P) -> Result<Option<usize>, Self::Error>
    where
        P: FnMut(FallibleLend<'_, Self>) -> Result<bool, Self::Error>,
        Self: Sized + ExactSizeFallibleLender + DoubleEndedFallibleLender,
    {
        match self.try_rfold(self.len(), |i, x| {
            let i = i - 1;
            if predicate(x)? { Ok(ControlFlow::Break(i)) } else { Ok(ControlFlow::Continue(i)) }
        })? {
            ControlFlow::Continue(_) => Ok(None),
            ControlFlow::Break(x) => Ok(Some(x)),
        }
    }

    /// The [`FallibleLender`] version of [`Iterator::max`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lender::prelude::*;
    /// let lender =
    ///     [1, 3, 2].into_iter().into_lender().into_fallible();
    /// assert_eq!(lender.max(), Ok(Some(3)));
    /// ```
    #[inline]
    fn max<T: Ord>(self) -> Result<Option<T>, Self::Error>
    where
        Self: Sized,
        for<'all> FallibleLend<'all, Self>: ToOwned<Owned = T>,
    {
        fallible_iterator::FallibleIterator::max(self.owned())
    }

    /// The [`FallibleLender`] version of [`Iterator::min`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lender::prelude::*;
    /// let lender =
    ///     [3, 1, 2].into_iter().into_lender().into_fallible();
    /// assert_eq!(lender.min(), Ok(Some(1)));
    /// ```
    #[inline]
    fn min<T: Ord>(self) -> Result<Option<T>, Self::Error>
    where
        Self: Sized,
        for<'all> FallibleLend<'all, Self>: ToOwned<Owned = T>,
    {
        fallible_iterator::FallibleIterator::min(self.owned())
    }

    /// The [`FallibleLender`] version of [`Iterator::max_by_key`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lender::prelude::*;
    /// let lender =
    ///     ["a", "bbb", "cc"].into_iter().into_lender().into_fallible();
    /// assert_eq!(lender.max_by_key(|s| Ok(s.len())), Ok(Some("bbb")));
    /// ```
    #[inline]
    fn max_by_key<B: Ord, T, F>(self, f: F) -> Result<Option<T>, Self::Error>
    where
        Self: Sized,
        for<'all> FallibleLend<'all, Self>: ToOwned<Owned = T>,
        F: FnMut(&T) -> Result<B, Self::Error>,
    {
        fallible_iterator::FallibleIterator::max_by_key(self.owned(), f)
    }

    /// The [`FallibleLender`] version of [`Iterator::max_by`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lender::prelude::*;
    /// let lender =
    ///     ["a", "bbb", "cc"].into_iter().into_lender().into_fallible();
    /// assert_eq!(
    ///     lender.max_by(|x, y| Ok(x.len().cmp(&y.len()))),
    ///     Ok(Some("bbb"))
    /// );
    /// ```
    #[inline]
    fn max_by<T, F>(self, mut compare: F) -> Result<Option<T>, Self::Error>
    where
        Self: Sized,
        for<'all> FallibleLend<'all, Self>: ToOwned<Owned = T>,
        F: FnMut(&T, &FallibleLend<'_, Self>) -> Result<Ordering, Self::Error>
    {
        self.reduce(move |x, y| Ok(
            match compare(&x, &y)? {
                Ordering::Greater => x,
                _ => y.to_owned(),
            }
        ))
    }

    /// The [`FallibleLender`] version of [`Iterator::min_by_key`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lender::prelude::*;
    /// let lender =
    ///     ["a", "bbb", "cc"].into_iter().into_lender().into_fallible();
    /// assert_eq!(lender.min_by_key(|s| Ok(s.len())), Ok(Some("a")));
    /// ```
    #[inline]
    fn min_by_key<B: Ord, T, F>(self, f: F) -> Result<Option<T>, Self::Error>
    where
        Self: Sized,
        for<'all> FallibleLend<'all, Self>: ToOwned<Owned = T>,
        F: FnMut(&T) -> Result<B, Self::Error>,
    {
        fallible_iterator::FallibleIterator::min_by_key(self.owned(), f)
    }

    /// The [`FallibleLender`] version of [`Iterator::min_by`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lender::prelude::*;
    /// let lender =
    ///     ["a", "bbb", "cc"].into_iter().into_lender().into_fallible();
    /// assert_eq!(
    ///     lender.min_by(|x, y| Ok(x.len().cmp(&y.len()))),
    ///     Ok(Some("a"))
    /// );
    /// ```
    #[inline]
    fn min_by<T, F>(self, mut compare: F) -> Result<Option<T>, Self::Error>
    where
        Self: Sized,
        for<'all> FallibleLend<'all, Self>: ToOwned<Owned = T>,
        F: FnMut(&T, &FallibleLend<'_, Self>) -> Result<Ordering, Self::Error>,
    {
        self.reduce(move |x, y| Ok(
            match compare(&x, &y)? {
                Ordering::Greater => y.to_owned(),
                _ => x,
            }
        ))
    }

    /// The [`FallibleLender`] version of [`Iterator::rev`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>(
    ///     [1, 2, 3].iter().into_fallible(),
    /// )
    /// .rev();
    /// assert_eq!(lender.next(), Ok(Some(&3)));
    /// assert_eq!(lender.next(), Ok(Some(&2)));
    /// assert_eq!(lender.next(), Ok(Some(&1)));
    /// assert_eq!(lender.next(), Ok(None));
    /// ```
    #[inline]
    fn rev(self) -> Rev<Self>
    where
        Self: Sized + DoubleEndedFallibleLender,
    {
        Rev::new(self)
    }

    /// The [`FallibleLender`] version of [`Iterator::unzip`].
    #[inline]
    fn unzip<ExtA, ExtB>(self) -> Result<(ExtA, ExtB), Self::Error>
    where
        Self: Sized,
        for<'all> FallibleLend<'all, Self>: TupleLend<'all>,
        ExtA: Default + for<'this> ExtendLender<NonFallibleAdapter<'this,
            <FirstShunt<Self> as IntoFallibleLender>::FallibleLender,
        >>,
        ExtB: Default + for<'this> ExtendLender<NonFallibleAdapter<'this,
            <SecondShunt<Self> as IntoFallibleLender>::FallibleLender,
        >>, {
        fallible_unzip(self)
    }

    /// The [`FallibleLender`] version of [`Iterator::copied`].
    ///
    /// Turns this [`FallibleLender`] into a
    /// [`FallibleIterator`](fallible_iterator::FallibleIterator).
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::{IteratorExt as _, FallibleIterator};
    /// # use lender::prelude::*;
    /// let lender = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>(
    ///     [1, 2, 3].iter().into_fallible(),
    /// );
    /// let mut copied = lender.copied();
    /// assert_eq!(copied.next()?, Some(1));
    /// assert_eq!(copied.next()?, Some(2));
    /// assert_eq!(copied.next()?, Some(3));
    /// assert_eq!(copied.next()?, None);
    /// # Ok::<(), core::convert::Infallible>(())
    /// ```
    #[inline]
    fn copied<T>(self) -> Copied<Self>
    where
        Self: Sized + for<'all> FallibleLending<'all, Lend = &'all T>,
        T: Copy,
    {
        Copied::new(self)
    }

    /// The [`FallibleLender`] version of [`Iterator::cloned`].
    ///
    /// Turns this [`FallibleLender`] into a
    /// [`FallibleIterator`](fallible_iterator::FallibleIterator).
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::{IteratorExt as _, FallibleIterator};
    /// # use lender::prelude::*;
    /// let lender = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>(
    ///     [1, 2, 3].iter().into_fallible(),
    /// );
    /// let mut cloned = lender.cloned();
    /// assert_eq!(cloned.next()?, Some(1));
    /// assert_eq!(cloned.next()?, Some(2));
    /// assert_eq!(cloned.next()?, Some(3));
    /// assert_eq!(cloned.next()?, None);
    /// # Ok::<(), core::convert::Infallible>(())
    /// ```
    #[inline]
    fn cloned<T>(self) -> Cloned<Self>
    where
        Self: Sized + for<'all> FallibleLending<'all, Lend = &'all T>,
        T: Clone,
    {
        Cloned::new(self)
    }

    // not std::iter
    /// Turns this [`FallibleLender`] into a
    /// [`FallibleIterator`](fallible_iterator::FallibleIterator).
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::{IteratorExt as _, FallibleIterator};
    /// # use lender::prelude::*;
    /// let mut owned = [1, 2, 3].into_iter().into_lender().into_fallible().owned();
    /// assert_eq!(owned.next()?, Some(1));
    /// assert_eq!(owned.next()?, Some(2));
    /// assert_eq!(owned.next()?, Some(3));
    /// assert_eq!(owned.next()?, None);
    /// # Ok::<(), core::convert::Infallible>(())
    /// ```
    #[inline]
    fn owned(self) -> Owned<Self>
    where
        Self: Sized,
        for<'all> FallibleLend<'all, Self>: ToOwned
    {
        Owned::new(self)
    }

    /// The [`FallibleLender`] version of [`Iterator::cycle`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let mut cycled = [1, 2].iter().into_lender().into_fallible().cycle();
    /// assert_eq!(cycled.next(), Ok(Some(&1)));
    /// assert_eq!(cycled.next(), Ok(Some(&2)));
    /// assert_eq!(cycled.next(), Ok(Some(&1)));
    /// assert_eq!(cycled.next(), Ok(Some(&2)));
    /// ```
    #[inline]
    fn cycle(self) -> Cycle<Self>
    where
        Self: Sized + Clone,
    {
        Cycle::new(self)
    }

    /// The [`FallibleLender`] version of [`Iterator::sum`].
    ///
    /// Note: This method requires a type implementing
    /// [`SumFallibleLender`](crate::SumFallibleLender). For simple numeric
    /// sums, consider using [`owned()`](FallibleLender::owned) and then
    /// [`FallibleIterator::sum()`](https://docs.rs/fallible-iterator/latest/fallible_iterator/trait.FallibleIterator.html#method.sum).
    #[inline]
    fn sum<S>(self) -> Result<S, Self::Error>
    where
        Self: Sized,
        S: SumFallibleLender<Self>,
    {
        S::sum_lender(self)
    }

    /// The [`FallibleLender`] version of [`Iterator::product`].
    ///
    /// Note: This method requires a type implementing
    /// [`ProductFallibleLender`](crate::ProductFallibleLender). For simple
    /// numeric products, consider using [`owned()`](FallibleLender::owned)
    /// and then
    /// [`FallibleIterator::product()`](https://docs.rs/fallible-iterator/latest/fallible_iterator/trait.FallibleIterator.html#method.product).
    #[inline]
    fn product<P>(self) -> Result<P, Self::Error>
    where
        Self: Sized,
        P: ProductFallibleLender<Self>,
    {
        P::product_lender(self)
    }

    /// The [`FallibleLender`] version of [`Iterator::cmp`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// # use core::cmp::Ordering;
    /// let a = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>(
    ///     [1, 2].iter().into_fallible(),
    /// );
    /// let b = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>(
    ///     [1, 3].iter().into_fallible(),
    /// );
    /// assert_eq!(a.cmp(b), Ok(Ordering::Less));
    /// ```
    #[inline]
    fn cmp<L>(self, other: L) -> Result<Ordering, Self::Error>
    where
        L: IntoFallibleLender<Error = Self::Error>,
        L::FallibleLender: for<'all> FallibleLending<'all, Lend = FallibleLend<'all, Self>>,
        for <'all> FallibleLend<'all, Self>: Ord,
        Self: Sized,
    {
        self.cmp_by(other, |x, y| Ok(x.cmp(&y)))
    }

    /// The [`FallibleLender`] version of [`Iterator::cmp_by`].
    ///
    /// Note: the closure receives both lends under a single lifetime
    /// (`for<'all>`). This is an HRTB limitation: the two lenders cannot
    /// be advanced independently within the closure.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// # use core::cmp::Ordering;
    /// let a = [1, 2, 3].into_iter().into_lender().into_fallible();
    /// let b = [1, 2, 4].into_iter().into_lender().into_fallible();
    /// assert_eq!(
    ///     a.cmp_by::<lender::IntoFallible<_>, _>(b, |x: i32, y: i32| Ok(x.cmp(&y))),
    ///     Ok(Ordering::Less)
    /// );
    /// ```
    #[inline]
    fn cmp_by<L, F>(self, other: L, mut cmp: F) -> Result<Ordering, Self::Error>
    where
        Self: Sized,
        L: IntoFallibleLender<Error = Self::Error>,
        F: for<'all> FnMut(FallibleLend<'all, Self>, FallibleLend<'all, L::FallibleLender>) -> Result<Ordering, Self::Error>,
    {
        match lender_compare(self, other.into_fallible_lender(), move |x, y| match cmp(x, y)? {
            Ordering::Equal => Ok(ControlFlow::Continue(())),
            neq => Ok(ControlFlow::Break(neq)),
        })? {
            ControlFlow::Continue(ord) => Ok(ord),
            ControlFlow::Break(ord) => Ok(ord),
        }
    }

    /// The [`FallibleLender`] version of [`Iterator::partial_cmp`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// # use core::cmp::Ordering;
    /// let a = [1, 2].into_iter().into_lender().into_fallible();
    /// let b = [1, 3].into_iter().into_lender().into_fallible();
    /// assert_eq!(
    ///     a.partial_cmp::<lender::IntoFallible<_>>(b),
    ///     Ok(Some(Ordering::Less))
    /// );
    /// ```
    #[inline]
    fn partial_cmp<L>(self, other: L) -> Result<Option<Ordering>, Self::Error>
    where
        L: IntoFallibleLender<Error = Self::Error>,
        for<'all> FallibleLend<'all, Self>: PartialOrd<FallibleLend<'all, L::FallibleLender>>,
        Self: Sized,
    {
        self.partial_cmp_by(other, |x, y| Ok(x.partial_cmp(&y)))
    }

    /// The [`FallibleLender`] version of [`Iterator::partial_cmp_by`].
    ///
    /// See [`cmp_by`](FallibleLender::cmp_by) for a note on the unified
    /// lifetime constraint.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// # use core::cmp::Ordering;
    /// let a = [1.0, 2.0].into_iter().into_lender().into_fallible();
    /// let b = [1.0, 3.0].into_iter().into_lender().into_fallible();
    /// assert_eq!(
    ///     a.partial_cmp_by::<lender::IntoFallible<_>, _>(
    ///         b,
    ///         |x: f64, y: f64| Ok(x.partial_cmp(&y))
    ///     ),
    ///     Ok(Some(Ordering::Less))
    /// );
    /// ```
    #[inline]
    fn partial_cmp_by<L, F>(self, other: L, mut partial_cmp: F) -> Result<Option<Ordering>, Self::Error>
    where
        Self: Sized,
        L: IntoFallibleLender<Error = Self::Error>,
        F: for<'all> FnMut(FallibleLend<'all, Self>, FallibleLend<'all, L::FallibleLender>) -> Result<Option<Ordering>, Self::Error>,
    {
        match lender_compare(self, other.into_fallible_lender(), move |x, y| match partial_cmp(x, y)? {
            Some(Ordering::Equal) => Ok(ControlFlow::Continue(())),
            neq => Ok(ControlFlow::Break(neq)),
        })? {
            ControlFlow::Continue(ord) => Ok(Some(ord)),
            ControlFlow::Break(ord) => Ok(ord),
        }
    }

    /// The [`FallibleLender`] version of [`Iterator::eq`].
    #[inline]
    fn eq<L>(self, other: L) -> Result<bool, Self::Error>
    where
        L: IntoFallibleLender<Error = Self::Error>,
        for<'all> FallibleLend<'all, Self>: PartialEq<FallibleLend<'all, L::FallibleLender>>,
        Self: Sized,
    {
        self.eq_by(other, |x, y| Ok(x == y))
    }

    /// The [`FallibleLender`] version of [`Iterator::eq_by`].
    ///
    /// See [`cmp_by`](FallibleLender::cmp_by) for a note on the unified
    /// lifetime constraint.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let a = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>(
    ///     [1, 2].iter().into_fallible(),
    /// );
    /// let b = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>(
    ///     [1, 2].iter().into_fallible(),
    /// );
    /// assert_eq!(a.eq_by(b, |x, y| Ok(x == y)), Ok(true));
    /// ```
    #[inline]
    fn eq_by<L, F>(self, other: L, mut eq: F) -> Result<bool, Self::Error>
    where
        Self: Sized,
        L: IntoFallibleLender<Error = Self::Error>,
        F: for<'all> FnMut(FallibleLend<'all, Self>, FallibleLend<'all, L::FallibleLender>) -> Result<bool, Self::Error>,
    {
        match lender_compare(self, other.into_fallible_lender(), move |x, y| Ok(
            if eq(x, y)? { ControlFlow::Continue(()) } else { ControlFlow::Break(()) }
        ))? {
            ControlFlow::Continue(ord) => Ok(ord == Ordering::Equal),
            ControlFlow::Break(()) => Ok(false),
        }
    }

    /// The [`FallibleLender`] version of [`Iterator::ne`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let a = [1, 2].into_iter().into_lender().into_fallible();
    /// let b = [1, 3].into_iter().into_lender().into_fallible();
    /// assert_eq!(a.ne::<lender::IntoFallible<_>>(b), Ok(true));
    /// ```
    #[inline]
    fn ne<L>(self, other: L) -> Result<bool, Self::Error>
    where
        L: IntoFallibleLender<Error = Self::Error>,
        for<'all> FallibleLend<'all, Self>: PartialEq<FallibleLend<'all, L::FallibleLender>>,
        Self: Sized,
    {
        self.eq(other).map(|eq| !eq)
    }

    /// The [`FallibleLender`] version of [`Iterator::lt`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let a = [1, 2].into_iter().into_lender().into_fallible();
    /// let b = [1, 3].into_iter().into_lender().into_fallible();
    /// assert_eq!(a.lt::<lender::IntoFallible<_>>(b), Ok(true));
    /// ```
    #[inline]
    fn lt<L>(self, other: L) -> Result<bool, Self::Error>
    where
        L: IntoFallibleLender<Error = Self::Error>,
        for<'all> FallibleLend<'all, Self>: PartialOrd<FallibleLend<'all, L::FallibleLender>>,
        Self: Sized,
    {
        Ok(self.partial_cmp(other)? == Some(Ordering::Less))
    }

    /// The [`FallibleLender`] version of [`Iterator::le`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let a = [1, 2].into_iter().into_lender().into_fallible();
    /// let b = [1, 2].into_iter().into_lender().into_fallible();
    /// assert_eq!(a.le::<lender::IntoFallible<_>>(b), Ok(true));
    /// ```
    #[inline]
    fn le<L>(self, other: L) -> Result<bool, Self::Error>
    where
        L: IntoFallibleLender<Error = Self::Error>,
        for<'all> FallibleLend<'all, Self>: PartialOrd<FallibleLend<'all, L::FallibleLender>>,
        Self: Sized,
    {
        Ok(matches!(self.partial_cmp(other)?, Some(Ordering::Less | Ordering::Equal)))
    }

    /// The [`FallibleLender`] version of [`Iterator::gt`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let a = [1, 3].into_iter().into_lender().into_fallible();
    /// let b = [1, 2].into_iter().into_lender().into_fallible();
    /// assert_eq!(a.gt::<lender::IntoFallible<_>>(b), Ok(true));
    /// ```
    #[inline]
    fn gt<L>(self, other: L) -> Result<bool, Self::Error>
    where
        L: IntoFallibleLender<Error = Self::Error>,
        for<'all> FallibleLend<'all, Self>: PartialOrd<FallibleLend<'all, L::FallibleLender>>,
        Self: Sized,
    {
        Ok(self.partial_cmp(other)? == Some(Ordering::Greater))
    }

    /// The [`FallibleLender`] version of [`Iterator::ge`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let a = [1, 2].into_iter().into_lender().into_fallible();
    /// let b = [1, 2].into_iter().into_lender().into_fallible();
    /// assert_eq!(a.ge::<lender::IntoFallible<_>>(b), Ok(true));
    /// ```
    #[inline]
    fn ge<L>(self, other: L) -> Result<bool, Self::Error>
    where
        L: IntoFallibleLender<Error = Self::Error>,
        for<'all> FallibleLend<'all, Self>: PartialOrd<FallibleLend<'all, L::FallibleLender>>,
        Self: Sized,
    {
        Ok(matches!(self.partial_cmp(other)?, Some(Ordering::Greater | Ordering::Equal)))
    }

    /// The [`FallibleLender`] version of [`Iterator::is_sorted`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lender::prelude::*;
    /// let lender = [1, 2, 3].into_iter().into_lender().into_fallible();
    /// assert_eq!(lender.is_sorted(), Ok(true));
    ///
    /// let lender = [1, 3, 2].into_iter().into_lender().into_fallible();
    /// assert_eq!(lender.is_sorted(), Ok(false));
    /// ```
    #[inline]
    #[allow(clippy::wrong_self_convention)]
    fn is_sorted<T>(self) -> Result<bool, Self::Error>
    where
        Self: Sized,
        for<'all> FallibleLend<'all, Self>: ToOwned<Owned = T>,
        T: PartialOrd,
    {
        self.is_sorted_by(|x, y| Ok(PartialOrd::partial_cmp(x, y)))
    }

    /// The [`FallibleLender`] version of [`Iterator::is_sorted_by`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lender::prelude::*;
    /// // Sorted in descending order.
    /// let lender = [3, 2, 1].into_iter().into_lender().into_fallible();
    /// assert_eq!(
    ///     lender.is_sorted_by(|a, b| Ok(b.partial_cmp(a))),
    ///     Ok(true)
    /// );
    /// ```
    #[inline]
    #[allow(clippy::wrong_self_convention)]
    fn is_sorted_by<T, F>(self, mut compare: F) -> Result<bool, Self::Error>
    where
        Self: Sized,
        for<'all> FallibleLend<'all, Self>: ToOwned<Owned = T>,
        F: FnMut(&T, &T) -> Result<Option<Ordering>, Self::Error>,
    {
        use fallible_iterator::FallibleIterator;
        let mut this = self.owned();
        let Some(mut last) = this.next()? else {
            return Ok(true);
        };
        this.all(move |curr| {
            if let Some(Ordering::Greater) | None = compare(&last, &curr)? {
                return Ok(false);
            }
            last = curr;
            Ok(true)
        })
    }

    /// The [`FallibleLender`] version of [`Iterator::is_sorted_by_key`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lender::prelude::*;
    /// let lender =
    ///     ["a", "bbb", "cc"].into_iter().into_lender().into_fallible();
    /// assert_eq!(
    ///     lender.is_sorted_by_key(|s| Ok(s.len())),
    ///     Ok(false)
    /// );
    ///
    /// let lender =
    ///     ["a", "cc", "bbb"].into_iter().into_lender().into_fallible();
    /// assert_eq!(
    ///     lender.is_sorted_by_key(|s| Ok(s.len())),
    ///     Ok(true)
    /// );
    /// ```
    #[inline]
    #[allow(clippy::wrong_self_convention)]
    fn is_sorted_by_key<F, K>(mut self, mut f: F) -> Result<bool, Self::Error>
    where
        Self: Sized,
        F: FnMut(FallibleLend<'_, Self>) -> Result<K, Self::Error>,
        K: PartialOrd,
    {
        let mut last = match self.next()? {
            None => return Ok(true),
            Some(x) => f(x)?,
        };
        while let Some(x) = self.next()? {
            let curr = f(x)?;
            if let Some(Ordering::Greater) | None = last.partial_cmp(&curr) {
                return Ok(false);
            }
            last = curr;
        }
        Ok(true)
    }

    /// A lending replacement for [`Iterator::array_chunks`].
    ///
    /// This method returns a lender that yields lenders returning the next
    /// `chunk_size` lends.
    ///
    /// # Panics
    ///
    /// Panics if `chunk_size` is zero.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lender::prelude::*;
    /// # use std::convert::Infallible;
    /// let data = [1, 2, 3, 4, 5];
    /// let lender = data.iter().into_lender().into_fallible();
    /// let mut chunky = lender.chunky(2);
    /// let mut chunk1 = chunky.next().unwrap().unwrap();
    /// assert_eq!(chunk1.next().unwrap(), Some(&1));
    /// assert_eq!(chunk1.next().unwrap(), Some(&2));
    /// assert_eq!(chunk1.next().unwrap(), None);
    /// let mut chunk2 = chunky.next().unwrap().unwrap();
    /// assert_eq!(chunk2.next().unwrap(), Some(&3));
    /// assert_eq!(chunk2.next().unwrap(), Some(&4));
    /// assert_eq!(chunk2.next().unwrap(), None);
    /// let mut chunk3 = chunky.next().unwrap().unwrap();
    /// assert_eq!(chunk3.next().unwrap(), Some(&5));
    /// assert_eq!(chunk3.next().unwrap(), None);
    /// ```
    #[inline]
    fn chunky(self, chunk_size: usize) -> Chunky<Self>
    where
        Self: Sized + ExactSizeFallibleLender,
    {
        Chunky::new_fallible(self, chunk_size)
    }

    /// Turns this [`FallibleLender`] into a
    /// [`FallibleIterator`](fallible_iterator::FallibleIterator) where it has
    /// already fulfilled the requirements of the
    /// [`FallibleIterator`](fallible_iterator::FallibleIterator) trait.
    ///
    /// Note: Due to HRTB limitations, consider using
    /// [`copied`](FallibleLender::copied), [`cloned`](FallibleLender::cloned),
    /// or [`owned`](FallibleLender::owned) instead when applicable.
    #[inline]
    fn iter<'this>(self) -> Iter<'this, Self>
    where
        Self: Sized + 'this,
        for<'all> FallibleLend<'all, Self>: 'this,
    {
        Iter::new(self)
    }
}

#[inline]
pub(crate) fn lender_compare<A, B, F, T>(
    mut a: A,
    mut b: B,
    mut f: F,
) -> Result<ControlFlow<T, Ordering>, A::Error>
where
    A: FallibleLender,
    B: FallibleLender<Error = A::Error>,
    for<'all> F:
        FnMut(FallibleLend<'all, A>, FallibleLend<'all, B>) -> Result<ControlFlow<T>, A::Error>,
{
    let mut ctl = ControlFlow::Continue(());
    while let Some(x) = a.next()? {
        match b.next()? {
            None => {
                ctl = ControlFlow::Break(ControlFlow::Continue(Ordering::Greater));
                break;
            }
            Some(y) => {
                if let ControlFlow::Break(x) = f(x, y)? {
                    ctl = ControlFlow::Break(ControlFlow::Break(x));
                    break;
                }
            }
        }
    }
    match ctl {
        ControlFlow::Continue(()) => Ok(ControlFlow::Continue(match b.next()? {
            None => Ordering::Equal,
            Some(_) => Ordering::Less,
        })),
        ControlFlow::Break(x) => Ok(x),
    }
}

impl<'lend, L: FallibleLender> FallibleLending<'lend> for &mut L {
    type Lend = FallibleLend<'lend, L>;
}

impl<L: FallibleLender> FallibleLender for &mut L {
    type Error = L::Error;
    // SAFETY: the lend is that of L
    crate::unsafe_assume_covariance_fallible!();
    #[inline(always)]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        (**self).next()
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (**self).size_hint()
    }

    #[inline(always)]
    fn advance_by(&mut self, n: usize) -> Result<Result<(), NonZeroUsize>, Self::Error> {
        (**self).advance_by(n)
    }
}
