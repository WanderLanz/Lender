//! Test that covar!() macros reject invariant return types.

use std::cell::Cell;

use lender::prelude::*;

fn main() {
    let data = vec![1, 2, 3];
    let lender = lender::lend_iter::<lend!(&'lend i32), _>(data.iter());

    // This should fail to compile because Cell<&'lend i32> is invariant in 'lend
    let _mapped = lender.map(covar_mut!(for<'lend> |_x: &'lend i32| -> Cell<&'lend i32> {
        Cell::new(&0)
    }));
}
