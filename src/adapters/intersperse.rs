use crate::{Lender, Lending, Peekable};
pub struct Intersperse<'this, L>
where
    for<'all> <L as Lending<'all>>::Lend: Clone,
    L: Lender + 'this,
{
    lender: Peekable<'this, L>,
    separator: <L as Lending<'this>>::Lend,
    needs_sep: bool,
}
impl<'this, L> Intersperse<'this, L>
where
    for<'all> <L as Lending<'all>>::Lend: Clone,
    L: Lender + 'this,
{
    pub(crate) fn new(lender: L, separator: <L as Lending<'this>>::Lend) -> Self {
        Self { lender: lender.peekable(), separator, needs_sep: false }
    }
}
impl<'lend, 'this, L> Lending<'lend> for Intersperse<'this, L>
where
    for<'all> <L as Lending<'all>>::Lend: Clone,
    L: Lender + 'this,
{
    type Lend = <L as Lending<'lend>>::Lend;
}
impl<'this, L> Lender for Intersperse<'this, L>
where
    for<'all> <L as Lending<'all>>::Lend: Clone,
    L: Lender + 'this,
{
    fn next<'next>(&'next mut self) -> Option<<Self as Lending<'next>>::Lend> {
        if self.needs_sep && self.lender.peek().is_some() {
            self.needs_sep = false;
            // SAFETY: 'this: 'next
            Some(unsafe {
                core::mem::transmute::<<Self as Lending<'this>>::Lend, <Self as Lending<'next>>::Lend>(
                    self.separator.clone(),
                )
            })
        } else {
            self.needs_sep = true;
            self.lender.next()
        }
    }
}
