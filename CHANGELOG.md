# Change Log

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
