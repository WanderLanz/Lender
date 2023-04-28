use alloc::borrow::ToOwned;
use core::{cmp::Ordering, num::NonZeroUsize, ops::ControlFlow};

use crate::{
    hkts::{HKAFnMut, HKGFnMut},
    try_trait_v2::{ChangeOutputType, FromResidual, Residual, Try},
    *,
};

/// A trait necessary for implementing `Lender`.
///
/// This is a result of Higher-Ranked Trait Bounds (HRTBs) not having a way to express qualifiers (```for<'any where Self: 'any> Self: Trait```)
/// and effectively making HRTBs only useful when you want to express a trait constraint on ALL lifetimes, including 'static (```for<'all> Self: trait```)
///
/// Although the common example of implementing your own LendingIterator uses a (```type Item<'a> where Self: 'a;```) GAT,
/// that generally only works withing a small subset of the features that a LendingIterator needs to provide to be useful.
///
/// Please see [Sabrina Jewson's Blog][1] for more information on the problem and how a trait like this can be used to solve it.
///
/// [1]: (https://sabrinajewson.org/blog/the-better-alternative-to-lifetime-gats)
pub trait Lending<'lend, __Seal: Sealed = Seal<&'lend Self>> {
    type Lend: 'lend;
}

/// An iterator that yields items bound by the lifetime of each iteration.
///
/// Please user [`lending-iterator`] for a more ergonomic way to implement lending iterators, or create your own if you only need a small subset of the features of iterators.
///
/// This crate is meant to be a blanket implementation of the standard library's `iter` module for types that run into the lending iterator problem.
///
/// [`lending-iterator`]: https://crates.io/crate/lending-iterator
pub trait Lender: for<'all /* where Self: 'all */> Lending<'all> {
    fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend>;
    #[inline]
    fn next_chunk(&mut self, len: usize) -> Chunk<'_, Self>
    where
        Self: Sized,
    {
        Chunk::new(self, len)
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) { (0, None) }
    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.fold(0, |count, _| count + 1)
    }
    #[inline]
    fn last<'call>(mut self) -> Option<<Self as Lending<'call>>::Lend>
    where
        Self: Sized,
    {
        let mut last = None;
        while let Some(x) = self.next() {
            // SAFETY: polonius
            last = Some(unsafe {
                core::mem::transmute::<
                    <Self as Lending<'_>>::Lend,
                    <Self as Lending<'call>>::Lend
                >(x)
            });
        }
        last
    }
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
    #[inline]
    fn nth(&mut self, n: usize) -> Option<<Self as Lending<'_>>::Lend> {
        self.advance_by(n).ok()?;
        self.next()
    }
    #[inline]
    fn step_by(self, step: usize) -> StepBy<Self>
    where
        Self: Sized,
    {
        StepBy::new(self, step)
    }
    #[inline]
    fn chain<U: IntoLender>(self, other: U) -> Chain<Self, <U as IntoLender>::Lender>
    where
        Self: Sized,
        for<'all> U: Lending<'all, Lend = <Self as Lending<'all>>::Lend>,
    {
        Chain::new(self, other.into_lender())
    }
    #[inline]
    fn zip<U: IntoLender>(self, other: U) -> Zip<Self, <U as IntoLender>::Lender>
    where
        Self: Sized,
        for<'all> U: Lending<'all, Lend = <Self as Lending<'all>>::Lend>,
    {
        Zip::new(self, other.into_lender())
    }
    #[inline]
    fn intersperse<'call>(self, separator: <Self as Lending<'call>>::Lend) -> Intersperse<'call, Self>
    where
        Self: Sized,
        for<'all> <Self as Lending<'all>>::Lend: Clone,
    {
        Intersperse::new(self, separator)
    }
    #[inline]
    fn intersperse_with<'call, G>(self, separator: G) -> IntersperseWith<'call, Self, G>
    where
        Self: Sized,
        G: FnMut() -> <Self as Lending<'call>>::Lend,
    {
        IntersperseWith::new(self, separator)
    }
    #[inline]
    fn map<B, F>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
        F: for<'all> HKGFnMut<'all, <Self as Lending<'all>>::Lend, B>,
    {
        Map::new(self, f)
    }
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
    #[inline]
    fn filter<P>(self, predicate: P) -> Filter<Self, P>
    where
        Self: Sized,
        P: FnMut(&<Self as Lending<'_>>::Lend) -> bool,
    {
        Filter::new(self, predicate)
    }
    #[inline]
    fn filter_map<B, F>(self, f: F) -> FilterMap<Self, F>
    where
        Self: Sized,
        F: for<'all> HKGFnMut<'all, <Self as Lending<'all>>::Lend, Option<B>>,
    {
        FilterMap::new(self, f)
    }
    #[inline]
    fn enumerate(self) -> Enumerate<Self>
    where
        Self: Sized,
    {
        Enumerate::new(self)
    }
    #[inline]
    fn peekable<'call>(self) -> Peekable<'call, Self>
    where
        Self: Sized,
    {
        Peekable::new(self)
    }
    #[inline]
    fn skip_while<P>(self, predicate: P) -> SkipWhile<Self, P>
    where
        Self: Sized,
        P: FnMut(&<Self as Lending<'_>>::Lend) -> bool,
    {
        SkipWhile::new(self, predicate)
    }
    #[inline]
    fn take_while<P>(self, predicate: P) -> TakeWhile<Self, P>
    where
        Self: Sized,
        P: FnMut(&<Self as Lending<'_>>::Lend) -> bool,
    {
        TakeWhile::new(self, predicate)
    }
    #[inline]
    fn map_while<B, P>(self, predicate: P) -> MapWhile<Self, P>
    where
        Self: Sized,
        P: for<'all> HKGFnMut<'all, <Self as Lending<'all>>::Lend, Option<B>>,
    {
        MapWhile::new(self, predicate)
    }
    #[inline]
    fn skip(self, n: usize) -> Skip<Self>
    where
        Self: Sized,
    {
        Skip::new(self, n)
    }
    #[inline]
    fn take(self, n: usize) -> Take<Self>
    where
        Self: Sized,
    {
        Take::new(self, n)
    }
    #[inline]
    fn scan<St, B, F>(self, initial_state: St, f: F) -> Scan<Self, St, F>
    where
        Self: Sized,
        F: for<'all> HKGFnMut<'all, (&'all mut St, <Self as Lending<'all>>::Lend), Option<B>>,
    {
        Scan::new(self, initial_state, f)
    }
    #[inline]
    fn flat_map<'call, F>(self, f: F) -> FlatMap<'call, Self, F>
    where
        Self: Sized,
        F: for<'all> HKAFnMut<'all, <Self as Lending<'all>>::Lend>,
        for<'all> <F as HKAFnMut<'all, <Self as Lending<'all>>::Lend>>::B: IntoLender,
    {
        FlatMap::new(self, f)
    }
    #[inline]
    fn flatten<'call>(self) -> Flatten<'call, Self>
    where
        Self: Sized,
        for<'all> <Self as Lending<'all>>::Lend: IntoLender,
    {
        Flatten::new(self)
    }
    #[inline]
    fn fuse(self) -> Fuse<Self>
    where
        Self: Sized,
    {
        Fuse::new(self)
    }
    #[inline]
    fn inspect<F>(self, f: F) -> Inspect<Self, F>
    where
        Self: Sized,
        F: FnMut(&<Self as Lending<'_>>::Lend),
    {
        Inspect::new(self, f)
    }
    // not std::iter
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
    #[inline]
    fn collect<B>(self) -> B
    where
        Self: Sized,
        B: FromLender<Self>,
    {
        FromLender::from_lender(self)
    }
    #[inline]
    fn try_collect<'a, B>(&'a mut self) -> ChangeOutputType<<Self as Lending<'a>>::Lend, B>
    where
        Self: Sized,
        for<'all> <Self as Lending<'all>>::Lend: Try,
        for<'all> <<Self as Lending<'all>>::Lend as Try>::Residual: Residual<B>,
        B: FromLender<TryShunt<'a, &'a mut Self>>,
    {
        let mut residual = None;
        // SAFETY: we ensure that `B` does not have access to `residual`.
        let reborrow = unsafe { &mut *(&mut residual as *mut _) };
        let shunt = TryShunt::<&'a mut Self>::new(self, reborrow);
        let value = FromLender::from_lender(shunt);
        match residual {
            Some(r) => FromResidual::from_residual(r),
            None => Try::from_output(value),
        }
    }
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
    // UNIMPLEMENTABLE: partition_in_place
    #[inline]
    fn is_partitioned<P>(mut self, mut predicate: P) -> bool
    where
        Self: Sized,
        P: FnMut(<Self as Lending<'_>>::Lend) -> bool,
    {
        self.all(&mut predicate) || !self.any(predicate)
    }
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
    #[inline]
    fn find<P>(&mut self, mut predicate: P) -> Option<<Self as Lending<'_>>::Lend>
    where
        Self: Sized,
        P: FnMut(&<Self as Lending<'_>>::Lend) -> bool,
    {
        while let Some(x) = self.next() {
            if predicate(&x) {
                // SAFETY: polonius
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
    #[inline]
    fn find_map<B, F>(&mut self, mut f: F) -> Option<B>
    where
        Self: Sized,
        F: for<'all> HKGFnMut<'all, <Self as Lending<'all>>::Lend, Option<B>>,
    {
        while let Some(x) = self.next() {
            if let Some(y) = f(x) {
                return Some(y);
            }
        }
        None
    }
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
                        // SAFETY: polonius
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
    #[inline]
    fn max<T>(self) -> Option<T>
    where
        Self: Sized,
        T: for<'all> PartialOrd<<Self as Lending<'all>>::Lend>,
        for<'all> <Self as Lending<'all>>::Lend: ToOwned<Owned = T>,
    {
        self.max_by(|x, y| x.partial_cmp(y).unwrap_or(Ordering::Equal))
    }
    #[inline]
    fn min<T>(self) -> Option<T>
    where
        Self: Sized,
        T: for<'all> PartialOrd<<Self as Lending<'all>>::Lend>,
        for<'all> <Self as Lending<'all>>::Lend: ToOwned<Owned = T>,
    {
        self.min_by(|x, y| x.partial_cmp(y).unwrap_or(Ordering::Equal))
    }
    #[inline]
    fn max_by<T, F>(self, mut compare: F) -> Option<T>
    where
        Self: Sized,
        for<'all> <Self as Lending<'all>>::Lend: ToOwned<Owned = T>,
        F: for<'all> FnMut(&T, &<Self as Lending<'all>>::Lend) -> Ordering,
    {
        self.reduce(move |x, y| {
            match compare(&x, &y) {
                Ordering::Less => y.to_owned(),
                _ => x,
            }
        })
    }
    #[inline]
    fn min_by<T, F>(self, mut compare: F) -> Option<T>
    where
        Self: Sized,
        for<'all> <Self as Lending<'all>>::Lend: ToOwned<Owned = T>,
        F: for<'all> FnMut(&T, &<Self as Lending<'all>>::Lend) -> Ordering,
    {
        self.reduce(move |x, y| {
            match compare(&x, &y) {
                Ordering::Greater => y.to_owned(),
                _ => x,
            }
        })
    }
    #[inline]
    fn rev(self) -> Rev<Self>
    where
        Self: Sized + DoubleEndedLender,
    {
        Rev::new(self)
    }
    fn copied<T>(self) -> Copied<Self>
    where
        Self: Sized + for<'all> Lending<'all, Lend = &'all T>,
        T: Copy,
    {
        Copied::new(self)
    }
    fn cloned<T>(self) -> Cloned<Self>
    where
        Self: Sized + for<'all> Lending<'all, Lend = &'all T>,
        T: Clone,
    {
        Cloned::new(self)
    }
    // not std::iter
    #[inline]
    fn owned(self) -> Owned<Self>
    where
        Self: Sized,
        for<'all> <Self as Lending<'all>>::Lend: ToOwned
    {
        Owned::new(self)
    }
    #[inline]
    fn cycle(self) -> Cycle<Self>
    where
        Self: Sized + Clone,
    {
        Cycle::new(self)
    }
    fn cmp<L>(self, other: L) -> Ordering
    where
        L: IntoLender + for<'all> Lending<'all, Lend = <Self as Lending<'all>>::Lend>,
        for <'all> <Self as Lending<'all>>::Lend: Ord,
        Self: Sized,
    {
        self.cmp_by(other, |x, y| x.cmp(&y))
    }
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
    fn partial_cmp<L>(self, other: L) -> Option<Ordering>
    where
        L: IntoLender,
        for<'all> <Self as Lending<'all>>::Lend: PartialOrd<<L as Lending<'all>>::Lend>,
        Self: Sized,
    {
        self.partial_cmp_by(other, |x, y| x.partial_cmp(&y))
    }
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
    fn eq<L>(self, other: L) -> bool
    where
        L: IntoLender,
        for<'all> <Self as Lending<'all>>::Lend: PartialEq<<L as Lending<'all>>::Lend>,
        Self: Sized,
    {
        self.eq_by(other, |x, y| x == y)
    }
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
    fn ne<L>(self, other: L) -> bool
    where
        L: IntoLender,
        for<'all> <Self as Lending<'all>>::Lend: PartialEq<<L as Lending<'all>>::Lend>,
        Self: Sized,
    {
        !self.eq(other)
    }
    fn lt<L>(self, other: L) -> bool
    where
        L: IntoLender,
        for<'all> <Self as Lending<'all>>::Lend: PartialOrd<<L as Lending<'all>>::Lend>,
        Self: Sized,
    {
        self.partial_cmp(other) == Some(Ordering::Less)
    }
    fn le<L>(self, other: L) -> bool
    where
        L: IntoLender,
        for<'all> <Self as Lending<'all>>::Lend: PartialOrd<<L as Lending<'all>>::Lend>,
        Self: Sized,
    {
        matches!(self.partial_cmp(other), Some(Ordering::Less | Ordering::Equal))
    }
    fn gt<L>(self, other: L) -> bool
    where
        L: IntoLender,
        for<'all> <Self as Lending<'all>>::Lend: PartialOrd<<L as Lending<'all>>::Lend>,
        Self: Sized,
    {
        self.partial_cmp(other) == Some(Ordering::Greater)
    }
    fn ge<L>(self, other: L) -> bool
    where
        L: IntoLender,
        for<'all> <Self as Lending<'all>>::Lend: PartialOrd<<L as Lending<'all>>::Lend>,
        Self: Sized,
    {
        matches!(self.partial_cmp(other), Some(Ordering::Greater | Ordering::Equal))
    }
    #[inline]
    fn is_sorted<T>(self) -> bool
    where
        Self: Sized,
        for<'all> <Self as Lending<'all>>::Lend: ToOwned<Owned = T>,
        T: PartialOrd,
    {
        self.is_sorted_by(PartialOrd::partial_cmp)
    }
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
    // not std::iter
    /// Iterators method names already have unique meanings in stdlib, so why not use the ergonomic `iter` method name over `into_iterator`?
    fn iter<'this>(self) -> Iter<'this, Self>
    where
        Self: Sized + 'this,
        for<'all> <Self as Lending<'all>>::Lend: 'static,
    {
        Iter::new(self)
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

impl<'lend, L> Lending<'lend> for &mut L
where
    L: Lender,
{
    type Lend = <L as Lending<'lend>>::Lend;
}
impl<L> Lender for &mut L
where
    L: Lender,
{
    #[inline]
    fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> { (*self).next() }
}
