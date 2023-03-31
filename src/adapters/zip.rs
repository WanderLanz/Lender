use crate::{Lender, Lending};
pub struct Zip<A, B> {
    a: A,
    b: B,
}
impl<A, B> Zip<A, B> {
    pub(crate) fn new(a: A, b: B) -> Self { Self { a, b } }
}
impl<'lend, A, B> Lending<'lend> for Zip<A, B>
where
    for<'all> B: Lending<'all, Lend = <A as Lending<'all>>::Lend>,
    A: Lender,
    B: Lender,
{
    type Lend = (<A as Lending<'lend>>::Lend, <B as Lending<'lend>>::Lend);
}
impl<A, B> Lender for Zip<A, B>
where
    for<'all> B: Lending<'all, Lend = <A as Lending<'all>>::Lend>,
    A: Lender,
    B: Lender,
{
    #[inline]
    fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> { Some((self.a.next()?, self.b.next()?)) }
}
