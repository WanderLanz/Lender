use crate::*;

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

pub trait FusedLender: Lender {}
impl<L: FusedLender> FusedLender for &mut L {}
