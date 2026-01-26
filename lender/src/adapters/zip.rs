use crate::{
    DoubleEndedLender, ExactSizeLender, FallibleLend, FallibleLender, FallibleLending, FusedFallibleLender, FusedLender,
    IntoLender, Lend, Lender, Lending,
};

pub fn zip<A, B>(a: A, b: B) -> Zip<A::Lender, B::Lender>
where
    A: IntoLender,
    B: IntoLender,
{
    Zip::new(a.into_lender(), b.into_lender())
}

#[derive(Clone)]
#[must_use = "lenders are lazy and do nothing unless consumed"]
pub struct Zip<A, B> {
    a: A,
    b: B,
}
impl<A, B> Zip<A, B> {
    pub(crate) fn new(a: A, b: B) -> Self {
        Self { a, b }
    }
    pub fn into_inner(self) -> (A, B) {
        (self.a, self.b)
    }
}
impl<'lend, A, B> Lending<'lend> for Zip<A, B>
where
    A: Lender,
    B: Lender,
{
    type Lend = (Lend<'lend, A>, Lend<'lend, B>);
}
impl<A, B> Lender for Zip<A, B>
where
    A: Lender,
    B: Lender,
{
    crate::inherit_covariance!();
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        Some((self.a.next()?, self.b.next()?))
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (a_lower, a_upper) = self.a.size_hint();
        let (b_lower, b_upper) = self.b.size_hint();

        let lower = core::cmp::min(a_lower, b_lower);

        let upper = match (a_upper, b_upper) {
            (Some(x), Some(y)) => Some(core::cmp::min(x, y)),
            (Some(x), None) => Some(x),
            (None, Some(y)) => Some(y),
            (None, None) => None,
        };

        (lower, upper)
    }
}
impl<A, B> DoubleEndedLender for Zip<A, B>
where
    A: DoubleEndedLender + ExactSizeIterator,
    B: DoubleEndedLender + ExactSizeIterator,
{
    #[inline]
    fn next_back(&mut self) -> Option<Lend<'_, Self>> {
        let a_sz = self.a.len();
        let b_sz = self.b.len();
        if a_sz != b_sz {
            // Adjust a, b to equal length
            if a_sz > b_sz {
                for _ in 0..a_sz - b_sz {
                    self.a.next_back();
                }
            } else {
                for _ in 0..b_sz - a_sz {
                    self.b.next_back();
                }
            }
        }
        match (self.a.next_back(), self.b.next_back()) {
            (Some(x), Some(y)) => Some((x, y)),
            (None, None) => None,
            _ => unreachable!(),
        }
    }
}
impl<A, B> ExactSizeLender for Zip<A, B>
where
    A: ExactSizeLender,
    B: ExactSizeLender,
{
}
impl<A, B> FusedLender for Zip<A, B>
where
    A: FusedLender,
    B: FusedLender,
{
}

impl<'lend, A, B> FallibleLending<'lend> for Zip<A, B>
where
    A: FallibleLender,
    B: FallibleLender<Error = A::Error>,
{
    type Lend = (FallibleLend<'lend, A>, FallibleLend<'lend, B>);
}
impl<A, B> FallibleLender for Zip<A, B>
where
    A: FallibleLender,
    B: FallibleLender<Error = A::Error>,
{
    type Error = A::Error;
    crate::inherit_covariance_fallible!();

    #[inline]
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        let Some(value_a) = self.a.next()? else { return Ok(None) };
        let Some(value_b) = self.b.next()? else { return Ok(None) };
        Ok(Some((value_a, value_b)))
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (a_lower, a_upper) = self.a.size_hint();
        let (b_lower, b_upper) = self.b.size_hint();

        let lower = core::cmp::min(a_lower, b_lower);

        let upper = match (a_upper, b_upper) {
            (Some(x), Some(y)) => Some(core::cmp::min(x, y)),
            (Some(x), None) => Some(x),
            (None, Some(y)) => Some(y),
            (None, None) => None,
        };

        (lower, upper)
    }
}
impl<A, B> FusedFallibleLender for Zip<A, B>
where
    A: FusedFallibleLender,
    B: FusedFallibleLender<Error = A::Error>,
{
}
