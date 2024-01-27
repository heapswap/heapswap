# Heapswap Macros

This exports: 

## sled_zero_copy
A procedural macro that expands:
```rust
#[sled_zero_copy]
```
into:
```rust
#[derive(zerocopy_derive::FromBytes, zerocopy_derive::FromZeroes, zerocopy_derive::AsBytes, zerocopy_derive::Unaligned)]
#[repr(C)]
```
for use with Sled's zero-copy storage capabilities.