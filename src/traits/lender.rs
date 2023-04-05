use alloc::borrow::ToOwned;
use core::{cmp::Ordering, num::NonZeroUsize, ops::ControlFlow};

use crate::{
    hkts::{HKGFnMut, WithLifetime, HKT},
    try_trait_v2::{ChangeOutputType, FromResidual, Residual, Try},
    *,
};

/// An iterator that yields items bound by the lifetime of each iteration.
///
/// Please user [`lending-iterator`] for a more ergonomic way to implement lending iterators, or create your own if you only need a small subset of the features of iterators.
///
/// This crate is meant to be a blanket implementation of the standard library's `iter` module for types that run into the lending iterator problem.
///
/// [`lending-iterator`]: https://crates.io/crate/lending-iterator
pub trait Lender: for<'lend /* where Self: 'lend */> Lending<'lend> {
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
        for<'lend> U: Lending<'lend, Lend = <Self as Lending<'lend>>::Lend>,
    {
        Chain::new(self, other.into_lender())
    }
    #[inline]
    fn zip<U: IntoLender>(self, other: U) -> Zip<Self, <U as IntoLender>::Lender>
    where
        Self: Sized,
        for<'lend> U: Lending<'lend, Lend = <Self as Lending<'lend>>::Lend>,
    {
        Zip::new(self, other.into_lender())
    }
    #[inline]
    fn intersperse<'call>(self, separator: <Self as Lending<'call>>::Lend) -> Intersperse<'call, Self>
    where
        Self: Sized,
        for<'lend> <Self as Lending<'lend>>::Lend: Clone,
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
        F: for<'lend> HKGFnMut<'lend, <Self as Lending<'lend>>::Lend, B>,
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
        F: for<'lend> HKGFnMut<'lend, <Self as Lending<'lend>>::Lend, Option<B>>,
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
        P: for<'lend> HKGFnMut<'lend, <Self as Lending<'lend>>::Lend, Option<B>>,
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
        F: for<'lend> HKGFnMut<'lend, (&'lend mut St, <Self as Lending<'lend>>::Lend), Option<B>>,
    {
        Scan::new(self, initial_state, f)
    }

    // HELP: flat_map
    // HELP: flatten

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
    fn collect<T, B>(self) -> B
    where
        Self: Sized + for<'lend> Lending<'lend, Lend = <T as WithLifetime<'lend>>::T>,
        T: HKT,
        B: FromLender<T>,
    {
        <B as FromLender<T>>::from_lender(self)
    }
    // #[inline]
    // fn try_collect<'call, A: HKT, B: 'call>(&'call mut self) -> <<<Self as Lending<'call>>::Lend as Try>::Residual as Residual<B>>::TryType
    // where
    //     Self: Sized,
    //     for<'all> <Self as Lending<'all>>::Lend: Try<Output = A>,
    //     for<'all> <<Self as Lending<'all>>::Lend as Try>::Residual: Residual<B>,
    //     B: FromLender<A>,
    // {
    //     try_process(self, |i| FromLender::<A>::from_lender(i))
    // }
    #[inline]
    fn collect_into<A: HKT, E: ExtendLender<A>>(self, collection: &mut E) -> &mut E
    where
        Self: Sized + for<'lend> Lending<'lend, Lend = <A as WithLifetime<'lend>>::T>,
    {
        collection.extend_lender(self);
        collection
    }
    fn partition<A: HKT, E, F>(mut self, mut f: F) -> (E, E)
    where
        Self: Sized + for<'lend> Lending<'lend, Lend = <A as WithLifetime<'lend>>::T>,
        E: Default + ExtendLender<A>,
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
        for<'lend> <Self as Lending<'lend>>::Lend: ToOwned<Owned = T>,
        F: FnMut(T, <Self as Lending<'_>>::Lend) -> T,
    {
        let first = self.next()?.to_owned();
        Some(self.fold(first, f))
    }
    #[inline]
    fn try_reduce<T, F, R>(mut self, f: F) -> ChangeOutputType<R, Option<T>>
    where
        Self: Sized,
        for<'lend> <Self as Lending<'lend>>::Lend: ToOwned<Owned = T>,
        F: FnMut(T, <Self as Lending<'_>>::Lend) -> R,
        R: Try<Output = T>,
        R::Residual: Residual<Option<T>>,
    {
        let first = match self.next() {
            Some(x) => x.to_owned(),
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
        F: for<'lend> HKGFnMut<'lend, <Self as Lending<'lend>>::Lend, Option<B>>,
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
        for<'lend> R::Residual: Residual<Option<<Self as Lending<'lend>>::Lend>>,
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
        T: for<'lend> PartialOrd<<Self as Lending<'lend>>::Lend>,
        for<'lend> <Self as Lending<'lend>>::Lend: ToOwned<Owned = T>,
    {
        self.max_by(|x, y| x.partial_cmp(y).unwrap_or(Ordering::Equal))
    }
    #[inline]
    fn min<T>(self) -> Option<T>
    where
        Self: Sized,
        T: for<'lend> PartialOrd<<Self as Lending<'lend>>::Lend>,
        for<'lend> <Self as Lending<'lend>>::Lend: ToOwned<Owned = T>,
    {
        self.min_by(|x, y| x.partial_cmp(y).unwrap_or(Ordering::Equal))
    }
    #[inline]
    fn max_by<T, F>(self, mut compare: F) -> Option<T>
    where
        Self: Sized,
        for<'lend> <Self as Lending<'lend>>::Lend: ToOwned<Owned = T>,
        F: for<'lend> FnMut(&T, &<Self as Lending<'lend>>::Lend) -> Ordering,
    {
        self.reduce(move |x, y| {
            match compare(&x, &y) {
                Ordering::Greater => x,
                _ => y.to_owned(),
            }
        })
    }
    #[inline]
    fn min_by<T, F>(self, mut compare: F) -> Option<T>
    where
        Self: Sized,
        for<'lend> <Self as Lending<'lend>>::Lend: ToOwned<Owned = T>,
        F: for<'lend> FnMut(&T, &<Self as Lending<'lend>>::Lend) -> Ordering,
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
    fn unzip<A: HKT, B: HKT, FromA, FromB>(self) -> (FromA, FromB)
    where
        Self: Sized + for<'lend> Lending<'lend, Lend = (<A as WithLifetime<'lend>>::T, <B as WithLifetime<'lend>>::T)>,
        FromA: Default + ExtendLender<A>,
        FromB: Default + ExtendLender<B>,
    {
        let mut unzipped: (FromA, FromB) = Default::default();
        unzipped.extend_lender(self);
        unzipped
    }
    fn copied<T>(self) -> Copied<Self>
    where
        Self: Sized + for<'lend> Lending<'lend, Lend = &'lend T>,
        T: Copy,
    {
        Copied::new(self)
    }
    fn cloned<T>(self) -> Cloned<Self>
    where
        Self: Sized + for<'lend> Lending<'lend, Lend = &'lend T>,
        T: Clone,
    {
        Cloned::new(self)
    }
    // not std::iter
    #[inline]
    fn owned(self) -> Owned<Self>
    where
        Self: Sized,
        for<'lend> <Self as Lending<'lend>>::Lend: ToOwned
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
        L: IntoLender + for<'lend> Lending<'lend, Lend = <Self as Lending<'lend>>::Lend>,
        for <'lend> <Self as Lending<'lend>>::Lend: Ord,
        Self: Sized,
    {
        self.cmp_by(other, |x, y| x.cmp(&y))
    }
    fn cmp_by<L, F>(self, other: L, mut cmp: F) -> Ordering
    where
        Self: Sized,
        L: IntoLender,
        F: for<'lend> FnMut(<Self as Lending<'lend>>::Lend, <L as Lending<'lend>>::Lend) -> Ordering,
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
        for<'lend> <Self as Lending<'lend>>::Lend: PartialOrd<<L as Lending<'lend>>::Lend>,
        Self: Sized,
    {
        self.partial_cmp_by(other, |x, y| x.partial_cmp(&y))
    }
    fn partial_cmp_by<L, F>(self, other: L, mut partial_cmp: F) -> Option<Ordering>
    where
        Self: Sized,
        L: IntoLender,
        F: for<'lend> FnMut(<Self as Lending<'lend>>::Lend, <L as Lending<'lend>>::Lend) -> Option<Ordering>,
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
        for<'lend> <Self as Lending<'lend>>::Lend: PartialEq<<L as Lending<'lend>>::Lend>,
        Self: Sized,
    {
        self.eq_by(other, |x, y| x == y)
    }
    fn eq_by<L, F>(self, other: L, mut eq: F) -> bool
    where
        Self: Sized,
        L: IntoLender,
        F: for<'lend> FnMut(<Self as Lending<'lend>>::Lend, <L as Lending<'lend>>::Lend) -> bool,
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
        for<'lend> <Self as Lending<'lend>>::Lend: PartialEq<<L as Lending<'lend>>::Lend>,
        Self: Sized,
    {
        !self.eq(other)
    }
    fn lt<L>(self, other: L) -> bool
    where
        L: IntoLender,
        for<'lend> <Self as Lending<'lend>>::Lend: PartialOrd<<L as Lending<'lend>>::Lend>,
        Self: Sized,
    {
        self.partial_cmp(other) == Some(Ordering::Less)
    }
    fn le<L>(self, other: L) -> bool
    where
        L: IntoLender,
        for<'lend> <Self as Lending<'lend>>::Lend: PartialOrd<<L as Lending<'lend>>::Lend>,
        Self: Sized,
    {
        matches!(self.partial_cmp(other), Some(Ordering::Less | Ordering::Equal))
    }
    fn gt<L>(self, other: L) -> bool
    where
        L: IntoLender,
        for<'lend> <Self as Lending<'lend>>::Lend: PartialOrd<<L as Lending<'lend>>::Lend>,
        Self: Sized,
    {
        self.partial_cmp(other) == Some(Ordering::Greater)
    }
    fn ge<L>(self, other: L) -> bool
    where
        L: IntoLender,
        for<'lend> <Self as Lending<'lend>>::Lend: PartialOrd<<L as Lending<'lend>>::Lend>,
        Self: Sized,
    {
        matches!(self.partial_cmp(other), Some(Ordering::Greater | Ordering::Equal))
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
            let k = f(x);
            if k < last {
                return false;
            } else {
                last = k;
            }
        }
        true
    }
    // not std::iter
    /// Iterators method names already have unique meanings in stdlib, so why not use the ergonomic `iter` method name over `into_iterator`?
    fn iter<'this>(self) -> Iter<'this, Self>
    where
        Self: Sized + 'this,
        for<'lend> <Self as Lending<'lend>>::Lend: 'static,
    {
        Iter::new(self)
    }
}

#[inline]
pub(crate) fn lender_compare<A, B, F, T>(mut a: A, mut b: B, mut f: F) -> ControlFlow<T, Ordering>
where
    A: Lender,
    B: Lender,
    for<'lend> F: FnMut(<A as Lending<'lend>>::Lend, <B as Lending<'lend>>::Lend) -> ControlFlow<T>,
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
