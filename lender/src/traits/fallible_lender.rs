use alloc::borrow::ToOwned;
use core::{cmp::Ordering, num::NonZeroUsize, ops::ControlFlow};

use stable_try_trait_v2::{internal::NeverShortCircuit, ChangeOutputType, FromResidual, Residual, Try};

use crate::{
    fallible_unzip,
    higher_order::{FnMutHKARes, FnMutHKAResOpt},
    non_fallible_adapter,
    traits::collect::IntoFallibleLender,
    Chain, Chunk, Cloned, Copied, Cycle, DoubleEndedFallibleLender, Enumerate, ExtendLender, FallibleFlatMap,
    FallibleFlatten, FallibleIntersperse, FallibleIntersperseWith, FalliblePeekable, FallibleTryShuntAdapter, Filter,
    FilterMap, FirstShunt, FromLender, Fuse, ImplBound, Inspect, Iter, Map, MapErr, MapIntoIter, MapWhile, Mutate,
    NonFallibleAdapter, Owned, ProductFallibleLender, Ref, Rev, Scan, SecondShunt, Skip, SkipWhile, StepBy,
    SumFallibleLender, Take, TakeWhile, TupleLend, Zip,
};

/// A trait for dealing with the 'items' of fallible lending iterators.
///
/// Must be defined for any type that implements [`FallibleLender`].
///
/// It implicitly restricts the lifetime `'lend` used in `FallibleLending<'lend>` to be `where Self: 'lend`.
pub trait FallibleLending<'lend, __ImplBound: ImplBound = Ref<'lend, Self>> {
    /// The type being lent.
    type Lend: 'lend;
}

/// A readable shorthand for the type of the items of a [`FallibleLender`] `L`.
pub type FallibleLend<'lend, L> = <L as FallibleLending<'lend>>::Lend;

/// A trait for dealing with lending iterators.
///
/// This is the main lender trait. For more about the concept of lenders
/// generally, please see the [crate documentation](crate).
///
/// For more about the concept of iterators
/// generally, please see [`core::iter`].
pub trait FallibleLender: for<'all /* where Self: 'all */> FallibleLending<'all> {
    /// The error type.
    type Error;

    /// Internal method for compile-time covariance checking.
    /// 
    /// This method should never be implemented directly. Instead, use the
    /// provided macros:
    ///
    /// - When implementing source lenders (lenders with concrete
    ///   [`Lend`](FallibleLending::Lend) types), users should invoke
    ///   [`check_covariance_fallible!`](crate::check_covariance_fallible) in
    ///   their [`FallibleLender`] impl. The macro implements the method as `{
    ///   lend }`, which only compiles if the [`Lend`](FallibleLending::Lend) type
    ///   is covariant in its lifetime.
    ///
    /// - When implementing adapters (lenders whose
    ///   [`Lend`](FallibleLending::Lend) type is derived from an underlying
    ///   fallible lender), users should invoke
    ///   [`inherit_covariance_fallible!`](crate::inherit_covariance_fallible).
    ///   The macro implements the method as `unsafe { core::mem::transmute(lend)
    ///   }`, which is safe because the underlying fallible lender's covariance
    ///   was already verified.
    ///
    /// # Safety
    ///
    /// Source lenders must implement this method as `{ lend }`. Adapters must
    /// implement this method as `unsafe { core::mem::transmute(lend) }`. In
    /// general, the implementation must guarantee that the [`Lend`](FallibleLending::Lend)
    /// type is covariant in its lifetime.
    unsafe fn _check_covariance<'long: 'short, 'short>(
        lend: <Self as FallibleLending<'long>>::Lend,
    ) -> <Self as FallibleLending<'short>>::Lend;

    /// Yield the next lend, if any, of the lender, or `Ok(None)` when iteration
    /// is finished.
    /// 
    /// The behavior of calling this method after a previous call has returned
    /// Ok(None) or Err is implementation defined.
    ///
    /// Every lend is only guaranteed to be valid one at a time for any kind of lender.
    /// 
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>([1, 2, 3u8].iter().into_fallible());
    /// assert_eq!(lender.next(), Ok(Some(&1)));
    /// assert_eq!(lender.next(), Ok(Some(&2)));
    /// assert_eq!(lender.next(), Ok(Some(&3)));
    /// assert_eq!(lender.next(), Ok(None));
    /// ```
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error>;

    /// Take the next `len` lends of the lender with temporary lender `Chunk`.
    /// This is equivalent to cloning the lender and calling `take(len)` on it.
    /// 
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>([1, 2, 3u8].iter().into_fallible());
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

    /// Get the estimated minimum and maximum length of the lender.
    /// Both bounds assume that all remaining calls to next succeed.
    /// That is, next could return an Err in fewer calls than specified by the lower bound.
    /// 
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>([1, 2, 3u8].iter().into_fallible());
    /// assert_eq!(lender.size_hint(), (3, Some(3)));
    /// ```
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) { (0, None) }

    /// Count the number of lends in the lender by consuming it until the
    /// lender yields `Ok(None)` or `Err(_)`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>([1, 2, 3u8].iter().into_fallible());
    /// assert_eq!(lender.count(), Ok(3));
    /// ```
    #[inline]
    fn count(self) -> Result<usize, Self::Error>
    where Self: Sized
    {
        self.fold(0, |n, _| Ok(n + 1))
    }

    /// Returns the last element of the iterator.
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

    /// Advance the lender by `n` lends. If the lender does not have enough lends, return the number of lends left.
    #[inline]
    fn advance_by(&mut self, n: usize) -> Result<Option<NonZeroUsize>, Self::Error> {
        for i in 0..n {
            if self.next()?.is_none() {
                // SAFETY: `i` is always less than `n`.
                return Ok(Some(unsafe { NonZeroUsize::new_unchecked(n - i) }));
            }
        }
        Ok(None)
    }

    /// Yield the nth lend of the lender, if any, by consuming it. If the lender does not have enough lends, returns `None`.
    ///
    /// n is zero-indexed.
    #[inline]
    fn nth(&mut self, n: usize) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        for _ in 0..n {
            if self.next()?.is_none() {
                return Ok(None)
            }
        }
        self.next()
    }

    /// Skip `step - 1` lends between each lend of the lender.
    ///
    /// # Panics
    ///
    /// Panics if `step` is zero.
    #[inline]
    fn step_by(self, step: usize) -> StepBy<Self>
    where
        Self: Sized,
    {
        StepBy::new(self, step)
    }

    /// Chain the lender with another lender of the same type.
    ///
    /// The resulting lender will first yield all lends from `self`, then all lends from `other`.
    fn chain<U>(self, other: U) -> Chain<Self, <U as IntoFallibleLender>::FallibleLender>
        where
            Self: Sized,
            U: IntoFallibleLender + for<'all> FallibleLending<'all, Lend = FallibleLend<'all, Self>>,
    {
        Chain::new(self, other.into_fallible_lender())
    }

    /// Zip the lender with another lender of the same or different type.
    #[inline]
    fn zip<U: IntoFallibleLender>(self, other: U) -> Zip<Self, <U as IntoFallibleLender>::FallibleLender>
    where
        Self: Sized,
    {
        Zip::new(self, other.into_fallible_lender())
    }

    /// Intersperse each lend of this lender with the given seperator.
    #[inline]
    fn intersperse<'call>(self, separator: FallibleLend<'call, Self>) -> FallibleIntersperse<'call, Self>
    where
        Self: Sized,
        for<'all> FallibleLend<'all, Self>: Clone,
    {
        FallibleIntersperse::new(self, separator)
    }

    /// Intersperse each lend of this lender with the seperator produced by the given function.
    #[inline]
    fn intersperse_with<'call, G>(self, separator: G) -> FallibleIntersperseWith<'call, Self, G>
    where
        Self: Sized,
        G: FnMut() -> Result<FallibleLend<'call, Self>, Self::Error>,
    {
        FallibleIntersperseWith::new(self, separator)
    }

    /// Map each lend of this lender using the given function.
    ///
    /// Please note that it is likely required that you use the
    /// [`hrc_mut!`](crate::hrc_mut!) macro to create
    /// the closure.
    #[inline]
    fn map<F>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
        F: for<'all> FnMutHKARes<'all, FallibleLend<'all, Self>, Self::Error>,
    {
        Map::new(self, f)
    }

    /// Map the error of this lender using the given function.
    #[inline]
    fn map_err<E, F>(self, f: F) -> MapErr<E, Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Error) -> E,
    {
        MapErr::new(self, f)
    }

    /// Map each lend of this lender into an owned value using the given function.
    ///
    /// This is a weaker version of [`FallibleLender::map`] that returns a
    /// [`FallibleIterator`](fallible_iterator::FallibleIterator) instead
    /// of a [`FallibleLender`]. However, this behavior is very common, and so
    /// this method is included for convenience. The main advantage is better
    /// type inference for the mapping function.
    #[inline]
    fn map_into_iter<O, F>(self, f: F) -> MapIntoIter<Self, O, F>
    where
        Self: Sized,
        F: FnMut(FallibleLend<'_, Self>) -> Result<O, Self::Error>
    {
        MapIntoIter::new(self, f)
    }

    /// Call the given function with each lend of this lender.
    #[inline]
    fn for_each<F>(mut self, mut f: F) -> Result<(), Self::Error>
    where
        Self: Sized,
        F: FnMut(FallibleLend<'_, Self>) -> Result<(), Self::Error>,
    {
        while let Some(a) = self.next()? {
            f(a)?;
        }
        Ok(())
    }

    /// Filter this lender using the given predicate.
    #[inline]
    fn filter<P>(self, predicate: P) -> Filter<Self, P>
    where
        Self: Sized,
        P: FnMut(&FallibleLend<'_, Self>) -> Result<bool, Self::Error>,
    {
        Filter::new(self, predicate)
    }

    /// Filter and map this lender using the given function.
    #[inline]
    fn filter_map<F>(self, f: F) -> FilterMap<Self, F>
    where
        Self: Sized,
        F: for<'all> FnMutHKAResOpt<'all, FallibleLend<'all, Self>, Self::Error>,
    {
        FilterMap::new(self, f)
    }

    /// Enumerate this lender. Each lend is paired with its zero-based index.
    #[inline]
    fn enumerate(self) -> Enumerate<Self>
    where
        Self: Sized,
    {
        Enumerate::new(self)
    }

    /// Make this lender peekable, so that it is possible to peek at the next lend without consuming it.
    #[inline]
    fn peekable<'call>(self) -> FalliblePeekable<'call, Self>
    where
        Self: Sized,
    {
        FalliblePeekable::new(self)
    }

    /// Skip the first contiguous sequence lends of this lender that satisfy the given predicate.
    #[inline]
    fn skip_while<P>(self, predicate: P) -> SkipWhile<Self, P>
    where
        Self: Sized,
        P: FnMut(&FallibleLend<'_, Self>) -> Result<bool, Self::Error>,
    {
        SkipWhile::new(self, predicate)
    }

    /// Take the first contiguous sequence lends of this lender that satisfy the given predicate.
    #[inline]
    fn take_while<P>(self, predicate: P) -> TakeWhile<Self, P>
    where
        Self: Sized,
        P: FnMut(&FallibleLend<'_, Self>) -> Result<bool, Self::Error>,
    {
        TakeWhile::new(self, predicate)
    }

    /// Map this lender using the given function while it returns `Some`.
    #[inline]
    fn map_while<P>(self, predicate: P) -> MapWhile<Self, P>
    where
        Self: Sized,
        P: for<'all> FnMutHKAResOpt<'all, FallibleLend<'all, Self>, Self::Error>,
    {
        MapWhile::new(self, predicate)
    }

    /// Skip the first `n` lends of this lender.
    #[inline]
    fn skip(self, n: usize) -> Skip<Self>
    where
        Self: Sized,
    {
        Skip::new(self, n)
    }

    /// Take the first `n` lends of this lender.
    #[inline]
    fn take(self, n: usize) -> Take<Self>
    where
        Self: Sized,
    {
        Take::new(self, n)
    }

    /// Documentation is incomplete. Refer to [`Iterator::scan`] for more information.
    #[inline]
    fn scan<St, F>(self, initial_state: St, f: F) -> Scan<Self, St, F>
    where
        Self: Sized,
        F: for<'all> FnMutHKAResOpt<'all, (&'all mut St, FallibleLend<'all, Self>), Self::Error>,
    {
        Scan::new(self, initial_state, f)
    }

    /// Documentation is incomplete. Refer to [`Iterator::flat_map`] for more information
    #[inline]
    fn flat_map<'call, F>(self, f: F) -> FallibleFlatMap<'call, Self, F>
    where
        Self: Sized,
        F: for<'all> FnMutHKARes<'all, FallibleLend<'all, Self>, Self::Error>,
        for<'all> <F as FnMutHKARes<'all, FallibleLend<'all, Self>, Self::Error>>::B: IntoFallibleLender<Error = Self::Error>,
    {
        FallibleFlatMap::new(self, f)
    }

    /// Documentation is incomplete. Refer to [`Iterator::flatten`] for more information
    #[inline]
    fn flatten<'call>(self) -> FallibleFlatten<'call, Self>
    where
        Self: Sized,
        for<'all> FallibleLend<'all, Self>: IntoFallibleLender<Error = Self::Error>,
    {
        FallibleFlatten::new(self)
    }

    /// Documentation is incomplete. Refer to [`Iterator::fuse`] for more information
    #[inline]
    fn fuse(self) -> Fuse<Self>
    where
        Self: Sized,
    {
        Fuse::new(self)
    }

    /// Documentation is incomplete. Refer to [`Iterator::inspect`] for more information
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>([1, 2, 3u8].iter().into_fallible());
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
    /// Mutate each lend with the given function.
    #[inline]
    fn mutate<F>(self, f: F) -> Mutate<Self, F>
    where
        Self: Sized,
        F: FnMut(&mut FallibleLend<'_, Self>) -> Result<(), Self::Error>,
    {
        Mutate::new(self, f)
    }

    /// Documentation is incomplete. Refer to [`Iterator::by_ref`] for more information
    fn by_ref(&mut self) -> &mut Self
    where
        Self: Sized,
    {
        self
    }

    /// Transforms the iterator into a collection.
    /// If any invocation of next returns Err, returns the collection built
    /// from values yielded successfully, together with the error.
    #[inline]
    fn collect<B>(self) -> Result<B, (B, Self::Error)>
    where
        Self: Sized,
        for<'all> B: FromLender<NonFallibleAdapter<'all, Self>>,
    {
        non_fallible_adapter::process(self, |lender| B::from_lender(lender))
    }

    /// Transforms the iterator into a collection.
    /// If any invocation of next returns Err, returns the collection built
    /// from values yielded successfully, together with the error.
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

    /// Transforms the iterator into a collection.
    /// If any invocation of next returns Err, returns the collection built
    /// from values yielded successfully, together with the error.
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

    /// Documentation is incomplete. Refer to [`Iterator::partition`] for more information
    fn partition<'this, A, E, F>(mut self, mut f: F) -> Result<(E, E), Self::Error>
    where
        Self: Sized + 'this,
        E: Default + ExtendLender<NonFallibleAdapter<'this, Self>>,
        F: FnMut(&FallibleLend<'_, Self>) -> Result<bool, Self::Error>
    {
        let mut left = E::default();
        let mut right = E::default();
        while let Some(x) = self.next()? {
            if f(&x)? {
                left.extend_lender_one(x);
            } else {
                right.extend_lender_one(x);
            }
        }
        Ok((left, right))
    }

    /// Documentation is incomplete. Refer to [`Iterator::is_partitioned`] for more information
    #[inline]
    #[allow(clippy::wrong_self_convention)]
    fn is_partitioned<P>(mut self, mut predicate: P) -> Result<bool, Self::Error>
    where
        Self: Sized,
        P: FnMut(FallibleLend<'_, Self>) -> Result<bool, Self::Error>,
    {
        Ok(self.all(&mut predicate)? || !self.any(predicate)?)
    }

    /// Documentation is incomplete. Refer to [`Iterator::try_fold`] for more information
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

    /// Documentation is incomplete. Refer to [`Iterator::try_for_each`] for more information
    #[inline]
    fn try_for_each<F, R>(&mut self, mut f: F) -> Result<R, Self::Error>
    where
        Self: Sized,
        F: FnMut(FallibleLend<'_, Self>) -> Result<R, Self::Error>,
        R: Try<Output = ()>,
    {
        while let Some(x) = self.next()? {
            if let ControlFlow::Break(x) = f(x)?.branch() {
                return Ok(R::from_residual(x));
            }
        }
        Ok(R::from_output(()))
    }

    /// Documentation is incomplete. Refer to [`Iterator::fold`] for more information
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>([1, 2, 3u8].iter().into_fallible());
    /// let sum = lender.fold(0u8, |acc, &x| Ok(acc + x));
    /// assert_eq!(sum, Ok(6));
    /// ```
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// // Example concatenating strings
    /// let data = vec!["hello", " ", "world"];
    /// let lender = lender::lend_fallible_iter::<fallible_lend!(&&'lend str), _>(data.iter().into_fallible());
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

    /// Documentation is incomplete. Refer to [`Iterator::reduce`] for more information
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

    /// Documentation is incomplete. Refer to [`Iterator::try_reduce`] for more information
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

    /// Documentation is incomplete. Refer to [`Iterator::all`] for more information
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>([1, 2, 3u8].iter().into_fallible());
    /// assert_eq!(lender.all(|&x| Ok(x > 0)), Ok(true));
    /// ```
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>([1, 2, 3u8].iter().into_fallible());
    /// assert_eq!(lender.all(|&x| Ok(x > 2)), Ok(false));
    /// ```
    #[inline]
    fn all<F>(&mut self, mut f: F) -> Result<bool, Self::Error>
    where
        Self: Sized,
        F: FnMut(FallibleLend<'_, Self>) -> Result<bool, Self::Error>,
    {
        while let Some(x) = self.next()? {
            if !f(x)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Documentation is incomplete. Refer to [`Iterator::any`] for more information
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>([1, 2, 3u8].iter().into_fallible());
    /// assert_eq!(lender.any(|&x| Ok(x == 2)), Ok(true));
    /// ```
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>([1, 2, 3u8].iter().into_fallible());
    /// assert_eq!(lender.any(|&x| Ok(x > 5)), Ok(false));
    /// ```
    #[inline]
    fn any<F>(&mut self, mut f: F) -> Result<bool, Self::Error>
    where
        Self: Sized,
        F: FnMut(FallibleLend<'_, Self>) -> Result<bool, Self::Error>,
    {
        while let Some(x) = self.next()? {
            if f(x)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Documentation is incomplete. Refer to [`Iterator::find`] for more information
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>([1, 2, 3u8].iter().into_fallible());
    /// assert_eq!(lender.find(|&&x| Ok(x == 2)), Ok(Some(&2)));
    /// ```
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>([1, 2, 3u8].iter().into_fallible());
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

    /// Documentation is incomplete. Refer to [`Iterator::find_map`] for more information
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

    /// Documentation is incomplete. Refer to [`Iterator::try_find`] for more information
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

    /// Documentation is incomplete. Refer to [`Iterator::position`] for more information
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>([1, 2, 3u8].iter().into_fallible());
    /// assert_eq!(lender.position(|&x| Ok(x == 2)), Ok(Some(1)));
    /// ```
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>([1, 2, 3u8].iter().into_fallible());
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

    /// Documentation is incomplete. Refer to [`Iterator::max`] for more information
    #[inline]
    fn max<T>(self) -> Result<Option<T>, Self::Error>
    where
        Self: Sized,
        T: for<'all> PartialOrd<FallibleLend<'all, Self>>,
        for<'all> FallibleLend<'all, Self>: ToOwned<Owned = T>,
    {
        self.max_by(|x, y| Ok(x.partial_cmp(y).unwrap_or(Ordering::Equal)))
    }

    /// Documentation is incomplete. Refer to [`Iterator::min`] for more information
    #[inline]
    fn min<T>(self) -> Result<Option<T>, Self::Error>
    where
        Self: Sized,
        T: for<'all> PartialOrd<FallibleLend<'all, Self>>,
        for<'all> FallibleLend<'all, Self>: ToOwned<Owned = T>,
    {
        self.min_by(|x, y| Ok(x.partial_cmp(y).unwrap_or(Ordering::Equal)))
    }

    /// Documentation is incomplete. Refer to [`Iterator::max_by_key`] for more information
    #[inline]
    fn max_by_key<B: Ord, T, F>(self, f: F) -> Result<Option<T>, Self::Error>
    where
        Self: Sized,
        for<'all> FallibleLend<'all, Self>: ToOwned<Owned = T>,
        F: FnMut(&T) -> Result<B, Self::Error>,
    {
        fallible_iterator::FallibleIterator::max_by_key(self.owned(), f)
    }

    /// Documentation is incomplete. Refer to [`Iterator::max_by`] for more information
    #[inline]
    fn max_by<T, F>(self, mut compare: F) -> Result<Option<T>, Self::Error>
    where
        Self: Sized,
        for<'all> FallibleLend<'all, Self>: ToOwned<Owned = T>,
        F: FnMut(&T, &FallibleLend<'_, Self>) -> Result<Ordering, Self::Error>
    {
        self.reduce(move |x, y| Ok(
            match compare(&x, &y)? {
                Ordering::Less => y.to_owned(),
                _ => x,
            }
        ))
    }

    /// Documentation is incomplete. Refer to [`Iterator::min_by_key`] for more information
    #[inline]
    fn min_by_key<B: Ord, T, F>(self, f: F) -> Result<Option<T>, Self::Error>
    where
        Self: Sized,
        for<'all> FallibleLend<'all, Self>: ToOwned<Owned = T>,
        F: FnMut(&T) -> Result<B, Self::Error>,
    {
        fallible_iterator::FallibleIterator::min_by_key(self.owned(), f)
    }

    /// Documentation is incomplete. Refer to [`Iterator::min_by`] for more information
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

    /// Documentation is incomplete. Refer to [`Iterator::rev`] for more information
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::IteratorExt as _;
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>([1, 2, 3u8].iter().into_fallible()).rev();
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

    /// Documentation is incomplete. Refer to [`Iterator::unzip`] for more information
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

    /// Documentation is incomplete. Refer to [`Iterator::copied`] for more information.
    ///
    /// Turns this FallibleLender into a `FallibleIterator`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fallible_iterator::{IteratorExt as _, FallibleIterator};
    /// # use lender::prelude::*;
    /// let lender = lender::lend_fallible_iter::<fallible_lend!(&'lend u8), _>([1, 2, 3u8].iter().into_fallible());
    /// let mut copied = lender.copied();
    /// assert_eq!(copied.next()?, Some(1));
    /// assert_eq!(copied.next()?, Some(2));
    /// assert_eq!(copied.next()?, Some(3));
    /// assert_eq!(copied.next()?, None);
    /// # Ok::<(), core::convert::Infallible>(())
    /// ```
    fn copied<T>(self) -> Copied<Self>
    where
        Self: Sized + for<'all> FallibleLending<'all, Lend = &'all T>,
        T: Copy,
    {
        Copied::new(self)
    }

    /// Documentation is incomplete. Refer to [`Iterator::cloned`] for more information.
    ///
    /// Turns this FallibleLender into a  FallibleIterator.
    fn cloned<T>(self) -> Cloned<Self>
    where
        Self: Sized + for<'all> FallibleLending<'all, Lend = &'all T>,
        T: Clone,
    {
        Cloned::new(self)
    }

    // not std::iter
    /// Turn this `FallibleLender` into a `FallibleIterator`.
    #[inline]
    fn owned(self) -> Owned<Self>
    where
        Self: Sized,
        for<'all> FallibleLend<'all, Self>: ToOwned
    {
        Owned::new(self)
    }

    /// Documentation is incomplete. Refer to [`Iterator::cycle`] for more information
    #[inline]
    fn cycle(self) -> Cycle<Self>
    where
        Self: Sized + Clone,
    {
        Cycle::new(self)
    }

    /// Documentation is incomplete. Refer to [`Iterator::sum`] for more information
    #[inline]
    fn sum<S>(self) -> Result<S, Self::Error>
    where
        Self: Sized,
        S: SumFallibleLender<Self>,
    {
        S::sum_lender(self)
    }

    /// Documentation is incomplete. Refer to [`Iterator::product`] for more information
    #[inline]
    fn product<P>(self) -> Result<P, Self::Error>
    where
        Self: Sized,
        P: ProductFallibleLender<Self>,
    {
        P::product_lender(self)
    }

    /// Documentation is incomplete. Refer to [`Iterator::cmp`] for more information
    fn cmp<L>(self, other: L) -> Result<Ordering, Self::Error>
    where
        L: IntoFallibleLender<Error = Self::Error>,
        L::FallibleLender: for<'all> FallibleLending<'all, Lend = FallibleLend<'all, Self>>,
        for <'all> FallibleLend<'all, Self>: Ord,
        Self: Sized,
    {
        self.cmp_by(other, |x, y| Ok(x.cmp(&y)))
    }

    /// Documentation is incomplete. Refer to [`Iterator::cmp_by`] for more information
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

    /// Documentation is incomplete. Refer to [`Iterator::partial_cmp`] for more information
    fn partial_cmp<L>(self, other: L) -> Result<Option<Ordering>, Self::Error>
    where
        L: IntoFallibleLender<Error = Self::Error>,
        for<'all> FallibleLend<'all, Self>: PartialOrd<FallibleLend<'all, L::FallibleLender>>,
        Self: Sized,
    {
        self.partial_cmp_by(other, |x, y| Ok(x.partial_cmp(&y)))
    }

    /// Documentation is incomplete. Refer to [`Iterator::partial_cmp_by`] for more information
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

    /// Documentation is incomplete. Refer to [`Iterator::eq`] for more information
    fn eq<L>(self, other: L) -> Result<bool, Self::Error>
    where
        L: IntoFallibleLender<Error = Self::Error>,
        for<'all> FallibleLend<'all, Self>: PartialEq<FallibleLend<'all, L::FallibleLender>>,
        Self: Sized,
    {
        self.eq_by(other, |x, y| Ok(x == y))
    }

    /// Documentation is incomplete. Refer to [`Iterator::eq_by`] for more information
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

    /// Documentation is incomplete. Refer to [`Iterator::ne`] for more information
    fn ne<L>(self, other: L) -> Result<bool, Self::Error>
    where
        L: IntoFallibleLender<Error = Self::Error>,
        for<'all> FallibleLend<'all, Self>: PartialEq<FallibleLend<'all, L::FallibleLender>>,
        Self: Sized,
    {
        self.eq(other).map(|eq| !eq)
    }

    /// Documentation is incomplete. Refer to [`Iterator::lt`] for more information
    fn lt<L>(self, other: L) -> Result<bool, Self::Error>
    where
        L: IntoFallibleLender<Error = Self::Error>,
        for<'all> FallibleLend<'all, Self>: PartialOrd<FallibleLend<'all, L::FallibleLender>>,
        Self: Sized,
    {
        Ok(self.partial_cmp(other)? == Some(Ordering::Less))
    }

    /// Documentation is incomplete. Refer to [`Iterator::le`] for more information
    fn le<L>(self, other: L) -> Result<bool, Self::Error>
    where
        L: IntoFallibleLender<Error = Self::Error>,
        for<'all> FallibleLend<'all, Self>: PartialOrd<FallibleLend<'all, L::FallibleLender>>,
        Self: Sized,
    {
        Ok(matches!(self.partial_cmp(other)?, Some(Ordering::Less | Ordering::Equal)))
    }

    /// Documentation is incomplete. Refer to [`Iterator::gt`] for more information
    fn gt<L>(self, other: L) -> Result<bool, Self::Error>
    where
        L: IntoFallibleLender<Error = Self::Error>,
        for<'all> FallibleLend<'all, Self>: PartialOrd<FallibleLend<'all, L::FallibleLender>>,
        Self: Sized,
    {
        Ok(self.partial_cmp(other)? == Some(Ordering::Greater))
    }

    /// Documentation is incomplete. Refer to [`Iterator::ge`] for more information
    fn ge<L>(self, other: L) -> Result<bool, Self::Error>
    where
        L: IntoFallibleLender<Error = Self::Error>,
        for<'all> FallibleLend<'all, Self>: PartialOrd<FallibleLend<'all, L::FallibleLender>>,
        Self: Sized,
    {
        Ok(matches!(self.partial_cmp(other)?, Some(Ordering::Greater | Ordering::Equal)))
    }

    /// Documentation is incomplete. Refer to [`Iterator::is_sorted`] for more information
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

    /// Documentation is incomplete. Refer to [`Iterator::is_sorted_by`] for more information
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

    /// Documentation is incomplete. Refer to [`Iterator::is_sorted_by_key`] for more information
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

    /// Turn this lender into a `FallibleIterator` where it has already fulfilled the requirements of the `FallibleIterator` trait.
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
pub(crate) fn lender_compare<A, B, F, T>(mut a: A, mut b: B, mut f: F) -> Result<ControlFlow<T, Ordering>, A::Error>
where
    A: FallibleLender,
    B: FallibleLender<Error = A::Error>,
    for<'all> F: FnMut(FallibleLend<'all, A>, FallibleLend<'all, B>) -> Result<ControlFlow<T>, A::Error>,
{
    let mut ctl = ControlFlow::Continue(());
    while let Some(x) = a.next()? {
        match b.next()? {
            None => {
                ctl = ControlFlow::Break(ControlFlow::Continue(Ordering::Greater));
                break;
            }
            Some(y) => {
                let this = f(x, y)?;
                let f = ControlFlow::Break;
                if let ControlFlow::Break(x) = this {
                    ctl = ControlFlow::Break(f(x));
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
    crate::inherit_covariance_fallible!();
    #[inline]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        (**self).next()
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (**self).size_hint()
    }
    #[inline]
    fn advance_by(&mut self, n: usize) -> Result<Option<NonZeroUsize>, Self::Error> {
        (**self).advance_by(n)
    }
}
