use alloc::borrow::ToOwned;
use core::{cmp::Ordering, num::NonZeroUsize, ops::ControlFlow};

use crate::{
    higher_order::{FnMutHKA, FnMutHKAOpt},
    try_trait_v2::{ChangeOutputType, FromResidual, Residual, Try},
    *,
};

/// A trait necessary for implementing `Lender`. It implicitly restricts the lifetime `'lend` used in `Lending<'lend>` to be where `Self: 'lend`.
///
/// This is a result of Higher-Ranked Trait Bounds (HRTBs) not having a way to express qualifiers (```for<'any where Self: 'any> Self: Trait```)
/// and effectively making HRTBs only useful when you want to express a trait constraint on ALL lifetimes, including 'static (```for<'all> Self: trait```)
///
/// Although the common example of implementing your own LendingIterator uses a (```type Item<'a> where Self: 'a;```) GAT,
/// that generally only works withing a small subset of the features that a LendingIterator needs to provide to be useful.
///
/// Please see [Sabrina Jewson's Blog][1] for more information, and how a trait like this can be used to solve it by implicitly restricting HRTBs.
///
/// [1]: (https://sabrinajewson.org/blog/the-better-alternative-to-lifetime-gats)
pub trait Lending<'lend, __Seal: Sealed = Seal<&'lend Self>> {
    type Lend: 'lend;
}

/// An iterator that yields items that live at least as long as the iterator itself or until the next item is yielded.
///
/// A `Lender` cannot be used as a `dyn` trait object, because it is used `Self` as a type parameter to get around higher-ranked lifetime bounds.
///
/// Turn a lender into an iterator with [`cloned()`](Lender::cloned) where lend is [`Clone`], [`copied()`](Lender::copied) where lend is [`Copy`], [`owned()`](Lender::owned) where lend is [`ToOwned`], or [`iter()`](Lender::iter) where lend is already owned.
///
/// Both [`Iterator::partition_in_place`] and [`Iterator::array_chunks`] APIs are not supported by lending iterators.
///
/// # Examples
///
/// ```rust
/// use lender::prelude::*;
/// struct WindowsMut<'a, T> {
///    slice: &'a mut [T],
///    cur: usize,
///    window_size: usize,
/// }
/// impl<'lend, 'a, T> Lending<'lend> for WindowsMut<'a, T> {
///    type Lend = &'lend mut [T];
/// }
/// impl<'a, T> Lender for WindowsMut<'a, T> {
///   fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> {
///     if let elt @ Some(_) = self.slice.get_mut(self.cur..self.cur + self.window_size) {
///       self.cur += 1;
///       elt
///     } else {
///       None
///     }
///   }
/// }
/// ```
pub trait Lender: for<'all /* where Self: 'all */> Lending<'all> {
    /// Yield the next lend, if any, of the lender.
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
    fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend>;
    /// Take the next `len` lends of the lender with temporary lender `Chunk`. This is the quivalent of cloning the lender and calling `take(len)` on it.
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
    /// Get the estimated minimum and maximum length of the lender. Use `.len()` for the exact length if the lender implements `ExactSizeLender`.
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
    /// Count the number of lends in the lender by consuming it.
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
    /// Get the last lend of the lender, if any, by consuming it.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lender::prelude::*;
    /// let mut lender = lender::lend_iter::<lend!(&'lend u8), _>([1, 2, 3u8].iter());
    /// assert_eq!(lender.last(), Some(&3));
    /// ```
    #[inline]
    fn last<'call>(mut self) -> Option<<Self as Lending<'call>>::Lend>
    where
        Self: Sized,
    {
        let mut last = None;
        while let Some(x) = self.next() {
            // SAFETY: polonius return
            last = Some(unsafe {
                core::mem::transmute::<
                    <Self as Lending<'_>>::Lend,
                    <Self as Lending<'call>>::Lend
                >(x)
            });
        }
        last
    }
    /// Advance the lender by `n` lends. If the lender does not have enough lends, return the number of lends left.
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
    /// Yield the nth lend of the lender, if any, by consuming it. If the lender does not have enough lends, returns `None`.
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
    fn nth(&mut self, n: usize) -> Option<<Self as Lending<'_>>::Lend> {
        self.advance_by(n).ok()?;
        self.next()
    }
    /// Skip `step - 1` lends between each lend of the lender.
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
    /// Chain the lender with another lender of the same type.
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
    fn chain<U: IntoLender>(self, other: U) -> Chain<Self, <U as IntoLender>::Lender>
    where
        Self: Sized,
        for<'all> U: Lending<'all, Lend = <Self as Lending<'all>>::Lend>,
    {
        Chain::new(self, other.into_lender())
    }
    /// Zip the lender with another lender of the same or different type.
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
    /// Intersperse each lend of this lender with the given seperator.
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
    fn intersperse<'call>(self, separator: <Self as Lending<'call>>::Lend) -> Intersperse<'call, Self>
    where
        Self: Sized,
        for<'all> <Self as Lending<'all>>::Lend: Clone,
    {
        Intersperse::new(self, separator)
    }
    /// Intersperse each lend of this lender with the seperator produced by the given function.
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
        G: FnMut() -> <Self as Lending<'call>>::Lend,
    {
        IntersperseWith::new(self, separator)
    }
    /// Map each lend of this lender using the given function.
    ///
    /// Please note that it is likely required that you use the [`hrc_mut!`] macro to create the closure.
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
        F: for<'all> FnMutHKA<'all, <Self as Lending<'all>>::Lend>,
    {
        Map::new(self, f)
    }
    /// Call the given function with each lend of this lender.
    ///
    /// While for most situations a basic while loop will suffice,
    /// for long chains of operations it is recommended to use [`for_each`] method instead for readability.
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
    fn for_each<F>(mut self, mut f: F)
    where
        Self: Sized,
        F: FnMut(<Self as Lending<'_>>::Lend),
    {
        while let Some(a) = self.next() {
            f(a);
        }
    }
    /// Filter this lender using the given predicate.
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
        P: FnMut(&<Self as Lending<'_>>::Lend) -> bool,
    {
        Filter::new(self, predicate)
    }
    /// Filter and map this lender using the given function.
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
        F: for<'all> FnMutHKAOpt<'all, <Self as Lending<'all>>::Lend>,
    {
        FilterMap::new(self, f)
    }
    /// Enumerate this lender. Each lend is paired with its zero-based index.
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
    /// Make this lender peekable, so that it is possible to peek at the next lend without consuming it.
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
    /// Skip the first contiguous sequence lends of this lender that satisfy the given predicate.
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
        P: FnMut(&<Self as Lending<'_>>::Lend) -> bool,
    {
        SkipWhile::new(self, predicate)
    }
    /// Take the first contiguous sequence lends of this lender that satisfy the given predicate.
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
        P: FnMut(&<Self as Lending<'_>>::Lend) -> bool,
    {
        TakeWhile::new(self, predicate)
    }
    /// Map this lender using the given function while it returns `Some`.
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
        P: for<'all> FnMutHKAOpt<'all, <Self as Lending<'all>>::Lend>,
    {
        MapWhile::new(self, predicate)
    }
    /// Skip the first `n` lends of this lender.
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
    /// Take the first `n` lends of this lender.
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
    /// Refer to [`Iterator::scan`] for more information.
    #[inline]
    fn scan<St, F>(self, initial_state: St, f: F) -> Scan<Self, St, F>
    where
        Self: Sized,
        F: for<'all> FnMutHKAOpt<'all, (&'all mut St, <Self as Lending<'all>>::Lend)>,
    {
        Scan::new(self, initial_state, f)
    }
    /// Refer to [`Iterator::flat_map`] for more information
    #[inline]
    fn flat_map<'call, F>(self, f: F) -> FlatMap<'call, Self, F>
    where
        Self: Sized,
        F: for<'all> FnMutHKA<'all, <Self as Lending<'all>>::Lend>,
        for<'all> <F as FnMutHKA<'all, <Self as Lending<'all>>::Lend>>::B: IntoLender,
    {
        FlatMap::new(self, f)
    }
    /// Refer to [`Iterator::flatten`] for more information
    #[inline]
    fn flatten<'call>(self) -> Flatten<'call, Self>
    where
        Self: Sized,
        for<'all> <Self as Lending<'all>>::Lend: IntoLender,
    {
        Flatten::new(self)
    }
    /// Refer to [`Iterator::fuse`] for more information
    #[inline]
    fn fuse(self) -> Fuse<Self>
    where
        Self: Sized,
    {
        Fuse::new(self)
    }
    /// Refer to [`Iterator::inspect`] for more information
    #[inline]
    fn inspect<F>(self, f: F) -> Inspect<Self, F>
    where
        Self: Sized,
        F: FnMut(&<Self as Lending<'_>>::Lend),
    {
        Inspect::new(self, f)
    }
    // not std::iter
    /// Mutate each lend with the given function.
    #[inline]
    fn mutate<F>(self, f: F) -> Mutate<Self, F>
    where
        Self: Sized,
        F: FnMut(&mut <Self as Lending<'_>>::Lend),
    {
        Mutate::new(self, f)
    }
    fn by_ref(&mut self) -> &mut Self
    where
        Self: Sized,
    {
        self
    }
    /// Refer to [`Iterator::collect`] for more information
    #[inline]
    fn collect<B>(self) -> B
    where
        Self: Sized,
        B: FromLender<Self>,
    {
        B::from_lender(self)
    }
    /// Refer to [`Iterator::try_collect`] for more information
    #[inline]
    fn try_collect<'a, B>(&'a mut self) -> ChangeOutputType<<Self as Lending<'a>>::Lend, B>
    where
        Self: Sized,
        for<'all> <Self as Lending<'all>>::Lend: Try,
        for<'all> <<Self as Lending<'all>>::Lend as Try>::Residual: Residual<B>,
        B: FromLender<TryShunt<'a, &'a mut Self>>,
    {
        try_process::<&'a mut Self, _, B>(self.by_ref(),|shunt: TryShunt<'a, &'a mut Self>| B::from_lender(shunt))
    }
    /// Refer to [`Iterator::collect_into`] for more information
    #[inline]
    fn collect_into<E>(self, collection: &mut E) -> &mut E
    where
        Self: Sized,
        E: ExtendLender<Self>,
    {
        collection.extend_lender(self);
        collection
    }
    fn partition<A, E, F>(mut self, mut f: F) -> (E, E)
    where
        Self: Sized,
        E: Default + ExtendLender<Self>,
        F: FnMut(&<Self as Lending<'_>>::Lend) -> bool,
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
    /// Refer to [`Iterator::is_partitioned`] for more information
    #[inline]
    fn is_partitioned<P>(mut self, mut predicate: P) -> bool
    where
        Self: Sized,
        P: FnMut(<Self as Lending<'_>>::Lend) -> bool,
    {
        self.all(&mut predicate) || !self.any(predicate)
    }
    /// Refer to [`Iterator::try_fold`] for more information
    #[inline]
    fn try_fold<B, F, R>(&mut self, init: B, mut f: F) -> R
    where
        Self: Sized,
        F: FnMut(B, <Self as Lending<'_>>::Lend) -> R,
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
    /// Refer to [`Iterator::try_for_each`] for more information
    #[inline]
    fn try_for_each<F, R>(&mut self, mut f: F) -> R
    where
        Self: Sized,
        F: FnMut(<Self as Lending<'_>>::Lend) -> R,
        R: Try<Output = ()>,
    {
        while let Some(x) = self.next() {
            if let ControlFlow::Break(x) = f(x).branch() {
                return R::from_residual(x);
            }
        }
        R::from_output(())
    }
    /// Refer to [`Iterator::fold`] for more information
    #[inline]
    fn fold<B, F>(mut self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, <Self as Lending<'_>>::Lend) -> B,
    {
        let mut accum = init;
        while let Some(x) = self.next() {
            accum = f(accum, x);
        }
        accum
    }
    /// Refer to [`Iterator::reduce`] for more information
    #[inline]
    fn reduce<T, F>(mut self, f: F) -> Option<T>
    where
        Self: Sized,
        for<'all> <Self as Lending<'all>>::Lend: ToOwned<Owned = T>,
        F: FnMut(T, <Self as Lending<'_>>::Lend) -> T,
    {
        let first = self.next()?.to_owned();
        Some(self.fold(first, f))
    }
    /// Refer to [`Iterator::try_reduce`] for more information
    #[inline]
    fn try_reduce<T, F, R>(mut self, f: F) -> ChangeOutputType<R, Option<T>>
    where
        Self: Sized,
        for<'all> <Self as Lending<'all>>::Lend: ToOwned<Owned = T>,
        F: FnMut(T, <Self as Lending<'_>>::Lend) -> R,
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
    /// Refer to [`Iterator::all`] for more information
    #[inline]
    fn all<F>(&mut self, mut f: F) -> bool
    where
        Self: Sized,
        F: FnMut(<Self as Lending<'_>>::Lend) -> bool,
    {
        while let Some(x) = self.next() {
            if !f(x) {
                return false;
            }
        }
        true
    }
    /// Refer to [`Iterator::any`] for more information
    #[inline]
    fn any<F>(&mut self, mut f: F) -> bool
    where
        Self: Sized,
        F: FnMut(<Self as Lending<'_>>::Lend) -> bool,
    {
        while let Some(x) = self.next() {
            if f(x) {
                return true;
            }
        }
        false
    }
    /// Refer to [`Iterator::find`] for more information
    #[inline]
    fn find<P>(&mut self, mut predicate: P) -> Option<<Self as Lending<'_>>::Lend>
    where
        Self: Sized,
        P: FnMut(&<Self as Lending<'_>>::Lend) -> bool,
    {
        while let Some(x) = self.next() {
            if predicate(&x) {
                // SAFETY: polonius return
                return Some(unsafe {
                    core::mem::transmute::<
                        <Self as Lending<'_>>::Lend,
                        <Self as Lending<'_>>::Lend
                    >(x)
                });
            }
        }
        None
    }
    /// Refer to [`Iterator::find_map`] for more information
    #[inline]
    fn find_map<'a, F>(&'a mut self, mut f: F) -> Option<<F as FnMutHKAOpt<'a, <Self as Lending<'a>>::Lend>>::B>
    where
        Self: Sized,
        F: for<'all> FnMutHKAOpt<'all, <Self as Lending<'all>>::Lend>,
    {
        while let Some(x) = self.next() {
            if let Some(y) = f(x) {
                // SAFETY: polonius return
                return Some(unsafe {
                    core::mem::transmute::<
                        <F as FnMutHKAOpt<'_, <Self as Lending<'_>>::Lend>>::B,
                        <F as FnMutHKAOpt<'a, <Self as Lending<'a>>::Lend>>::B
                    >(y)
                });
            }
        }
        None
    }
    /// Refer to [`Iterator::try_find`] for more information
    #[inline]
    fn try_find<F, R>(&mut self, mut f: F) -> ChangeOutputType<R, Option<<Self as Lending<'_>>::Lend>>
    where
        Self: Sized,
        F: FnMut(&<Self as Lending<'_>>::Lend) -> R,
        R: Try<Output = bool>,
        for<'all> R::Residual: Residual<Option<<Self as Lending<'all>>::Lend>>,
    {
        while let Some(x) = self.next() {
            match f(&x).branch() {
                ControlFlow::Break(x) => return <ChangeOutputType<R, Option<<Self as Lending<'_>>::Lend>>>::from_residual(x),
                ControlFlow::Continue(cond) => {
                    if cond {
                        // SAFETY: polonius return
                        return <ChangeOutputType<R, Option<<Self as Lending<'_>>::Lend>>>::from_output(
                            Some(unsafe {
                                core::mem::transmute::<
                                    <Self as Lending<'_>>::Lend,
                                    <Self as Lending<'_>>::Lend
                                >(x)
                            })
                        );
                    }
                }
            }
        }
        <ChangeOutputType<R, Option<<Self as Lending<'_>>::Lend>>>::from_output(None)
    }
    /// Refer to [`Iterator::position`] for more information
    #[inline]
    fn position<P>(&mut self, mut predicate: P) -> Option<usize>
    where
        Self: Sized,
        P: FnMut(<Self as Lending<'_>>::Lend) -> bool,
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
    /// Refer to [`Iterator::rposition`] for more information
    #[inline]
    fn rposition<P>(&mut self, mut predicate: P) -> Option<usize>
    where
        P: FnMut(<Self as Lending<'_>>::Lend) -> bool,
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
    /// Refer to [`Iterator::max`] for more information
    #[inline]
    fn max<T>(self) -> Option<T>
    where
        Self: Sized,
        T: for<'all> PartialOrd<<Self as Lending<'all>>::Lend>,
        for<'all> <Self as Lending<'all>>::Lend: ToOwned<Owned = T>,
    {
        self.max_by(|x, y| x.partial_cmp(y).unwrap_or(Ordering::Equal))
    }
    /// Refer to [`Iterator::min`] for more information
    #[inline]
    fn min<T>(self) -> Option<T>
    where
        Self: Sized,
        T: for<'all> PartialOrd<<Self as Lending<'all>>::Lend>,
        for<'all> <Self as Lending<'all>>::Lend: ToOwned<Owned = T>,
    {
        self.min_by(|x, y| x.partial_cmp(y).unwrap_or(Ordering::Equal))
    }
    /// Refer to [`Iterator::max_by_key`] for more information
    #[inline]
    fn max_by_key<B: Ord, T, F>(self, f: F) -> Option<T>
    where
        Self: Sized,
        for<'all> <Self as Lending<'all>>::Lend: ToOwned<Owned = T>,
        F: FnMut(&T) -> B,
    {
        self.owned().max_by_key::<B, F>(f)
    }
    /// Refer to [`Iterator::max_by`] for more information
    #[inline]
    fn max_by<T, F>(self, mut compare: F) -> Option<T>
    where
        Self: Sized,
        for<'all> <Self as Lending<'all>>::Lend: ToOwned<Owned = T>,
        F: FnMut(&T, &<Self as Lending<'_>>::Lend) -> Ordering,
    {
        self.reduce(move |x, y| {
            match compare(&x, &y) {
                Ordering::Less => y.to_owned(),
                _ => x,
            }
        })
    }
    /// Refer to [`Iterator::min_by_key`] for more information
    #[inline]
    fn min_by_key<B: Ord, T, F>(self, f: F) -> Option<T>
    where
        Self: Sized,
        for<'all> <Self as Lending<'all>>::Lend: ToOwned<Owned = T>,
        F: FnMut(&T) -> B,
    {
        self.owned().min_by_key::<B, F>(f)
    }
    /// Refer to [`Iterator::min_by`] for more information
    #[inline]
    fn min_by<T, F>(self, mut compare: F) -> Option<T>
    where
        Self: Sized,
        for<'all> <Self as Lending<'all>>::Lend: ToOwned<Owned = T>,
        F: FnMut(&T, &<Self as Lending<'_>>::Lend) -> Ordering,
    {
        self.reduce(move |x, y| {
            match compare(&x, &y) {
                Ordering::Greater => y.to_owned(),
                _ => x,
            }
        })
    }
    /// Refer to [`Iterator::rev`] for more information
    #[inline]
    fn rev(self) -> Rev<Self>
    where
        Self: Sized + DoubleEndedLender,
    {
        Rev::new(self)
    }
    /// Refer to [`Iterator::unzip`] for more information
    #[inline]
    fn unzip<ExtA, ExtB>(self) -> (ExtA, ExtB)
    where
    Self: Sized,
    for<'all> <Self as Lending<'all>>::Lend: TupleLend<'all>,
    ExtA: Default + ExtendLender<FirstShunt<Self>>,
    ExtB: Default + ExtendLender<SecondShunt<Self>>, {
        unzip(self)
    }
    /// Refer to [`Iterator::copied`] for more information.
    ///
    /// Turns this Lender into an Iterator.
    fn copied<T>(self) -> Copied<Self>
    where
        Self: Sized + for<'all> Lending<'all, Lend = &'all T>,
        T: Copy,
    {
        Copied::new(self)
    }
    /// Refer to [`Iterator::cloned`] for more information.
    ///
    /// Turns this Lender into an Iterator.
    fn cloned<T>(self) -> Cloned<Self>
    where
        Self: Sized + for<'all> Lending<'all, Lend = &'all T>,
        T: Clone,
    {
        Cloned::new(self)
    }
    // not std::iter
    /// Turn this Lender into an Iterator.
    #[inline]
    fn owned(self) -> Owned<Self>
    where
        Self: Sized,
        for<'all> <Self as Lending<'all>>::Lend: ToOwned
    {
        Owned::new(self)
    }
    /// Refer to [`Iterator::cycle`] for more information
    #[inline]
    fn cycle(self) -> Cycle<Self>
    where
        Self: Sized + Clone,
    {
        Cycle::new(self)
    }
    /// Refer to [`Iterator::sum`] for more information
    #[inline]
    fn sum<S>(self) -> S
    where
        Self: Sized,
        S: SumLender<Self>,
    {
        S::sum_lender(self)
    }
    /// Refer to [`Iterator::product`] for more information
    #[inline]
    fn product<P>(self) -> P
    where
        Self: Sized,
        P: ProductLender<Self>,
    {
        P::product_lender(self)
    }
    /// Refer to [`Iterator::cmp`] for more information
    fn cmp<L>(self, other: L) -> Ordering
    where
        L: IntoLender + for<'all> Lending<'all, Lend = <Self as Lending<'all>>::Lend>,
        for <'all> <Self as Lending<'all>>::Lend: Ord,
        Self: Sized,
    {
        self.cmp_by(other, |x, y| x.cmp(&y))
    }
    /// Refer to [`Iterator::cmp_by`] for more information
    fn cmp_by<L, F>(self, other: L, mut cmp: F) -> Ordering
    where
        Self: Sized,
        L: IntoLender,
        F: for<'all> FnMut(<Self as Lending<'all>>::Lend, <L as Lending<'all>>::Lend) -> Ordering,
    {
        match lender_compare(self, other.into_lender(), move |x, y| match cmp(x, y) {
            Ordering::Equal => ControlFlow::Continue(()),
            neq => ControlFlow::Break(neq),
        }) {
            ControlFlow::Continue(ord) => ord,
            ControlFlow::Break(ord) => ord,
        }
    }
    /// Refer to [`Iterator::partial_cmp`] for more information
    fn partial_cmp<L>(self, other: L) -> Option<Ordering>
    where
        L: IntoLender,
        for<'all> <Self as Lending<'all>>::Lend: PartialOrd<<L as Lending<'all>>::Lend>,
        Self: Sized,
    {
        self.partial_cmp_by(other, |x, y| x.partial_cmp(&y))
    }
    /// Refer to [`Iterator::partial_cmp_by`] for more information
    fn partial_cmp_by<L, F>(self, other: L, mut partial_cmp: F) -> Option<Ordering>
    where
        Self: Sized,
        L: IntoLender,
        F: for<'all> FnMut(<Self as Lending<'all>>::Lend, <L as Lending<'all>>::Lend) -> Option<Ordering>,
    {
        match lender_compare(self, other.into_lender(), move |x, y| match partial_cmp(x, y) {
            Some(Ordering::Equal) => ControlFlow::Continue(()),
            neq => ControlFlow::Break(neq),
        }) {
            ControlFlow::Continue(ord) => Some(ord),
            ControlFlow::Break(ord) => ord,
        }
    }
    /// Refer to [`Iterator::eq`] for more information
    fn eq<L>(self, other: L) -> bool
    where
        L: IntoLender,
        for<'all> <Self as Lending<'all>>::Lend: PartialEq<<L as Lending<'all>>::Lend>,
        Self: Sized,
    {
        self.eq_by(other, |x, y| x == y)
    }
    /// Refer to [`Iterator::eq_by`] for more information
    fn eq_by<L, F>(self, other: L, mut eq: F) -> bool
    where
        Self: Sized,
        L: IntoLender,
        F: for<'all> FnMut(<Self as Lending<'all>>::Lend, <L as Lending<'all>>::Lend) -> bool,
    {
        match lender_compare(self, other.into_lender(), move |x, y| {
            if eq(x, y) { ControlFlow::Continue(()) } else { ControlFlow::Break(()) }
        }) {
            ControlFlow::Continue(ord) => ord == Ordering::Equal,
            ControlFlow::Break(()) => false,
        }
    }
    /// Refer to [`Iterator::ne`] for more information
    fn ne<L>(self, other: L) -> bool
    where
        L: IntoLender,
        for<'all> <Self as Lending<'all>>::Lend: PartialEq<<L as Lending<'all>>::Lend>,
        Self: Sized,
    {
        !self.eq(other)
    }
    /// Refer to [`Iterator::lt`] for more information
    fn lt<L>(self, other: L) -> bool
    where
        L: IntoLender,
        for<'all> <Self as Lending<'all>>::Lend: PartialOrd<<L as Lending<'all>>::Lend>,
        Self: Sized,
    {
        self.partial_cmp(other) == Some(Ordering::Less)
    }
    /// Refer to [`Iterator::le`] for more information
    fn le<L>(self, other: L) -> bool
    where
        L: IntoLender,
        for<'all> <Self as Lending<'all>>::Lend: PartialOrd<<L as Lending<'all>>::Lend>,
        Self: Sized,
    {
        matches!(self.partial_cmp(other), Some(Ordering::Less | Ordering::Equal))
    }
    /// Refer to [`Iterator::gt`] for more information
    fn gt<L>(self, other: L) -> bool
    where
        L: IntoLender,
        for<'all> <Self as Lending<'all>>::Lend: PartialOrd<<L as Lending<'all>>::Lend>,
        Self: Sized,
    {
        self.partial_cmp(other) == Some(Ordering::Greater)
    }
    /// Refer to [`Iterator::ge`] for more information
    fn ge<L>(self, other: L) -> bool
    where
        L: IntoLender,
        for<'all> <Self as Lending<'all>>::Lend: PartialOrd<<L as Lending<'all>>::Lend>,
        Self: Sized,
    {
        matches!(self.partial_cmp(other), Some(Ordering::Greater | Ordering::Equal))
    }
    /// Refer to [`Iterator::is_sorted`] for more information
    #[inline]
    fn is_sorted<T>(self) -> bool
    where
        Self: Sized,
        for<'all> <Self as Lending<'all>>::Lend: ToOwned<Owned = T>,
        T: PartialOrd,
    {
        self.is_sorted_by(PartialOrd::partial_cmp)
    }
    /// Refer to [`Iterator::is_sorted_by`] for more information
    #[inline]
    fn is_sorted_by<T, F>(self, mut compare: F) -> bool
    where
        Self: Sized,
        for<'all> <Self as Lending<'all>>::Lend: ToOwned<Owned = T>,
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
    /// Refer to [`Iterator::is_sorted_by_key`] for more information
    #[inline]
    fn is_sorted_by_key<F, K>(mut self, mut f: F) -> bool
    where
        Self: Sized,
        F: FnMut(<Self as Lending<'_>>::Lend) -> K,
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
    /// Turn into an Iterator where the lender has fulfilled the requirements of Iterator.
    #[inline]
    fn iter<'this>(self) -> Iter<'this, Self>
    where
        Self: Sized + 'this,
        for<'all> <Self as Lending<'all>>::Lend: 'this,
    {
        Iter::new(self)
    }
    /// Make this lender nice and chunky. (Adapter that calls `lend_chunk` on the lender on `next`)
    #[inline]
    fn chunky(self, chunk_size: usize) -> Chunky<Self>
    where
        Self: Sized + ExactSizeLender,
    {
        Chunky::new(self, chunk_size)
    }
}

#[inline]
pub(crate) fn lender_compare<A, B, F, T>(mut a: A, mut b: B, mut f: F) -> ControlFlow<T, Ordering>
where
    A: Lender,
    B: Lender,
    for<'all> F: FnMut(<A as Lending<'all>>::Lend, <B as Lending<'all>>::Lend) -> ControlFlow<T>,
{
    let mut ctl = ControlFlow::Continue(());
    while let Some(x) = a.next() {
        match b.next() {
            None => {
                ctl = ControlFlow::Break(ControlFlow::Continue(Ordering::Greater));
                break;
            }
            Some(y) => {
                let this = f(x, y);
                let f = ControlFlow::Break;
                if let ControlFlow::Break(x) = this {
                    ctl = ControlFlow::Break(f(x));
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
    type Lend = <L as Lending<'lend>>::Lend;
}
impl<L: Lender> Lender for &mut L {
    #[inline]
    fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> { (**self).next() }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) { (**self).size_hint() }
    #[inline]
    fn advance_by(&mut self, n: usize) -> Result<(), NonZeroUsize> { (**self).advance_by(n) }
}
