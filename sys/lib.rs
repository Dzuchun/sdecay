#![doc = include_str!("README.md")]
#![cfg_attr(not(test), no_std)]

mod ffi {
    #![expect(
        unused,
        unsafe_op_in_unsafe_fn,
        clippy::unreadable_literal,
        clippy::semicolon_if_nothing_returned
    )]
    include!("bindings.rs");
}

#[doc = include_str!("BUILDING.md")]
pub mod building {}
