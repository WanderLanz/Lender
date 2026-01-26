// This test demonstrates that covariant_lend! correctly rejects invariant types.
// The type `&'lend Cell<Option<&'lend String>>` is invariant in `'lend` because Cell
// is invariant in its type parameter.
//
// Using such a type with lend_iter would lead to undefined behavior (dangling references),
// but covariant_lend! catches this at compile time.

use std::cell::Cell;

use lender::{covariant_lend, LendIter, Lender};

// This should fail to compile because the type is invariant in 'lend
covariant_lend!(InvariantLend = &'lend Cell<Option<&'lend String>>);

fn main() {
    let world: Option<Cell<Option<&String>>> = Some(Cell::new(None));

    {
        let mut lending_iter: LendIter<'_, InvariantLend, _> = lender::lend_iter(world.iter());

        lending_iter.next().unwrap().set(Some(&String::from("world")));
    }

    // This would be UB if it compiled - but covariant_lend! prevents it!
    println!("Hello, {:?}!", world.as_ref().unwrap().get().unwrap());
}
