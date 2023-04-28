use crate::{Lender, Lending};
/// # Example
/// ```rust
/// use lender::*;
/// struct U32Sum(pub u32);
/// impl<'sum> SumLender<&'sum u32> for U32Sum {
///     type Lend<'lend> = &'lend u32;
///     fn sum_lender<L>(lender: L) -> Self
///     where
///         L: Lender + for<'all> Lending<'all, Lend = Self::Lend<'all>>,
///     {
///         U32Sum(lender.fold(0, |acc, x| acc + *x))
///     }
/// }
/// fn f(mut l: impl Lender + for<'all> Lending<'all, Lend = &'all u32>) -> u32 {
///     <U32Sum as SumLender<&u32>>::sum_lender(&mut l).0
/// }
/// ```
pub trait SumLender<A>: Sized {
    type Lend<'lend>;
    fn sum_lender<L>(lender: L) -> Self
    where
        L: Lender + for<'all> Lending<'all, Lend = Self::Lend<'all>>;
}
