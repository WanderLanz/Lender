use crate::{Lender, Lending};
#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct Mutate<L, F> {
    lender: L,
    f: F,
}
impl<L, F> Mutate<L, F> {
    pub(crate) fn new(lender: L, f: F) -> Mutate<L, F> { Mutate { lender, f } }
}
impl<'lend, L, F> Lending<'lend> for Mutate<L, F>
where
    L: Lender,
    F: FnMut(&mut <L as Lending<'lend>>::Lend),
{
    type Lend = <L as Lending<'lend>>::Lend;
}
impl<L, F> Lender for Mutate<L, F>
where
    L: Lender,
    F: FnMut(&mut <L as Lending<'_>>::Lend),
{
    #[inline]
    fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> {
        let mut next = self.lender.next();
        if let Some(ref mut x) = next {
            (self.f)(x);
        }
        next
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) { self.lender.size_hint() }
}
