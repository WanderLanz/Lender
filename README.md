# Lender

This is just my own personal Rust practice, trying to implement a lending iterator.

Check out the [`lending-iterator`] crate, by the great [Daniel Henry-Mantilla][Daniel], for a full implementation of lending iterators (with the 'higher-kinded type' workaround/bug). The [`streaming-iterator`] crate is also a great crate if you only need to iterate over a reference to an item.

Turn a lender into an iterator with `cloned()` where lend is `Clone`, `copied()` where lend is `Copy`, `owned()` where lend is `ToOwned`, or `iter()` where lend is already owned.

Neither `partition_in_place` or `array_chunks` is supported by lending iterators.

## Resources

I couldn't have even hoped to begin trying to use this problem
as practice without relying on help from the following people and resources,
please check them out:

- [Sabrina Jewson's Blog](https://sabrinajewson.org/blog/the-better-alternative-to-lifetime-gats) for their awesome
blog post on why lifetime GATs are not (yet) the solution to this problem, I highly recommend reading it.
- [@vague](https://users.rust-lang.org/u/vague), [Cole Miller](https://users.rust-lang.org/u/cole-miller),
and awesome people on the [Rust Users Forum](https://users.rust-lang.org/) in helping me understand the borrow checker and HRTBs better
and being patient with me and other aspiring rustaceans as we try to learn more about Rust.
- [Daniel Henry-Mantilla][Daniel] by writing [`lending-iterator`] and many other great crates and being a great teacher.
- Finally, it goes without saying, everyone who's contributed to Rust core library for making such a great language and iterator library.

<!-- markdownlint-disable MD026 -->
## Unsafe & Transmutes Beware!!!

Many patterns in lending iterators require polonius-emulating unsafe code, but please, if you see any unsafe code that can be made safe, please let me know! I am still learning Rust and I'm sure I've made many mistakes.

## License

Licensed under either the [MIT](/LICENSE-MIT.txt) or [Apache-2.0](/LICENSE-APACHE.txt) license.

[Daniel]: https://github.com/danielhenrymantilla
[`lending-iterator`]: https://crates.io/crates/lending-iterator
[`streaming-iterator`]: https://crates.io/crates/streaming-iterator
