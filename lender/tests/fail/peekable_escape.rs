// Test that peek() correctly ties the lend lifetime to the borrow,
// preventing the lend from escaping its valid scope.

use lender::{Lend, Lender, Lending};

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

fn main() {
    let goodbye: &'static str;
    {
        let mut lending_iter = StaticLender(String::from("Goodbye, world!")).peekable();
        goodbye = lending_iter.peek().unwrap();
    }

    println!("{goodbye:?}");
}
