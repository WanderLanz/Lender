use alloc::borrow::ToOwned;
use core::{cmp::Ordering, num::NonZeroUsize, ops::ControlFlow};

use crate::{
    higher_order::{FnMutHKA, FnMutHKAOpt},
    try_trait_v2::{ChangeOutputType, FromResidual, Residual, Try},
    *,
};

/// A trait for dealing with the 'items' of lending iterators.
///
/// Must be defined for any type that implements [`Lender`].
///
/// It implicitly restricts the lifetime `'lend` used in [`Lending<'lend>`](Lending) to be
/// `where Self: 'lend`.
///
/// This is a result of Higher-Rank Trait Bounds (HRTBs) not having a way to express qualifiers
/// (`for<'any where Self: 'any> Self: Trait`) and effectively making HRTBs only useful when you
/// want to express a trait constraint on ALL lifetimes, including 'static
/// (`for<'all> Self: trait`).
///
/// Although the common example of implementing your own lending iterator uses a
/// (`type Item<'a> where Self: 'a;`) GAT, that generally only works within a small subset of the
/// features that a lending iterator needs to provide to be comparable to [`Iterator`].
///
/// Please see [Sabrina Jewson's Blog][1] for more information, and how a trait like this can be
/// used to solve it by implicitly restricting HRTBs.
///
/// [1]: https://sabrinajewson.org/blog/the-better-alternative-to-lifetime-gats
pub trait Lending<'lend, __ImplBound: ImplBound = Ref<'lend, Self>> {
    /// The type being lent.
    type Lend: 'lend;
}

/// A readable shorthand for the type of the items of a [`Lender`] `L`.
pub type Lend<'lend, L> = <L as Lending<'lend>>::Lend;

/// A trait for dealing with lending iterators.
///
/// This is the main lender trait. For more about the concept of lenders
/// generally, please see the [crate documentation](crate).
///
/// For more about the concept of iterators
/// generally, please see [`core::iter`].
pub trait Lender: for<'all /* where Self: 'all */> Lending<'all> {
    /// Internal method for compile-time covariance checking.
    /// 
    /// This method should rarely be implemented directly. Instead, use the
    /// provided macros:
    ///
    /// - When implementing source lenders (lenders with concrete
    ///   [`Lend`](Lending::Lend) types), use
    ///   [`check_covariance!`](crate::check_covariance) in the [`Lender`] impl.
    ///   The macro implements the method as `{ lend }`, which only compiles if
    ///   the [`Lend`](Lending::Lend) type is covariant in its lifetime.
    ///
    /// - In all other cases (e.g., when implementing adapters), use
    ///   [`unsafe_assume_covariance!`](crate::unsafe_assume_covariance) in the
    ///   [`Lender`] impl. The macro implements the method as `unsafe {
    ///   core::mem::transmute(lend) }`, which is a no-op. This is unsafe because
    ///   it is up to the implementor to guarantee that the [`Lend`](Lending::Lend)
    ///   type is covariant in its lifetime.
    fn _check_covariance<'long: 'short, 'short>(
        lend: *const &'short <Self as Lending<'long>>::Lend, _: crate::Uncallable,
    ) -> *const &'short <Self as Lending<'short>>::Lend;

    /// Yields the next lend, if any, of the lender.
    ///
    /// Every lend is only guaranteed to be valid one at a time for any kind of lender.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_iter::<lend!(&'lend u8), _>([1, 2, 3u8].iter());
    /// assert_eq!(lender.next(), Some(&1));
    /// assert_eq!(lender.next(), Some(&2));
    /// assert_eq!(lender.next(), Some(&3));
    /// assert_eq!(lender.next(), None);
    /// ```
    fn next(&mut self) -> Option<Lend<'_, Self>>;
    /// Takes the next `chunk_size` lends of the lender with temporary lender [`Chunk`]. This is equivalent to cloning the lender and calling [`take(chunk_size)`](Lender::take) on it.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_iter::<lend!(&'lend u8), _>([1, 2, 3u8].iter());
    /// let mut chunk_lender = lender.next_chunk(2);
    /// assert_eq!(chunk_lender.next(), Some(&1));
    /// assert_eq!(chunk_lender.next(), Some(&2));
    /// assert_eq!(chunk_lender.next(), None);
    /// ```
    #[inline]
    fn next_chunk(&mut self, chunk_size: usize) -> Chunk<'_, Self>
    where
        Self: Sized,
    {
        Chunk::new(self, chunk_size)
    }
    /// Gets the estimated minimum and maximum length of the lender. Use [`.len()`](ExactSizeLender::len) for the exact length if the lender implements [`ExactSizeLender`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_iter::<lend!(&'lend u8), _>([1, 2, 3u8].iter());
    /// assert_eq!(lender.size_hint(), (3, Some(3)));
    /// ```
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) { (0, None) }
    /// Counts the number of lends in the lender by consuming it.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_iter::<lend!(&'lend u8), _>([1, 2, 3u8].iter());
    /// assert_eq!(lender.count(), 3);
    /// ```
    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.fold(0, |count, _| count + 1)
    }
    /// Gets the last lend of the lender, if any, by consuming it.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_iter::<lend!(&'lend u8), _>([1, 2, 3u8].iter());
    /// assert_eq!(lender.last(), Some(&3));
    /// ```
    #[inline]
    fn last<'call>(&'call mut self) -> Option<Lend<'call, Self>>
    where
        Self: Sized,
    {
        let mut last = None;
        while let Some(x) = self.next() {
            // SAFETY: polonius return
            last = Some(unsafe {
                core::mem::transmute::<
                    Lend<'_, Self>,
                    Lend<'call, Self>
                >(x)
            });
        }
        last
    }
    /// Advances the lender by `n` lends. If the lender does not have enough lends, returns the number of lends left.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_iter::<lend!(&'lend u8), _>([1, 2, 3u8].iter());
    /// assert_eq!(lender.advance_by(2), Ok(()));
    /// assert_eq!(lender.next(), Some(&3));
    /// ```
    #[inline]
    fn advance_by(&mut self, n: usize) -> Result<(), NonZeroUsize> {
        for i in 0..n {
            if self.next().is_none() {
                // SAFETY: `i` is always less than `n`.
                return Err(unsafe { NonZeroUsize::new_unchecked(n - i) });
            }
        }
        Ok(())
    }
    /// Yields the nth lend of the lender, if any, by consuming it. If the lender does not have enough lends, returns [`None`].
    ///
    /// n is zero-indexed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_iter::<lend!(&'lend u8), _>([1, 2, 3u8].iter());
    /// assert_eq!(lender.nth(2), Some(&3));
    /// ```
    #[inline]
    fn nth(&mut self, n: usize) -> Option<Lend<'_, Self>> {
        self.advance_by(n).ok()?;
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
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_iter::<lend!(&'lend u8), _>([1, 2, 3, 4, 5, 6, 7, 8, 9, 10].iter());
    /// let mut step_lender = lender.step_by(2);
    /// assert_eq!(step_lender.next(), Some(&1));
    /// assert_eq!(step_lender.next(), Some(&3));
    /// assert_eq!(step_lender.next(), Some(&5));
    /// assert_eq!(step_lender.next(), Some(&7));
    /// assert_eq!(step_lender.next(), Some(&9));
    /// assert_eq!(step_lender.next(), None);
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
    /// The resulting lender will first yield all lends from `self`, then all lends from `other`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_iter::<lend!(&'lend u8), _>([1, 2].iter());
    /// let mut other = lender::lend_iter::<lend!(&'lend u8), _>([3, 4].iter());
    /// let mut chained = lender.chain(other);
    /// assert_eq!(chained.next(), Some(&1));
    /// assert_eq!(chained.next(), Some(&2));
    /// assert_eq!(chained.next(), Some(&3));
    /// assert_eq!(chained.next(), Some(&4));
    /// assert_eq!(chained.next(), None);
    /// ```
    #[inline]
    fn chain<U>(self, other: U) -> Chain<Self, <U as IntoLender>::Lender>
    where
        Self: Sized,
        for<'all> U: IntoLender + Lending<'all, Lend = Lend<'all, Self>>,
    {
        Chain::new(self, other.into_lender())
    }
    /// Zips the lender with another lender of the same or different type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_iter::<lend!(&'lend u8), _>([1, 2].iter());
    /// let mut other = lender::lend_iter::<lend!(&'lend u8), _>([3, 4].iter());
    /// let mut zipped = lender.zip(other);
    /// assert_eq!(zipped.next(), Some((&1, &3)));
    /// assert_eq!(zipped.next(), Some((&2, &4)));
    /// assert_eq!(zipped.next(), None);
    /// ```
    #[inline]
    fn zip<U: IntoLender>(self, other: U) -> Zip<Self, <U as IntoLender>::Lender>
    where
        Self: Sized,
    {
        Zip::new(self, other.into_lender())
    }
    /// Intersperses each lend of this lender with the given separator.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_iter::<lend!(&'lend u8), _>([1, 2].iter());
    /// let mut interspersed = lender.intersperse(&0);
    /// assert_eq!(interspersed.next(), Some(&1));
    /// assert_eq!(interspersed.next(), Some(&0));
    /// assert_eq!(interspersed.next(), Some(&2));
    /// assert_eq!(interspersed.next(), None);
    /// ```
    #[inline]
    fn intersperse<'call>(self, separator: Lend<'call, Self>) -> Intersperse<'call, Self>
    where
        Self: Sized,
        for<'all> Lend<'all, Self>: Clone,
    {
        Intersperse::new(self, separator)
    }
    /// Intersperses each lend of this lender with the separator produced by the given function.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_iter::<lend!(&'lend u8), _>([1, 2].iter());
    /// let mut interspersed = lender.intersperse_with(|| &0);
    /// assert_eq!(interspersed.next(), Some(&1));
    /// assert_eq!(interspersed.next(), Some(&0));
    /// assert_eq!(interspersed.next(), Some(&2));
    /// assert_eq!(interspersed.next(), None);
    /// ```
    #[inline]
    fn intersperse_with<'call, G>(self, separator: G) -> IntersperseWith<'call, Self, G>
    where
        Self: Sized,
        G: FnMut() -> Lend<'call, Self>,
    {
        IntersperseWith::new(self, separator)
    }
    /// Maps each lend of this lender using the given function.
    ///
    /// Note that functions passed to this method must be built using the
    /// [`hrc!`](crate::hrc) or [`hrc_mut!`](crate::hrc_mut) macro, which also
    /// checks for covariance of the returned type. Circumventing the macro may
    /// result in undefined behavior if the return type is not covariant.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lender::prelude::*;
    /// let mut data = [1, 2u8];
    /// let mut lender = lender::lend_iter::<lend!(&'lend mut u8), _>(data.iter_mut());
    /// let mut mapped = lender.map(hrc_mut!(for<'all> |a: &'all mut u8| -> &'all u8 {
    ///     *a += 1;
    ///     &*a
    /// }));
    /// assert_eq!(mapped.next(), Some(&2));
    /// assert_eq!(mapped.next(), Some(&3));
    /// assert_eq!(mapped.next(), None);
    /// ```
    #[inline]
    fn map<F>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
        F: for<'all> FnMutHKA<'all, Lend<'all, Self>>,
    {
        Map::new(self, f)
    }
    /// Maps each lend of this lender into an owned value using the given function.
    ///
    /// This is a weaker version of [`Lender::map`] that returns an [`Iterator`] instead of a [`Lender`].
    /// However, this behavior is very common, and so this method is included for convenience.
    /// The main advantage is better type inference for the mapping function.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lender::prelude::*;
    /// let mut data = [1, 2u8];
    /// let mut lender = lender::lend_iter::<lend!(&'lend mut u8), _>(data.iter_mut());
    /// let mut mapped_into_iter = lender.map_into_iter(|a| {
    ///     *a += 1;
    ///     *a
    /// });
    /// assert_eq!(mapped_into_iter.next(), Some(2));
    /// assert_eq!(mapped_into_iter.next(), Some(3));
    /// assert_eq!(mapped_into_iter.next(), None);
    /// ```
    #[inline]
    fn map_into_iter<O, F: FnMut(Lend<'_, Self>) -> O>(self, f: F) -> MapIntoIter<Self, O, F>
    where Self: Sized
    {
        MapIntoIter::new(self, f)
    }
    /// Calls the given function with each lend of this lender.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_iter::<lend!(&'lend u8), _>([1, 2u8].iter());
    /// lender.for_each(|a| {
    ///     let _ = *a + 1;
    /// });
    /// ```
    #[inline]
    fn for_each<F>(self, mut f: F)
    where
        Self: Sized,
        F: FnMut(Lend<'_, Self>),
    {
	self.fold((), |_, t| f(t))
    }
    /// Filters this lender using the given predicate.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_iter::<lend!(&'lend u8), _>([1, 2u8].iter());
    /// let mut filtered = lender.filter(|&a| *a > 1);
    /// assert_eq!(filtered.next(), Some(&2));
    /// assert_eq!(filtered.next(), None);
    /// ```
    #[inline]
    fn filter<P>(self, predicate: P) -> Filter<Self, P>
    where
        Self: Sized,
        P: FnMut(&Lend<'_, Self>) -> bool,
    {
        Filter::new(self, predicate)
    }
    /// Filters and maps this lender using the given function.
    ///
    /// Note that functions passed to this method must be built using the
    /// [`hrc!`](crate::hrc) or [`hrc_mut!`](crate::hrc_mut) macro, which also
    /// checks for covariance of the returned type. Circumventing the macro may
    /// result in undefined behavior if the return type is not covariant.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lender::prelude::*;
    /// let mut data = [1, 2u8];
    /// let mut lender = lender::lend_iter::<lend!(&'lend mut u8), _>(data.iter_mut());
    /// let mut filtered = lender.filter_map(hrc_mut!(for<'all> |a: &'all mut u8| -> Option<&'all u8> {
    ///     if *a > 1 {
    ///         Some(&*a)
    ///     } else {
    ///         None
    ///     }
    /// }));
    /// assert_eq!(filtered.next(), Some(&2));
    /// assert_eq!(filtered.next(), None);
    /// ```
    #[inline]
    fn filter_map<F>(self, f: F) -> FilterMap<Self, F>
    where
        Self: Sized,
        F: for<'all> FnMutHKAOpt<'all, Lend<'all, Self>>,
    {
        FilterMap::new(self, f)
    }
    /// Enumerates this lender. Each lend is paired with its zero-based index.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_iter::<lend!(&'lend u8), _>([1, 2u8].iter());
    /// let mut enumerated = lender.enumerate();
    /// assert_eq!(enumerated.next(), Some((0, &1)));
    /// assert_eq!(enumerated.next(), Some((1, &2)));
    /// assert_eq!(enumerated.next(), None);
    /// ```
    #[inline]
    fn enumerate(self) -> Enumerate<Self>
    where
        Self: Sized,
    {
        Enumerate::new(self)
    }
    /// Makes this lender peekable, so that it is possible to peek at the next lend without consuming it.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_iter::<lend!(&'lend u8), _>([1, 2u8].iter());
    /// let mut peekable = lender.peekable();
    /// assert_eq!(peekable.peek(), Some(&&1));
    /// assert_eq!(peekable.next(), Some(&1));
    /// assert_eq!(peekable.peek(), Some(&&2));
    /// assert_eq!(peekable.next(), Some(&2));
    /// assert_eq!(peekable.peek(), None);
    /// ```
    #[inline]
    fn peekable<'call>(self) -> Peekable<'call, Self>
    where
        Self: Sized,
    {
        Peekable::new(self)
    }
    /// Skips the first contiguous sequence of lends of this lender that satisfy the given predicate.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_iter::<lend!(&'lend u8), _>([1, 2, 3, 4, 5].iter());
    /// let mut skipped = lender.skip_while(|&a| *a < 3);
    /// assert_eq!(skipped.next(), Some(&3));
    /// assert_eq!(skipped.next(), Some(&4));
    /// assert_eq!(skipped.next(), Some(&5));
    /// assert_eq!(skipped.next(), None);
    /// ```
    #[inline]
    fn skip_while<P>(self, predicate: P) -> SkipWhile<Self, P>
    where
        Self: Sized,
        P: FnMut(&Lend<'_, Self>) -> bool,
    {
        SkipWhile::new(self, predicate)
    }
    /// Takes the first contiguous sequence of lends of this lender that satisfy the given predicate.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_iter::<lend!(&'lend u8), _>([1, 2, 3, 4, 5].iter());
    /// let mut taken = lender.take_while(|&a| *a < 3);
    /// assert_eq!(taken.next(), Some(&1));
    /// assert_eq!(taken.next(), Some(&2));
    /// assert_eq!(taken.next(), None);
    /// ```
    #[inline]
    fn take_while<P>(self, predicate: P) -> TakeWhile<Self, P>
    where
        Self: Sized,
        P: FnMut(&Lend<'_, Self>) -> bool,
    {
        TakeWhile::new(self, predicate)
    }
    /// Maps this lender using the given function while it returns [`Some`].
    ///
    /// Note that functions passed to this method must be built using the
    /// [`hrc!`](crate::hrc) or [`hrc_mut!`](crate::hrc_mut) macro, which also
    /// checks for covariance of the returned type. Circumventing the macro may
    /// result in undefined behavior if the return type is not covariant.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lender::prelude::*;
    /// let mut data = [1, 2u8];
    /// let mut lender = lender::lend_iter::<lend!(&'lend mut u8), _>(data.iter_mut());
    /// let mut mapped = lender.map_while(hrc_mut!(for<'all> |a: &'all mut u8| -> Option<&'all u8> {
    ///     if *a < 2 {
    ///         Some(&*a)
    ///     } else {
    ///         None
    ///     }
    /// }));
    /// assert_eq!(mapped.next(), Some(&1));
    /// assert_eq!(mapped.next(), None);
    /// ```
    #[inline]
    fn map_while<P>(self, predicate: P) -> MapWhile<Self, P>
    where
        Self: Sized,
        P: for<'all> FnMutHKAOpt<'all, Lend<'all, Self>>,
    {
        MapWhile::new(self, predicate)
    }
    /// Skips the first `n` lends of this lender.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_iter::<lend!(&'lend u8), _>([1, 2, 3, 4, 5].iter());
    /// let mut skipped = lender.skip(3);
    /// assert_eq!(skipped.next(), Some(&4));
    /// assert_eq!(skipped.next(), Some(&5));
    /// assert_eq!(skipped.next(), None);
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
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_iter::<lend!(&'lend u8), _>([1, 2, 3, 4, 5].iter());
    /// let mut taken = lender.take(2);
    /// assert_eq!(taken.next(), Some(&1));
    /// assert_eq!(taken.next(), Some(&2));
    /// assert_eq!(taken.next(), None);
    /// ```
    #[inline]
    fn take(self, n: usize) -> Take<Self>
    where
        Self: Sized,
    {
        Take::new(self, n)
    }
    /// The [`Lender`] version of [`Iterator::scan`].
    ///
    /// Note that functions passed to this method must be built using the
    /// [`hrc!`](crate::hrc) or [`hrc_mut!`](crate::hrc_mut) macro, which also
    /// checks for covariance of the returned type. Circumventing the macro may
    /// result in undefined behavior if the return type is not covariant.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_iter::<lend!(&'lend u8), _>([1u8, 2, 3].iter());
    /// let mut scanned = lender.scan(0u8, hrc_mut!(for<'all> |args: (&'all mut u8, &'all u8)| -> Option<&'all u8> {
    ///     *args.0 += *args.1;
    ///     Some(args.1)
    /// }));
    /// assert_eq!(scanned.next(), Some(&1));
    /// assert_eq!(scanned.next(), Some(&2));
    /// ```
    #[inline]
    fn scan<St, F>(self, initial_state: St, f: F) -> Scan<Self, St, F>
    where
        Self: Sized,
        F: for<'all> FnMutHKAOpt<'all, (&'all mut St, Lend<'all, Self>)>,
    {
        Scan::new(self, initial_state, f)
    }
    /// The [`Lender`] version of [`Iterator::flat_map`].
    ///
    /// Note that functions passed to this method must be built using the
    /// [`hrc!`](crate::hrc) or [`hrc_mut!`](crate::hrc_mut) macro, which also
    /// checks for covariance of the returned type. Circumventing the macro may
    /// result in undefined behavior if the return type is not covariant.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lender::prelude::*;
    /// // Define a wrapper that implements Lender
    /// struct VecLender(Vec<i32>);
    ///
    /// impl<'lend> Lending<'lend> for VecLender {
    ///     type Lend = i32;
    /// }
    ///
    /// impl Lender for VecLender {
    ///     check_covariance!();
    ///     fn next(&mut self) -> Option<Lend<'_, Self>> {
    ///         if self.0.is_empty() { None } else { Some(self.0.remove(0)) }
    ///     }
    /// }
    ///
    /// let data = vec![1i32, 2, 3];
    /// let mut flat = data.into_iter().into_lender().flat_map(
    ///     hrc_mut!(for<'lend> |x: i32| -> VecLender { VecLender(vec![x, x * 10]) })
    /// );
    /// assert_eq!(flat.next(), Some(1));
    /// assert_eq!(flat.next(), Some(10));
    /// assert_eq!(flat.next(), Some(2));
    /// ```
    #[inline]
    fn flat_map<'call, F>(self, f: F) -> FlatMap<'call, Self, F>
    where
        Self: Sized,
        F: for<'all> FnMutHKA<'all, Lend<'all, Self>>,
        for<'all> <F as FnMutHKA<'all, Lend<'all, Self>>>::B: IntoLender,
    {
        FlatMap::new(self, f)
    }
    /// The [`Lender`] version of [`Iterator::flatten`].
    #[inline]
    fn flatten<'call>(self) -> Flatten<'call, Self>
    where
        Self: Sized,
        for<'all> Lend<'all, Self>: IntoLender,
    {
        Flatten::new(self)
    }
    /// The [`Lender`] version of [`Iterator::fuse`].
    #[inline]
    fn fuse(self) -> Fuse<Self>
    where
        Self: Sized,
    {
        Fuse::new(self)
    }
    /// The [`Lender`] version of [`Iterator::inspect`].
    #[inline]
    fn inspect<F>(self, f: F) -> Inspect<Self, F>
    where
        Self: Sized,
        F: FnMut(&Lend<'_, Self>),
    {
        Inspect::new(self, f)
    }
    // not std::iter
    /// Mutates each lend with the given function.
    #[inline]
    fn mutate<F>(self, f: F) -> Mutate<Self, F>
    where
        Self: Sized,
        F: FnMut(&mut Lend<'_, Self>),
    {
        Mutate::new(self, f)
    }
    /// The [`Lender`] version of [`Iterator::by_ref`].
    fn by_ref(&mut self) -> &mut Self
    where
        Self: Sized,
    {
        self
    }
    /// The [`Lender`] version of [`Iterator::collect`].
    #[inline]
    fn collect<B>(self) -> B
    where
        Self: Sized,
        B: FromLender<Self>,
    {
        B::from_lender(self)
    }
    /// The [`Lender`] version of [`Iterator::try_collect`].
    #[inline]
    fn try_collect<'a, B>(&'a mut self) -> ChangeOutputType<Lend<'a, Self>, B>
    where
        Self: Sized,
        for<'all> Lend<'all, Self>: Try,
        for<'all> <Lend<'all, Self> as Try>::Residual: Residual<B>,
        for<'all> B: FromLender<TryShunt<'all, &'a mut Self>>,
    {
        try_process::<&'a mut Self, _, B>(self.by_ref(),|shunt: TryShunt<'_, &'a mut Self>| B::from_lender(shunt))
    }
    /// The [`Lender`] version of [`Iterator::collect_into`].
    #[inline]
    fn collect_into<E>(self, collection: &mut E) -> &mut E
    where
        Self: Sized,
        E: ExtendLender<Self>,
    {
        collection.extend_lender(self);
        collection
    }
    /// The [`Lender`] version of [`Iterator::partition`].
    fn partition<A, E, F>(mut self, mut f: F) -> (E, E)
    where
        Self: Sized,
        E: Default + ExtendLender<Self>,
        F: FnMut(&Lend<'_, Self>) -> bool,
    {
        let mut left = E::default();
        let mut right = E::default();
        while let Some(x) = self.next() {
            if f(&x) {
                left.extend_lender_one(x);
            } else {
                right.extend_lender_one(x);
            }
        }
        (left, right)
    }
    /// The [`Lender`] version of [`Iterator::is_partitioned`].
    #[inline]
    #[allow(clippy::wrong_self_convention)]
    fn is_partitioned<P>(mut self, mut predicate: P) -> bool
    where
        Self: Sized,
        P: FnMut(Lend<'_, Self>) -> bool,
    {
        self.all(&mut predicate) || !self.any(predicate)
    }
    /// The [`Lender`] version of [`Iterator::try_fold`].
    #[inline]
    fn try_fold<B, F, R>(&mut self, init: B, mut f: F) -> R
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> R,
        R: Try<Output = B>,
    {
        let mut acc = init;
        while let Some(x) = self.next() {
            acc = match f(acc, x).branch() {
                ControlFlow::Break(x) => return R::from_residual(x),
                ControlFlow::Continue(x) => x,
            };
        }
        R::from_output(acc)
    }
    /// The [`Lender`] version of [`Iterator::try_for_each`].
    #[inline]
    fn try_for_each<F, R>(&mut self, mut f: F) -> R
    where
        Self: Sized,
        F: FnMut(Lend<'_, Self>) -> R,
        R: Try<Output = ()>,
    {
        while let Some(x) = self.next() {
            if let ControlFlow::Break(x) = f(x).branch() {
                return R::from_residual(x);
            }
        }
        R::from_output(())
    }
    /// The [`Lender`] version of [`Iterator::fold`].
    #[inline]
    fn fold<B, F>(mut self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Lend<'_, Self>) -> B,
    {
        let mut accum = init;
        while let Some(x) = self.next() {
            accum = f(accum, x);
        }
        accum
    }
    /// The [`Lender`] version of [`Iterator::reduce`].
    #[inline]
    fn reduce<T, F>(mut self, f: F) -> Option<T>
    where
        Self: Sized,
        for<'all> Lend<'all, Self>: ToOwned<Owned = T>,
        F: FnMut(T, Lend<'_, Self>) -> T,
    {
        let first = self.next()?.to_owned();
        Some(self.fold(first, f))
    }
    /// The [`Lender`] version of [`Iterator::try_reduce`].
    #[inline]
    fn try_reduce<T, F, R>(mut self, f: F) -> ChangeOutputType<R, Option<T>>
    where
        Self: Sized,
        for<'all> Lend<'all, Self>: ToOwned<Owned = T>,
        F: FnMut(T, Lend<'_, Self>) -> R,
        R: Try<Output = T>,
        R::Residual: Residual<Option<T>>,
    {
        let first = match self.next() {
            Some(ref x) => x.to_owned(),
            None => return Try::from_output(None),
        };
        match self.try_fold(first, f).branch() {
            ControlFlow::Break(x) => FromResidual::from_residual(x),
            ControlFlow::Continue(x) => Try::from_output(Some(x)),
        }
    }
    /// The [`Lender`] version of [`Iterator::all`].
    #[inline]
    fn all<F>(&mut self, mut f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Lend<'_, Self>) -> bool,
    {
        while let Some(x) = self.next() {
            if !f(x) {
                return false;
            }
        }
        true
    }
    /// The [`Lender`] version of [`Iterator::any`].
    #[inline]
    fn any<F>(&mut self, mut f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Lend<'_, Self>) -> bool,
    {
        while let Some(x) = self.next() {
            if f(x) {
                return true;
            }
        }
        false
    }
    /// The [`Lender`] version of [`Iterator::find`].
    #[inline]
    fn find<P>(&mut self, mut predicate: P) -> Option<Lend<'_, Self>>
    where
        Self: Sized,
        P: FnMut(&Lend<'_, Self>) -> bool,
    {
        while let Some(x) = self.next() {
            if predicate(&x) {
                // SAFETY: polonius return
                return Some(unsafe {
                    core::mem::transmute::<
                        Lend<'_, Self>,
                        Lend<'_, Self>
                    >(x)
                });
            }
        }
        None
    }
    /// The [`Lender`] version of [`Iterator::find_map`].
    #[inline]
    fn find_map<'a, F>(&'a mut self, mut f: F) -> Option<<F as FnMutHKAOpt<'a, Lend<'a, Self>>>::B>
    where
        Self: Sized,
        F: for<'all> FnMutHKAOpt<'all, Lend<'all, Self>>,
    {
        while let Some(x) = self.next() {
            if let Some(y) = f(x) {
                // SAFETY: polonius return
                return Some(unsafe {
                    core::mem::transmute::<
                        <F as FnMutHKAOpt<'_, Lend<'_, Self>>>::B,
                        <F as FnMutHKAOpt<'a, Lend<'a, Self>>>::B
                    >(y)
                });
            }
        }
        None
    }
    /// The [`Lender`] version of [`Iterator::try_find`].
    #[inline]
    fn try_find<F, R>(&mut self, mut f: F) -> ChangeOutputType<R, Option<Lend<'_, Self>>>
    where
        Self: Sized,
        F: FnMut(&Lend<'_, Self>) -> R,
        R: Try<Output = bool>,
        for<'all> R::Residual: Residual<Option<Lend<'all, Self>>>,
    {
        while let Some(x) = self.next() {
            match f(&x).branch() {
                ControlFlow::Break(x) => return <ChangeOutputType<R, Option<Lend<'_, Self>>>>::from_residual(x),
                ControlFlow::Continue(cond) => {
                    if cond {
                        // SAFETY: polonius return
                        return <ChangeOutputType<R, Option<Lend<'_, Self>>>>::from_output(
                            Some(unsafe {
                                core::mem::transmute::<
                                    Lend<'_, Self>,
                                    Lend<'_, Self>
                                >(x)
                            })
                        );
                    }
                }
            }
        }
        <ChangeOutputType<R, Option<Lend<'_, Self>>>>::from_output(None)
    }
    /// The [`Lender`] version of [`Iterator::position`].
    #[inline]
    fn position<P>(&mut self, mut predicate: P) -> Option<usize>
    where
        Self: Sized,
        P: FnMut(Lend<'_, Self>) -> bool,
    {
        let mut i = 0;
        while let Some(x) = self.next() {
            if predicate(x) {
                return Some(i);
            }
            i += 1;
        }
        None
    }
    /// The [`Lender`] version of [`Iterator::rposition`].
    #[inline]
    fn rposition<P>(&mut self, mut predicate: P) -> Option<usize>
    where
        P: FnMut(Lend<'_, Self>) -> bool,
        Self: Sized + ExactSizeLender + DoubleEndedLender,
    {
        match self.try_rfold(self.len(), |i, x| {
            let i = i - 1;
            if predicate(x) { ControlFlow::Break(i) } else { ControlFlow::Continue(i) }
        }) {
            ControlFlow::Continue(_) => None,
            ControlFlow::Break(x) => Some(x),
        }
    }
    /// The [`Lender`] version of [`Iterator::max`].
    #[inline]
    fn max<T>(self) -> Option<T>
    where
        Self: Sized,
        T: for<'all> PartialOrd<Lend<'all, Self>>,
        for<'all> Lend<'all, Self>: ToOwned<Owned = T>,
    {
        self.max_by(|x, y| x.partial_cmp(y).unwrap_or(Ordering::Equal))
    }
    /// The [`Lender`] version of [`Iterator::min`].
    #[inline]
    fn min<T>(self) -> Option<T>
    where
        Self: Sized,
        T: for<'all> PartialOrd<Lend<'all, Self>>,
        for<'all> Lend<'all, Self>: ToOwned<Owned = T>,
    {
        self.min_by(|x, y| x.partial_cmp(y).unwrap_or(Ordering::Equal))
    }
    /// The [`Lender`] version of [`Iterator::max_by_key`].
    #[inline]
    fn max_by_key<B: Ord, T, F>(self, f: F) -> Option<T>
    where
        Self: Sized,
        for<'all> Lend<'all, Self>: ToOwned<Owned = T>,
        F: FnMut(&T) -> B,
    {
        self.owned().max_by_key::<B, F>(f)
    }
    /// The [`Lender`] version of [`Iterator::max_by`].
    #[inline]
    fn max_by<T, F>(self, mut compare: F) -> Option<T>
    where
        Self: Sized,
        for<'all> Lend<'all, Self>: ToOwned<Owned = T>,
        F: FnMut(&T, &Lend<'_, Self>) -> Ordering,
    {
        self.reduce(move |x, y| {
            match compare(&x, &y) {
                Ordering::Greater => x,
                _ => y.to_owned(),
            }
        })
    }
    /// The [`Lender`] version of [`Iterator::min_by_key`].
    #[inline]
    fn min_by_key<B: Ord, T, F>(self, f: F) -> Option<T>
    where
        Self: Sized,
        for<'all> Lend<'all, Self>: ToOwned<Owned = T>,
        F: FnMut(&T) -> B,
    {
        self.owned().min_by_key::<B, F>(f)
    }
    /// The [`Lender`] version of [`Iterator::min_by`].
    #[inline]
    fn min_by<T, F>(self, mut compare: F) -> Option<T>
    where
        Self: Sized,
        for<'all> Lend<'all, Self>: ToOwned<Owned = T>,
        F: FnMut(&T, &Lend<'_, Self>) -> Ordering,
    {
        self.reduce(move |x, y| {
            match compare(&x, &y) {
                Ordering::Greater => y.to_owned(),
                _ => x,
            }
        })
    }
    /// The [`Lender`] version of [`Iterator::rev`].
    #[inline]
    fn rev(self) -> Rev<Self>
    where
        Self: Sized + DoubleEndedLender,
    {
        Rev::new(self)
    }
    /// The [`Lender`] version of [`Iterator::unzip`].
    #[inline]
    fn unzip<ExtA, ExtB>(self) -> (ExtA, ExtB)
    where
    Self: Sized,
    for<'all> Lend<'all, Self>: TupleLend<'all>,
    ExtA: Default + ExtendLender<FirstShunt<Self>>,
    ExtB: Default + ExtendLender<SecondShunt<Self>>, {
        unzip(self)
    }
    /// The [`Lender`] version of [`Iterator::copied`].
    ///
    /// Turns this [`Lender`] into an [`Iterator`].
    fn copied<T>(self) -> Copied<Self>
    where
        Self: Sized + for<'all> Lending<'all, Lend = &'all T>,
        T: Copy,
    {
        Copied::new(self)
    }
    /// The [`Lender`] version of [`Iterator::cloned`].
    ///
    /// Turns this [`Lender`] into an [`Iterator`].
    fn cloned<T>(self) -> Cloned<Self>
    where
        Self: Sized + for<'all> Lending<'all, Lend = &'all T>,
        T: Clone,
    {
        Cloned::new(self)
    }
    // not std::iter
    /// Turns this [`Lender`] into an [`Iterator`].
    #[inline]
    fn owned(self) -> Owned<Self>
    where
        Self: Sized,
        for<'all> Lend<'all, Self>: ToOwned
    {
        Owned::new(self)
    }
    /// The [`Lender`] version of [`Iterator::cycle`].
    #[inline]
    fn cycle(self) -> Cycle<Self>
    where
        Self: Sized + Clone,
    {
        Cycle::new(self)
    }
    /// The [`Lender`] version of [`Iterator::sum`].
    #[inline]
    fn sum<S>(self) -> S
    where
        Self: Sized,
        S: SumLender<Self>,
    {
        S::sum_lender(self)
    }
    /// The [`Lender`] version of [`Iterator::product`].
    #[inline]
    fn product<P>(self) -> P
    where
        Self: Sized,
        P: ProductLender<Self>,
    {
        P::product_lender(self)
    }
    /// The [`Lender`] version of [`Iterator::cmp`].
    fn cmp<L>(self, other: L) -> Ordering
    where
        L: IntoLender,
        L::Lender: for<'all> Lending<'all, Lend = Lend<'all, Self>>,
        for <'all> Lend<'all, Self>: Ord,
        Self: Sized,
    {
        self.cmp_by(other, |x, y| x.cmp(&y))
    }
    /// The [`Lender`] version of [`Iterator::cmp_by`].
    fn cmp_by<L, F>(self, other: L, mut cmp: F) -> Ordering
    where
        Self: Sized,
        L: IntoLender,
        F: for<'all> FnMut(Lend<'all, Self>, Lend<'all, L::Lender>) -> Ordering,
    {
        match lender_compare(self, other.into_lender(), move |x, y| match cmp(x, y) {
            Ordering::Equal => ControlFlow::Continue(()),
            neq => ControlFlow::Break(neq),
        }) {
            ControlFlow::Continue(ord) => ord,
            ControlFlow::Break(ord) => ord,
        }
    }
    /// The [`Lender`] version of [`Iterator::partial_cmp`].
    fn partial_cmp<L>(self, other: L) -> Option<Ordering>
    where
        L: IntoLender,
        for<'all> Lend<'all, Self>: PartialOrd<Lend<'all, L::Lender>>,
        Self: Sized,
    {
        self.partial_cmp_by(other, |x, y| x.partial_cmp(&y))
    }
    /// The [`Lender`] version of [`Iterator::partial_cmp_by`].
    fn partial_cmp_by<L, F>(self, other: L, mut partial_cmp: F) -> Option<Ordering>
    where
        Self: Sized,
        L: IntoLender,
        F: for<'all> FnMut(Lend<'all, Self>, Lend<'all, L::Lender>) -> Option<Ordering>,
    {
        match lender_compare(self, other.into_lender(), move |x, y| match partial_cmp(x, y) {
            Some(Ordering::Equal) => ControlFlow::Continue(()),
            neq => ControlFlow::Break(neq),
        }) {
            ControlFlow::Continue(ord) => Some(ord),
            ControlFlow::Break(ord) => ord,
        }
    }
    /// The [`Lender`] version of [`Iterator::eq`].
    fn eq<L>(self, other: L) -> bool
    where
        L: IntoLender,
        for<'all> Lend<'all, Self>: PartialEq<Lend<'all, L::Lender>>,
        Self: Sized,
    {
        self.eq_by(other, |x, y| x == y)
    }
    /// The [`Lender`] version of [`Iterator::eq_by`].
    fn eq_by<L, F>(self, other: L, mut eq: F) -> bool
    where
        Self: Sized,
        L: IntoLender,
        F: for<'all> FnMut(Lend<'all, Self>, Lend<'all, L::Lender>) -> bool,
    {
        match lender_compare(self, other.into_lender(), move |x, y| {
            if eq(x, y) { ControlFlow::Continue(()) } else { ControlFlow::Break(()) }
        }) {
            ControlFlow::Continue(ord) => ord == Ordering::Equal,
            ControlFlow::Break(()) => false,
        }
    }
    /// The [`Lender`] version of [`Iterator::ne`].
    fn ne<L>(self, other: L) -> bool
    where
        L: IntoLender,
        for<'all> Lend<'all, Self>: PartialEq<Lend<'all, L::Lender>>,
        Self: Sized,
    {
        !self.eq(other)
    }
    /// The [`Lender`] version of [`Iterator::lt`].
    fn lt<L>(self, other: L) -> bool
    where
        L: IntoLender,
        for<'all> Lend<'all, Self>: PartialOrd<Lend<'all, L::Lender>>,
        Self: Sized,
    {
        self.partial_cmp(other) == Some(Ordering::Less)
    }
    /// The [`Lender`] version of [`Iterator::le`].
    fn le<L>(self, other: L) -> bool
    where
        L: IntoLender,
        for<'all> Lend<'all, Self>: PartialOrd<Lend<'all, L::Lender>>,
        Self: Sized,
    {
        matches!(self.partial_cmp(other), Some(Ordering::Less | Ordering::Equal))
    }
    /// The [`Lender`] version of [`Iterator::gt`].
    fn gt<L>(self, other: L) -> bool
    where
        L: IntoLender,
        for<'all> Lend<'all, Self>: PartialOrd<Lend<'all, L::Lender>>,
        Self: Sized,
    {
        self.partial_cmp(other) == Some(Ordering::Greater)
    }
    /// The [`Lender`] version of [`Iterator::ge`].
    fn ge<L>(self, other: L) -> bool
    where
        L: IntoLender,
        for<'all> Lend<'all, Self>: PartialOrd<Lend<'all, L::Lender>>,
        Self: Sized,
    {
        matches!(self.partial_cmp(other), Some(Ordering::Greater | Ordering::Equal))
    }
    /// The [`Lender`] version of [`Iterator::is_sorted`].
    #[inline]
    #[allow(clippy::wrong_self_convention)]
    fn is_sorted<T>(self) -> bool
    where
        Self: Sized,
        for<'all> Lend<'all, Self>: ToOwned<Owned = T>,
        T: PartialOrd,
    {
        self.is_sorted_by(PartialOrd::partial_cmp)
    }
    /// The [`Lender`] version of [`Iterator::is_sorted_by`].
    #[inline]
    #[allow(clippy::wrong_self_convention)]
    fn is_sorted_by<T, F>(self, mut compare: F) -> bool
    where
        Self: Sized,
        for<'all> Lend<'all, Self>: ToOwned<Owned = T>,
        F: FnMut(&T, &T) -> Option<Ordering>,
    {
        let mut this = self.owned();
        let Some(mut last) = this.next() else {
            return true;
        };
        this.all(move |curr| {
            if let Some(Ordering::Greater) | None = compare(&last, &curr) {
                return false;
            }
            last = curr;
            true
        })
    }
    /// The [`Lender`] version of [`Iterator::is_sorted_by_key`].
    #[inline]
    #[allow(clippy::wrong_self_convention)]
    fn is_sorted_by_key<F, K>(mut self, mut f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Lend<'_, Self>) -> K,
        K: PartialOrd,
    {
        let mut last = match self.next() {
            None => return true,
            Some(x) => f(x),
        };
        while let Some(x) = self.next() {
            let curr = f(x);
            if let Some(Ordering::Greater) | None = last.partial_cmp(&curr) {
                return false;
            }
            last = curr;
        }
        true
    }
    /// Turns this [`Lender`] into an [`Iterator`] where it has already fulfilled the requirements of the [`Iterator`] trait.
    #[inline]
    fn iter<'this>(self) -> Iter<'this, Self>
    where
        Self: Sized + 'this,
        for<'all> Lend<'all, Self>: 'this,
    {
        Iter::new(self)
    }
    /// A lending replacement for [`Iterator::array_chunks`].
    ///
    /// It is not possible to implement the exact behavior of
    /// [`Iterator::array_chunks`] in a lender, so this is the closest approximation:
    /// at each iteration, it yields a lender returning the next `chunk_size` lends.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_iter::<lend!(&'lend u8), _>([1u8, 2, 3].iter());
    /// let mut chunky = lender.chunky(2);
    /// let mut chunk1 = chunky.next().unwrap();
    /// assert_eq!(chunk1.next(), Some(&1));
    /// assert_eq!(chunk1.next(), Some(&2));
    /// assert_eq!(chunk1.next(), None);
    /// let mut chunk2 = chunky.next().unwrap();
    /// assert_eq!(chunk2.next(), Some(&3));
    /// assert_eq!(chunk2.next(), None);
    /// ```
    #[inline]
    fn chunky(self, chunk_size: usize) -> Chunky<Self>
    where
        Self: Sized + ExactSizeLender,
    {
        Chunky::new(self, chunk_size)
    }

    /// Converts a [`Lender`] whose lend type is `Result<T, E>` into a
    /// [`FallibleLender`] with error type `E` and lend type `T`.
    ///
    /// This is the lending equivalent of
    /// [`fallible_iterator::convert`](https://docs.rs/fallible-iterator/latest/fallible_iterator/fn.convert.html).
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lender::prelude::*;
    /// let data = vec![Ok(1), Ok(2), Err("oops")];
    /// let mut lender = lender::from_iter(data.into_iter())
    ///     .convert::<&str>();
    /// assert_eq!(lender.next(), Ok(Some(1)));
    /// assert_eq!(lender.next(), Ok(Some(2)));
    /// assert!(lender.next().is_err());
    /// ```
    #[inline]
    fn convert<E>(self) -> Convert<E, Self>
    where
        Self: Sized,
    {
        Convert::new(self)
    }

    /// Converts a [`Lender`] into a [`FallibleLender`] by wrapping
    /// into `Result<Lend<'_, Self>, E>` where `E` is an
    /// error that can never actually happen.
    fn into_fallible<E>(self) -> IntoFallible<E, Self> where Self: Sized {
        IntoFallible::new(self)
    }
}

#[inline]
pub(crate) fn lender_compare<A, B, F, T>(mut a: A, mut b: B, mut f: F) -> ControlFlow<T, Ordering>
where
    A: Lender,
    B: Lender,
    for<'all> F: FnMut(Lend<'all, A>, Lend<'all, B>) -> ControlFlow<T>,
{
    let mut ctl = ControlFlow::Continue(());
    while let Some(x) = a.next() {
        match b.next() {
            None => {
                ctl = ControlFlow::Break(ControlFlow::Continue(Ordering::Greater));
                break;
            }
            Some(y) => {
                if let ControlFlow::Break(x) = f(x, y) {
                    ctl = ControlFlow::Break(ControlFlow::Break(x));
                    break;
                }
            }
        }
    }
    match ctl {
        ControlFlow::Continue(()) => ControlFlow::Continue(match b.next() {
            None => Ordering::Equal,
            Some(_) => Ordering::Less,
        }),
        ControlFlow::Break(x) => x,
    }
}

impl<'lend, L: Lender> Lending<'lend> for &mut L {
    type Lend = Lend<'lend, L>;
}

impl<L: Lender> Lender for &mut L {
    // SAFETY: the lend is that of L
    crate::unsafe_assume_covariance!();
    #[inline(always)]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        (**self).next()
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (**self).size_hint()
    }

    #[inline(always)]
    fn advance_by(&mut self, n: usize) -> Result<(), NonZeroUsize> {
        (**self).advance_by(n)
    }
}
