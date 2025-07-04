#![doc = include_str!("README.md")]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

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
pub mod database {
    /// Default database provided by `SandiaDecay`
    ///
    /// Size: about 30MiB
    #[cfg(feature = "database")]
    pub const DATABASE: &[u8] = sandia_decay_database::FILE;

    /// `min` database provided by `SandiaDecay`
    ///
    /// Size: about 16MiB
    #[cfg(feature = "database-min")]
    pub const DATABASE_MIN: &[u8] = sandia_decay_database_min::FILE;

    /// `nocoinc-min` database provided by `SandiaDecay`
    ///
    /// Size: about 6MiB
    #[cfg(feature = "database-nocoinc-min")]
    pub const DATABASE_NOCOINC_MIN: &[u8] = sandia_decay_database_nocoinc_min::FILE;
}

#[cfg(test)]
mod tests;
