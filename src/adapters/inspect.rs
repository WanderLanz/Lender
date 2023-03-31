use crate::{Lender, Lending};
pub struct Inspect<L, F> {
    lender: L,
    f: F,
}
impl<L, F> Inspect<L, F> {
    pub(crate) fn new(lender: L, f: F) -> Inspect<L, F> { Inspect { lender, f } }
}
impl<'lend, L, F> Lending<'lend> for Inspect<L, F>
where
    L: Lender,
    F: FnMut(&<L as Lending<'lend>>::Lend),
{
    type Lend = <L as Lending<'lend>>::Lend;
}
impl<L, F> Lender for Inspect<L, F>
where
    L: Lender,
    F: FnMut(&<L as Lending<'_>>::Lend),
{
    #[inline]
    fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> {
        let next = self.lender.next();
        if let Some(ref x) = next {
            (self.f)(x);
        }
        next
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) { self.lender.size_hint() }
}
