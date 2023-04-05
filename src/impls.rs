use crate::{IntoLender, Lender, Lending};

pub struct OptionLender<T>(Option<T>);
impl<'lend, T> Lending<'lend> for OptionLender<T> {
    type Lend = T;
}
impl<T> Lender for OptionLender<T> {
    fn next<'next>(&'next mut self) -> Option<<Self as Lending<'next>>::Lend> { self.0.take() }
}
pub struct OptionLenderRef<'a, T>(&'a Option<T>);
impl<'a, 'lend, T> Lending<'lend> for OptionLenderRef<'a, T> {
    type Lend = &'lend T;
}
impl<'a, T> Lender for OptionLenderRef<'a, T> {
    fn next<'next>(&'next mut self) -> Option<<Self as Lending<'next>>::Lend> { self.0.as_ref().take() }
}

impl<'lend, T> Lending<'lend> for Option<T> {
    type Lend = T;
}
impl<T> IntoLender for Option<T> {
    type Lender = OptionLender<T>;
    #[inline]
    fn into_lender(self) -> OptionLender<T> { OptionLender(self) }
}
impl<'lend, T> Lending<'lend> for &Option<T> {
    type Lend = &'lend T;
}
impl<'a, T> IntoLender for &'a Option<T> {
    type Lender = OptionLenderRef<'a, T>;
    #[inline]
    fn into_lender(self) -> OptionLenderRef<'a, T> { OptionLenderRef(self) }
}