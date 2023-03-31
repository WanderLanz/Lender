use crate::{Lender, Lending};
pub struct StepBy<T> {
    lender: T,
    step: usize,
    first_take: bool,
}
impl<T> StepBy<T> {
    pub(crate) fn new(lender: T, step: usize) -> Self {
        assert!(step != 0);
        StepBy { lender, step: step - 1, first_take: true }
    }
}
impl<'lend, T> Lending<'lend> for StepBy<T>
where
    T: Lender,
{
    type Lend = <T as Lending<'lend>>::Lend;
}
impl<T> Lender for StepBy<T>
where
    T: Lender,
{
    fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> {
        if self.first_take {
            self.first_take = false;
            self.lender.next()
        } else {
            self.lender.nth(self.step)
        }
    }
}
