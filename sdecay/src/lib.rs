#![doc = include_str!("../README.md")]
#![cfg_attr(
    not(test),
    forbid(clippy::undocumented_unsafe_blocks, clippy::missing_safety_doc)
)]
#![cfg_attr(not(test), no_std)]
