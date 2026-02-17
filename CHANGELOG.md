# Change Log

## [0.6.0] - 2026-02-17

### New

- Major improvement to the covariance check: it is no longer possible to
  circumvent it using `core::ptr::null()`, `todo!()`, `unimplemented!()`,
  `panic!()`, `loop {}`, etc.

- Convenience `Lender::lender_by_ref` method that is equivalent to
  `iter` followed by `into_ref_lender`.

### Fixed

- `FilterMap`, `Scan`, and `MapWhile` (both standard and fallible) had an
  additional parameter that was forcing a `'static` bound when extending the
  trait. Now they are structured like `Map`.

## [0.5.1] - 2026-02-14

### New

- New `FromIterRef` (fallible) source to create lenders from references to
  elements returned by iterators. Accessible through the
  `into_ref_lender`/`into_fallible_ref_lender` extension methods.

## [0.5.0] - 2026-02-10

### New

- New infrastructure to check covariance of lends. Every implementation must
  use either the (safe) `check_covariance!`/`check_covariance_fallible!` macros
  (for sources) to check covariance of the lend lifetime, or the (unsafe)
  `unsafe_assume_covariance!`/`unsafe_assume_covariance_fallible!` macros when
  assuming covariance of underlying lends (without any check). Methods that used
  to take `for<'all> Lending<'all>` now take `CovariantLending`, which depends
  on `for<'all> Lending<'all, L>` but forces a covariance check.

- Covariance checks have also been embedded in the `covar!`, `covar_mut!` and
  `covar_once` macros (formerly `hrc...`), which are now required to pass a
  closure, as closures are passed through the `Covar` wrapper, which forces
  a covariance check. This unfortunately includes closures without lifetimes.

- Added missing `FromFallibleLender`/`ExtendFallibleLender` traits.

- The `Convert` adapter is now accessible through the `Lender::convert` method.

### Improved

- `windows_mut` is now double-ended.

- Fallible lenders have now feature parity with normal lenders. In particular,
  `FallibleLender` has now `chunk` and `rposition` methods.

- Thanks to `AliasableBox` and `MaybeDangling` we now pass miri.

- `FallibleLender::advance_by`/`FallibleLender::advance_back_by` have now a
  signature aligned with `Lender::advance_by`/`Lender::advance_back_by`.

- Completed implementation of standard traits (`Debug`, `Clone`, `Default`, etc.)
  where possible.

### Changed

- Macros `hrc`, `hrc_mut` and `hrc_once` have been renamed to `covar`, `covar_mut`
  and `covar_once`, respectively.

- Windows and array windows must have a non-zero length, as in `Iterator`,
  and they implement `FusedLender` and `ExactSizeLender`.

- The return type of `Peekable::peek` is now `Option<&'_ Lend<'_, L>>`, which
  fixes a problem of data escape. Analogously for `FalliblePeekable::peek`.

- The `lend!` macro now covers just a set of fixed covariant types. If you need
  to use more complex types, you can use the `covariant_lend!` macro, which
  however requires that you define a type name (with an optional
  visibility specifier), as it cannot be inlined. The same applies to
  `fallible_lend!` and `covariant_fallible_lend!`.

- Coherent use of `must_use` attribute.

- Fallible sources and adapters are now uniformly in separate modules with the
  same name of the standard ones and are renamed in `mod.rs`.

- Fallible `once`, `repeat`, etc. now follow the `fallible_iterator` design,
  with specific methods like `once_err`, `repeat_err`, etc. to generate
  errors.

- `FallibleFusedLender` guarantees `Ok(None)` to repeat, but does not
  have anymore a guarantee of behavior after an error (like it
  happens with `fallible_iterator`).

- `min`/`max` now require `Ord`, like the standard `Iterator` methods.

- `IntoFallible` now uses `Infallible` as fixed error type, like
  `fallible_iterator`.

### Fixed

- Several possible UBs are no longer possible thanks to the new covariance
  checking infrastructure.

- `Peekable` and `FalliblePeekable` are now deallocating their fields
  in the correct order.

- All implementations propagate correctly fused, double-ended and exact-size
  traits.

- `max`/`max_by` return the last instance in case of ties, as in `Iterator`
  (previously they returned the first instance).

- All repeat method return `(usize::MAX, None)` on `size_hint`, as in `Iterator`,
  if they return a value, or `(0, Some(0))` if they return an error.

- The order of parameters (lend/error) in a few methods was inconsistent.

- Removed `Clone` implementation that could lead to UB from `Peekable` and
  `FlattenCompat`.

## [0.4.2] - 2025-11-18

### Fixed

- Fixed flatten when one of the lenders is empty.

## [0.4.1] - 2025-11-15

### New

- Significant API expansion with the addition of fallible lenders,
  which return a result containing an option.

## [0.4.0] - 2025-09-30

### Fixed

- Removed semantically wrong `DoubleEndedLender` implementation for
  `RepeatWith`.

- Fixed possible UB in `Lender::try_collect`.

## [0.3.2] - 2025-05-22

### Fixed

- Fixed use-after-free bug in `Flatten`/`Peekable`.

## [0.3.1] - 2025-02-24

### New

- All structures returned by methods such as `take`, `map`, etc. that
  have additional data besides a wrapped lender have now `into_parts`
  methods.

## [0.3.0] - 2025-02-24

### New

- All structures returned by methods such as `take`, `map`, etc. have
  now `into_inner` methods.
