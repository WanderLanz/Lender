# Change Log

## [0.5.0]

### New

- New infrastructure to check covariance of lends. Every implementation must
  use either the `check_covariance!`/`check_covariance_fallible!` (for sources)
  or `inherit_covariance!`/`inherit_covariance_fallible!` (for adapters) macro
  to check covariance of the lend lifetime. Covariance checks have been embedded
  in the `hrc!`, `hrc_mut!` and `hrc_once` macros. 

### Changed

- `windows_mut` is now double-ended.

- The return type of `Peekable::peek` is now `Option<&'_ Lend<'_, L>>`, which
  fixes a problem of data escape. Analogously for `FalliblePeekable::peek`.

- The `lend!` macro now covers a set of fixed covariant types. If you need
  to use more complex types, you can use the `covariant_lend!` macro, which
  however requires that you define a type name (it cannot be inlined). The
  same applies to `fallible_lend!` and `covariant_fallible_lend!`.

- Thanks to `AliasableBox` and `MaybeDangling` we now pass miri.

## Fixed

- Several possible UBs are no longer possible thanks to the new covariance
  checking infrastructure.

- `Peekable` and `FalliblePeekable` are now deallocating their fields
  in the correct order.

## [0.4.2] - 2025-11-18

### Fixed

* Fixed flatten when one of the lenders is empty.

## [0.4.1] - 2025-11-15

### New

* Significant API expansion with the addition of fallible lenders,
  which return a result containing an option.

## [0.4.0] - 2025-09-30

### Fixed

* Removed semantically wrong `DoubleEndedLender` implementation for
  `RepeatWith`.

* Fixed possible UB in `Lender::try_collect`.

## [0.3.2] - 2025-05-22

### Fixed

* Fixed use-after-free bug in `Flatten`/`Peekable`.

## [0.3.1] - 2025-02-24

### New

* All structures returned by methods such as `take`, `map`, etc. that
  have additional data besides a wrapped lender have now `into_parts`
  methods.

## [0.3.0] - 2025-02-24

### New

* All structures returned by methods such as `take`, `map`, etc. have
  now `into_inner` methods.
