use crate::{Lender, Lending};
pub struct Chain<A, B> {
    a: A,
    b: B,
    switch: bool,
}
impl<A, B> Chain<A, B> {
    pub(crate) fn new(a: A, b: B) -> Self { Self { a, b, switch: false } }
}
impl<'lend, A, B> Lending<'lend> for Chain<A, B>
where
    for<'all> B: Lending<'all, Lend = <A as Lending<'all>>::Lend>,
    A: Lender,
    B: Lender,
{
    type Lend = <A as Lending<'lend>>::Lend;
}
impl<A, B> Lender for Chain<A, B>
where
    for<'all> B: Lending<'all, Lend = <A as Lending<'all>>::Lend>,
    A: Lender,
    B: Lender,
{
    fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> {
        if !self.switch {
            if let x @ Some(_) = self.a.next() {
                return x;
            }
            self.switch = true;
        }
        self.b.next()
    }
}
