use crate::{
    hkts::{WithLifetime, HKT},
    Lender, Lending,
};
/// not stable
///
/// It is still not decided whether this implementation is correct,
/// because it may force generic implementations and may cause issues with generic specifications.
pub trait FromLender<T>
where
    T: HKT,
{
    fn from_lender<L>(lender: L) -> Self
    where
        L: Lender + for<'lend> Lending<'lend, Lend = <T as WithLifetime<'lend>>::T>;
}
pub trait IntoLender: for<'lend /* where Self: 'lend */> Lending<'lend> {
    type Lender: Lender + for<'lend> Lending<'lend, Lend = <Self as Lending<'lend>>::Lend>;
    fn into_lender(self) -> <Self as IntoLender>::Lender;
}
impl<P: Lender> IntoLender for P {
    type Lender = P;
    #[inline]
    fn into_lender(self) -> P { self }
}
pub trait ExtendLender<A>
where
    A: HKT,
{
    fn extend_lender<T: IntoLender + for<'lend> Lending<'lend, Lend = <A as WithLifetime<'lend>>::T>>(&mut self, lender: T);
    /// Extends a collection with exactly one element.
    fn extend_lender_one(&mut self, item: A) { self.extend_lender(Some(item)); }
    /// Reserves capacity in a collection for the given number of additional elements.
    ///
    /// The default implementation does nothing.
    fn extend_lender_reserve(&mut self, additional: usize) { let _ = additional; }
}
impl ExtendLender<()> for () {
    fn extend_lender<T: IntoLender + for<'lend> Lending<'lend, Lend = ()>>(&mut self, lender: T) {
        lender.into_lender().for_each(drop);
    }
}
impl<A: HKT, B: HKT, ExtendA, ExtendB> ExtendLender<(A, B)> for (ExtendA, ExtendB)
where
    ExtendA: ExtendLender<A>,
    ExtendB: ExtendLender<B>,
{
    fn extend_lender<T: IntoLender + for<'lend> Lending<'lend, Lend = (A, B)>>(&mut self, lender: T) {
        let mut lender = lender.into_lender();
        let (reserve, _) = lender.size_hint();
        if reserve > 0 {
            self.0.extend_lender_reserve(reserve);
            self.1.extend_lender_reserve(reserve);
        }
        while let Some((x, y)) = lender.next() {
            self.0.extend_lender_one(x);
            self.1.extend_lender_one(y);
        }
    }
    fn extend_lender_one(&mut self, item: (A, B)) {
        self.0.extend_lender_one(item.0);
        self.1.extend_lender_one(item.1);
    }
    fn extend_lender_reserve(&mut self, additional: usize) {
        self.0.extend_lender_reserve(additional);
        self.1.extend_lender_reserve(additional);
    }
}
