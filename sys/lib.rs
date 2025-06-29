#![doc = include_str!("README.md")]
#![cfg_attr(not(test), no_std)]

mod ffi {
    #![expect(
        unused,
        missing_docs,
        missing_debug_implementations,
        unsafe_op_in_unsafe_fn,
        unnameable_types,
        unreachable_pub,
        clippy::pub_underscore_fields,
        clippy::unreadable_literal,
        clippy::missing_safety_doc,
        clippy::semicolon_if_nothing_returned
    )]
    include!("bindings.rs");
}

#[doc = include_str!("BUILDING.md")]
pub mod building {}

/// Raw bindings to `SandiaDecay` items
pub mod sandia_decay {
    pub use crate::ffi::root::SandiaDecay::NuclideMixture_HowToOrder as HowToOrder;
    pub use crate::ffi::root::SandiaDecay::*;
}

/// Helper functions used in the safe wrapper exposing tricky C++ semantics
pub mod sdecay {
    pub use crate::ffi::root::sdecay::*;
    pub use crate::ffi::root::std::string;
}

/// Provided databases as included byte blobs
#[cfg(any(
    feature = "embed",
    feature = "embed-min",
    feature = "embed-nocoinc-min"
))]
pub mod database {
    use pathsep::{join_path, path_separator};

    /// "Default" database provided by `SandiaDecay`
    ///
    /// Size: about 30MiB
    #[cfg(feature = "embed")]
    pub const DATABASE: &[u8] = include_bytes!(pathsep::join_path!(
        env!("CARGO_MANIFEST_DIR"),
        "vendor",
        "sandia.decay.xml"
    ));

    /// "Min" database provided by `SandiaDecay`
    ///
    /// Size: about 16MiB
    #[cfg(feature = "embed-min")]
    pub const DATABASE_MIN: &[u8] = include_bytes!(join_path!(
        env!("CARGO_MANIFEST_DIR"),
        "vendor",
        "sandia.decay.min.xml"
    ));

    /// "Nocoinc-min" database provided by `SandiaDecay`
    ///
    /// Size: about 6MiB
    #[cfg(feature = "embed-nocoinc-min")]
    pub const DATABASE_NOCOINC_MIN: &[u8] = include_bytes!(join_path!(
        env!("CARGO_MANIFEST_DIR"),
        "vendor",
        "sandia.decay.nocoinc.min.xml"
    ));
}

#[cfg(test)]
mod tests;
