# Roadmap

WIP, mostly for my own reference.

✅ = Implemented
⚠️ = Partially implemented
❌ = Not implemented

❤️ = Not implemented, and will either require a lot of work or require a significant change in the expected behavior of the API.

💀 = Not implemented, and represents an anti-pattern for lending. May be implemented in the future, but not by me (a.k.a I'm not smart enough to make them work well).

## Are We Iter Yet?

- ⚠️ Scaffold main traits and APIs
  - ⚠️ `Iterator`: [`Lender`](/##Lender)
  - ⚠️ `DoubleEndedIterator`: `DoubleEndedLender`
  - ❌ `ExactSizeIterator`: `ExactSizeLender`
  - ⚠️ `FusedIterator`: `FusedLender`
  - ✅ `IntoIterator`: `IntoLender`
  - ✅ `FromIterator`: `FromLender`
  - ❌ `Extend`: `ExtendLender`
- ❌ Make adapters functional
- ❌ Implement traits for common types (i.e. `IntoLender`, `FromLender`)
- ❌ Attempt from_fn and similar APIs
- ❌ Documentation, if it seems necessary...

## Lender

Methods which require two `Lend`s to be compared from the same `Lender` thus cannot be implemented for `Lender` without some significant shortcomings, and are better off used via `Copy`, `Clone`, or `Owned` turning the `Lender` into a `Iterator`.

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
|❤️`try_collect`       |❌`collect_into`      |❌`partition`         |
|❌`partition_in_place`|✅`is_partitioned`    |✅`try_fold`          |
|✅`try_for_each`      |✅`fold`              |❌`reduce`            |
|❌`try_reduce`        |✅`all`               |✅`any`               |
|✅`find`              |✅`find_map`          |✅`try_find`          |
|✅`position`          |❌`rposition`         |💀`max`               |
|💀`min`               |💀`max_by_key`        |💀`max_by`            |
|💀`min_by_key`        |💀`min_by`            |✅`rev`               |
|❌`unzip`             |✅`copied`            |✅`cloned`            |
|✅`cycle`             |❤️`array_chunks`      |💀`sum`               |
|💀`product`           |❌`cmp`               |❌`cmp_by`            |
|❌`partial_cmp`       |❌`partial_cmp_by`    |❌`eq`                |
|❌`eq_by`             |❌`ne`                |❌`lt`                |
|❌`le`                |❌`gt`                |❌`ge`                |
|💀`is_sorted`         |💀`is_sorted_by`      |❌`is_sorted_by_key`  |
