# Communitas patch of reed-solomon-erasure 6.0.0

Vendored from crates.io `reed-solomon-erasure` 6.0.0 (MIT) because that
release is still the latest published version and depends on `lru ^0.7.8`.

`lru` versions older than 0.18.2 are affected by RUSTSEC-2026-0253
(unsound UAF in `LruCache::pop()`). This tree is otherwise upstream 6.0.0
with two changes:

1. `Cargo.toml`: `lru` `"0.7.8"` → `"0.18.2"`.
2. `src/core.rs`: `LruCache::new` now takes `NonZeroUsize` (required since
   `lru` 0.8). Capacity remains the upstream constant 254.

Wired into the workspace via `[patch.crates-io]` in the root `Cargo.toml`.
Remove this vendor when upstream publishes a release that depends on
`lru >= 0.18.2`.
