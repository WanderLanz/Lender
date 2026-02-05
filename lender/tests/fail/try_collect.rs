use crate::try_trait_v2::ChangeOutputType;
use lender::*;
#[derive(Debug)]
struct PrintOnDrop(String);

impl Drop for PrintOnDrop {
    fn drop(&mut self) {
        eprintln!("Drop {}", self.0)
    }
}

struct Wrapper<L>(L);

impl<L> FromLender<L> for Wrapper<L>
where
    L: IntoLender,
{
    fn from_lender(lender: L) -> Self {
        Self(lender)
    }
}

fn main() {
    let mut count = 0u8;
    let mut lender = lender::from_fn(
        (),
        covar_mut!(for<'all> move |(): &'all mut ()| -> Option<Result<(), PrintOnDrop>> {
            let res = match count {
                0 => Some(Ok(())),
                count @ (2 | 4) => {
                    let err = PrintOnDrop(count.to_string());
                    Some(Err(err))
                }
                _ => None,
            };
            count += 1;
            res
        }),
    );
    let wrapper: ChangeOutputType<Result<(), _>, _> = lender.try_collect::<Wrapper<TryShunt<'_, _>>>();

    let mut try_shunt: TryShunt<_> = wrapper.expect("Expected to extract wrapper").0;
    while let Some(()) = try_shunt.next() {
        ()
    }
    while let Some(()) = try_shunt.next() {
        ()
    }
}
