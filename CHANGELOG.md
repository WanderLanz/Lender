# Change Log

## [0.5.0]

### New

- New infrastructure to check covariance of lends. Every implementation must
  use either the `covariant_lend!` (for sources) or `covariant_inherit!` (for
  adapters) macro to check covariance of the lend lifetime.

### Changed

- `windows_mut` is now double-ended.

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
