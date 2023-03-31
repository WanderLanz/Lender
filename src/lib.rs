#![doc = include_str!("../README.md")]
#![no_std]
use core::{cmp::Ordering, ops::ControlFlow};

mod adapters;
/// Avoiding reliance on nightly feature `try_trait_v2` to implement `Try`.
pub mod try_trait_v2;
use adapters::*;
use try_trait_v2::*;
pub mod hkts;
use hkts::*;

mod sealed {
    pub trait Sealed {}
    pub struct Seal<T>(T);
    impl<T> Sealed for Seal<T> {}
}
use sealed::*;

/// A trait necessary for implementing `Lender`.
///
/// This is a result of Higher-Ranked Trait Bounds (HRTBs) not having a way to express qualifiers (```for<'any where Self: 'any> Self: Trait```)
/// and effectively making HRTBs only useful when you want to express a trait constraint on ALL lifetimes, including 'static (```for<'all> Self: trait```)
///
/// Although the common example of implementing your own LendingIterator uses a (```type Item<'a> where Self: 'a;```) GAT,
/// that generally only works withing a small subset of the features that a LendingIterator needs to provide to be useful.
///
/// Believe me when I say I've tried almost everything else, yet I still ended up having to basically directly copy [Sabrina Jewson's Blog][1]
/// anyways because I couldn't find a way to make it work without this kind of trait indirection. Although I'm not completely sure if this is the only way to do it.
///
/// Please see [Sabrina Jewson's Blog][1] for more information on the problem and how a trait like this can be used to solve it.
///
/// [1]: (https://sabrinajewson.org/blog/the-better-alternative-to-lifetime-gats)
pub trait Lending<'lend, Lended: Sealed = Seal<&'lend Self>> {
    type Lend: 'lend;
}

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
    fn last(&mut self) -> Option<<Self as Lending<'_>>::Lend>
    where
        Self: Sized,
    {
        let mut last = None;
        while let end @ Some(_) = self.next() {
            // REVIEW: SAFETY: `end` is the last item yielded by the iterator, so it is safe to transmute its lifetime to `'this` until the next iteration, when it will be dropped anyways?
            last = unsafe { core::mem::transmute(end) };
        }
        last
    }
    #[inline]
    fn advance_by(&mut self, n: usize) -> Result<(), usize> {
        for i in 0..n {
            self.next().ok_or(i)?;
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
    fn map<F>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
        F: for<'all> HKFnMut<'all, <Self as Lending<'all>>::Lend>,
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
        P: for<'all> FnMut(&<Self as Lending<'all>>::Lend) -> bool,
    {
        Filter::new(self, predicate)
    }
    #[inline]
    fn filter_map<F>(self, f: F) -> FilterMap<Self, F>
    where
        Self: Sized,
        F: for<'all> HKFnMutOpt<'all, <Self as Lending<'all>>::Lend>,
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
    fn peekable<'a>(self) -> Peekable<'a, Self>
    where
        Self: Sized + 'a,
    {
        Peekable::new(self)
    }
    #[inline]
    fn skip_while<P>(self, predicate: P) -> SkipWhile<Self, P>
    where
        Self: Sized,
        P: for<'all> FnMut(&<Self as Lending<'all>>::Lend) -> bool,
    {
        SkipWhile::new(self, predicate)
    }
    #[inline]
    fn take_while<P>(self, predicate: P) -> TakeWhile<Self, P>
    where
        Self: Sized,
        P: for<'all> FnMut(&<Self as Lending<'all>>::Lend) -> bool,
    {
        TakeWhile::new(self, predicate)
    }
    #[inline]
    fn map_while<P>(self, predicate: P) -> MapWhile<Self, P>
    where
        Self: Sized,
        P: for<'all> HKFnMutOpt<'all, <Self as Lending<'all>>::Lend>,
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
    fn scan<St, F>(self, initial_state: St, f: F) -> Scan<Self, St, F>
    where
        Self: Sized,
        F: for<'all> HKFnMutOpt<'all, (&'all mut St, <Self as Lending<'all>>::Lend)>,
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
        F: for<'all> FnMut(&<Self as Lending<'all>>::Lend),
    {
        Inspect::new(self, f)
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
        Self: Sized + for<'all> Lending<'all, Lend = <B as FromLender>::Lend<'all>>,
        B: FromLender,
    {
        FromLender::from_lender(self)
    }
    // // Alot of work to do here to just reimplement std internals
    // #[inline]
    // fn try_collect<B>(&mut self) -> <<<Self as Lending<'_>>::Lend as Try>::Residual as Residual<B>>::TryType
    // where
    //     Self: Sized,
    //     for<'all> <Self as Lending<'all>>::Lend: Try,
    //     for<'all> <<Self as Lending<'all>>::Lend as Try>::Residual: Residual<B>,
    //     for<'all> B: FromLender<Lend<'all> = <<Self as Lending<'all>>::Lend as Try>::Output>,
    // {
    //     todo!()
    // }

    // need an `Extend` trait {
    //    collect_into
    //    partition
    // }

    // UNIMPLEMENTABLE: partition_in_place
    #[inline]
    fn is_partitioned<P>(mut self, mut predicate: P) -> bool
    where
        Self: Sized,
        P: for<'all> FnMut(<Self as Lending<'all>>::Lend) -> bool,
    {
        self.all(&mut predicate) || !self.any(predicate)
    }
    #[inline]
    fn try_fold<B, F, R>(&mut self, init: B, mut f: F) -> R
    where
        Self: Sized,
        F: for<'all> FnMut(B, <Self as Lending<'all>>::Lend) -> R,
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
        F: for<'all> FnMut(<Self as Lending<'all>>::Lend) -> R,
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
        F: for<'all> FnMut(B, <Self as Lending<'all>>::Lend) -> B,
    {
        let mut accum = init;
        while let Some(x) = self.next() {
            accum = f(accum, x);
        }
        accum
    }
    // UNIMPLEMENTABLE: reduce
    // UNIMPLEMENTABLE: try_reduce
    #[inline]
    fn all<F>(&mut self, mut f: F) -> bool
    where
        Self: Sized,
        F: for<'all> FnMut(<Self as Lending<'all>>::Lend) -> bool,
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
        F: for<'all> FnMut(<Self as Lending<'all>>::Lend) -> bool,
    {
        while let Some(x) = self.next() {
            if f(x) {
                return true;
            }
        }
        false
    }
    #[inline]
    fn find<'find, P>(&'find mut self, mut predicate: P) -> Option<<Self as Lending<'find>>::Lend>
    where
        Self: Sized,
        P: for<'all> FnMut(&<Self as Lending<'all>>::Lend) -> bool,
    {
        while let Some(x) = self.next() {
            if predicate(&x) {
                // SAFETY: #Polonius
                return Some(unsafe { core::mem::transmute::<<Self as Lending<'_>>::Lend, <Self as Lending<'find>>::Lend>(x) });
            }
        }
        None
    }
    #[inline]
    fn find_map<'find, F>(&'find mut self, mut f: F) -> Option<<F as HKFnOnceOpt<'find, <Self as Lending<'find>>::Lend>>::HKOutput>
    where
        Self: Sized,
        F: for<'all> HKFnMutOpt<'all, <Self as Lending<'all>>::Lend>,
    {
        while let Some(x) = self.next() {
            if let Some(y) = f(x) {
                // SAFETY: #Polonius
                return Some(unsafe { core::mem::transmute::<
                    <F as HKFnOnceOpt<'_, <Self as Lending<'_>>::Lend>>::HKOutput,
                    <F as HKFnOnceOpt<'find, <Self as Lending<'find>>::Lend>>::HKOutput
                >(y) });
            }
        }
        None
    }


    // TODO: ... bookmark here


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
    #[inline]
    fn cycle(self) -> Cycle<Self>
    where
        Self: Sized + Clone,
    {
        Cycle::new(self)
    }
}
pub trait DoubleEndedLender: Lender {
    fn next_back(&mut self) -> Option<<Self as Lending<'_>>::Lend>;
}
pub trait FromLender: Sized {
    type Lend<'lend>: 'lend;
    fn from_lender<T>(lender: T) -> Self
    where
        T: Lender + for<'lend> Lending<'lend, Lend = Self::Lend<'lend>>;
}
pub trait IntoLender: for<'all> Lending<'all> {
    type Lender: Lender + for<'all> Lending<'all, Lend = <Self as Lending<'all>>::Lend>;
    fn into_lender(self) -> <Self as IntoLender>::Lender;
}
impl<P: Lender> IntoLender for P {
    type Lender = P;
    #[inline]
    fn into_lender(self) -> P { self }
}

#[allow(dead_code)]
#[inline]
pub(crate) fn lender_compare<A, B, F, T>(mut a: A, mut b: B, mut f: F) -> ControlFlow<T, Ordering>
where
    A: Lender,
    B: Lender,
    for<'all> F: 'all + FnMut(<A as Lending<'all>>::Lend, <B as Lending<'all>>::Lend) -> ControlFlow<T>,
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

#[cfg(test)]
mod test {
    extern crate alloc;
    use alloc::vec::Vec;

    use super::*;

    /// Minimal example of a lender
    struct MyLender<'a, T: 'a>(&'a mut T);
    impl<'lend, 'a, T: 'a> Lending<'lend> for MyLender<'a, T> {
        type Lend = &'lend mut T;
    }
    impl<'a, T: 'a> Lender for MyLender<'a, T> {
        fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> { Some(&mut self.0) }
    }

    impl FromLender for Vec<u8> {
        type Lend<'lend> = &'lend u8;
        fn from_lender<T>(lender: T) -> Self
        where
            T: Lender + for<'lend> Lending<'lend, Lend = Self::Lend<'lend>>,
        {
            let mut vec = Vec::new();
            lender.for_each(|x| {
                vec.push(*x);
            });
            vec
        }
    }

    fn _next<'x>(x: &'x mut u32) {
        let mut bar: MyLender<'x, u32> = MyLender(x);
        let _ = bar.next();
        let _ = bar.next();
        while let Some(x) = bar.next() {
            let _ = x;
        }
    }
    fn _lender_into_lender<'x>(x: &'x mut u32) {
        let mut bar: MyLender<'x, u32> = MyLender(x);
        let _ = bar.into_lender();
    }
}
