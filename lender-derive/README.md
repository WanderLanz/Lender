# Derive procedural macros for the [`lender`](https://crates.io/crates/Lender) crate

This crate provides a [`for_!`] function-like macro that can be used to iterate over
an [`IntoLender`](https://docs.rs/lender/latest/lender/trait.IntoLender.html) with a
syntax similar to a `for` loop:

```ignore
for_!(x in into_lender {
    ...
});
```

The macro expands to a `while let` loop that iterates over a
[`Lender`](https://docs.rs/lender/latest/lender/trait.Lender.html) obtained from the
[`IntoLender`](https://docs.rs/lender/latest/lender/trait.IntoLender.html).
The full `for` syntax is supported (patterns, etc.).
