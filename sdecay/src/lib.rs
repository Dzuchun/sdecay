#![doc = include_str!("../README.md")]
#![cfg_attr(
    not(test),
    forbid(clippy::undocumented_unsafe_blocks, clippy::missing_safety_doc)
)]
#![cfg_attr(not(test), no_std)]

// -- FOLLOWING MODULES DO CONTAIN UNSAFE CODE --
pub mod wrapper;

pub mod container;

// -- REST OF THE MODULES ARE MARKED WITH `#[forbid(unsafe)]` --

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

pub use paste::paste;

#[forbid(unsafe_code)]
mod macros;
use macros::{generic_list, impl_moveable};
