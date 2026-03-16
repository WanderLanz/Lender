# Derive procedural macros for the [`lender`](https://crates.io/crates/Lender) crate

[![crates.io](https://img.shields.io/crates/v/lender-derive.svg)](https://crates.io/crates/lender-derive)
[![docs.rs](https://docs.rs/lender-derive/badge.svg)](https://docs.rs/lender-derive)
[![rustc](https://img.shields.io/badge/rustc-1.85+-red.svg)](https://rust-lang.github.io/rfcs/2495-min-rust-version.html)
[![CI](https://github.com/WanderLanz/Lender/actions/workflows/rust.yml/badge.svg)](https://github.com/WanderLanz/Lender/actions)
![license](https://img.shields.io/crates/l/lender-derive)
[![downloads](https://img.shields.io/crates/d/lender-derive)](https://crates.io/crates/lender-derive)

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
