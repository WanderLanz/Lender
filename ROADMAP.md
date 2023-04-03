# Roadmap

WIP, mostly for my own reference.

✅ = Implemented
⚠️ = Implemented, but may not be stable or fully functional
❌ = Not implemented

❤️ = Requires a lot of work.

💀 = Represents at least **somewhat** of an anti-pattern for lending iterators.

## Are We Iter Yet?

- ✅ Scaffold main traits and APIs
  - ✅ `Iterator`: [`Lender`](/##Lender)
  - ✅ `DoubleEndedIterator`: `DoubleEndedLender`
  - ✅ `ExactSizeIterator`: `ExactSizeLender`
  - ✅ `FusedIterator`: `FusedLender`
  - ✅ `IntoIterator`: `IntoLender`
  - ⚠️ `FromIterator`: `FromLender`
  - ⚠️ `Extend`: `ExtendLender`
- ❌ Make adapters functional
- ❌ Unit tests to see if it is usable.
- ❌ Implement traits for common types (i.e. `IntoLender`, `FromLender`)
- ❌ Attempt from_fn and similar APIs
- ❌ Documentation...

## Lender

Methods which require two `Lend`s to be compared from the same `Lender` cannot be implemented for `Lender` without some significant shortcomings.

These methods are better off used via `copied`, `cloned`, or `owned` directly turning the `Lender` into a `Iterator`.

You may also use `into_iterator` if the `Lender` already lends owned data.

### (Lender) Are We Iter Yet?

|Method|Method|Method|
|---   |---   |---   |
|✅`next`              |✅`next_chunk`        |⚠️`size_hint`         |
|✅`count`             |✅`last`              |⚠️`advance_by`        |
|✅`nth`               |✅`step_by`           |✅`chain`             |
|✅`zip`               |✅`intersperse`       |✅`intersperse_with`  |
|✅`map`               |✅`for_each`          |✅`filter`            |
|✅`filter_map`        |✅`enumerate`         |✅`peekable`          |
|✅`skip_while`        |✅`take_while`        |✅`map_while`         |
|✅`skip`              |✅`take`              |✅`scan`              |
|❤️`flat_map`          |❤️`flatten`           |✅`fuse`              |
|✅`inspect`           |✅`by_ref`            |✅`collect`           |
|❤️`try_collect`       |✅`collect_into`      |✅`partition`         |
|💀`partition_in_place`|✅`is_partitioned`    |✅`try_fold`          |
|✅`try_for_each`      |✅`fold`              |💀`reduce`            |
|💀`try_reduce`        |✅`all`               |✅`any`               |
|✅`find`              |✅`find_map`          |✅`try_find`          |
|✅`position`          |✅`rposition`         |💀`max`               |
|💀`min`               |💀`max_by_key`        |💀`max_by`            |
|💀`min_by_key`        |💀`min_by`            |✅`rev`               |
|✅`unzip`             |✅`copied`            |✅`cloned`            |
|✅`cycle`             |💀`array_chunks`      |💀`sum`               |
|💀`product`           |✅`cmp`               |✅`cmp_by`            |
|✅`partial_cmp`       |✅`partial_cmp_by`    |✅`eq`                |
|✅`eq_by`             |✅`ne`                |✅`lt`                |
|✅`le`                |✅`gt`                |✅`ge`                |
|💀`is_sorted`         |💀`is_sorted_by`      |✅`is_sorted_by_key`  |
