// Test that peek() correctly ties the lend lifetime to the borrow,
// preventing the lend from escaping its valid scope.

use std::convert::Infallible;

use lender::{FallibleLend, FallibleLender, FallibleLending, Lend, Lender, Lending};

struct StaticLender(String);

impl<'lend> Lending<'lend> for StaticLender {
    type Lend = &'lend str;
}

impl Lender for StaticLender {
    lender::check_covariance!();
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        Some(&self.0)
    }
}

struct StaticFallibleLender(String);

impl<'lend> FallibleLending<'lend> for StaticFallibleLender {
    type Lend = &'lend str;
}

impl FallibleLender for StaticFallibleLender {
    type Error = Infallible;
    lender::check_covariance_fallible!();
    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        Ok(Some(&self.0))
    }
}

fn main() {
    // Test Peekable
    let goodbye: &'static str;
    {
        let mut lending_iter = StaticLender(String::from("Goodbye, world!")).peekable();
        goodbye = lending_iter.peek().unwrap();
    }
    println!("{goodbye:?}");

    // Test FalliblePeekable
    let farewell: &'static str;
    {
        let mut fallible_iter = StaticFallibleLender(String::from("Farewell!")).peekable();
        farewell = fallible_iter.peek().unwrap().unwrap();
    }
    println!("{farewell:?}");
}
