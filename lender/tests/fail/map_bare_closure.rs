//! Test that map() rejects a bare closure without covar macros.

use lender::prelude::*;

fn main() {
    let data = vec![1, 2, 3];
    let lender = lender::lend_iter::<lend!(&'lend i32), _>(data.iter());

    // This should fail: map() requires Covar<F>, not a bare closure
    let _mapped = lender.map(|x: &i32| *x * 2);
}
