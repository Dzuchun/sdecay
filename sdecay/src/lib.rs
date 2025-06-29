#![doc = include_str!("../README.md")]
#![cfg_attr(
    not(test),
    forbid(clippy::undocumented_unsafe_blocks, clippy::missing_safety_doc)
)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![cfg_attr(all(not(test), not(feature = "std")), no_std)]

// -- FOLLOWING MODULES DO CONTAIN UNSAFE CODE --
pub mod wrapper;

pub mod container;

// -- REST OF THE MODULES ARE MARKED WITH `#[forbid(unsafe)]` --

#[doc = include_str!("../SAFETY.md")]
#[forbid(unsafe_code)]
pub mod safety {}

#[doc = include_str!("../../sys/BUILDING.md")]
#[forbid(unsafe_code)]
pub mod building {}

#[doc = include_str!("../FEATURES.md")]
#[forbid(unsafe_code)]
pub mod features {}

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

pub use paste::paste;

#[forbid(unsafe_code)]
mod macros;
use macros::{
    containers, ffi_unwrap_or, forward_pin_mut_call, impl_moveable, nolt, vec_wrapper, wrapper,
};

/// Constants defining `Sandia Decay`'s unit system.
///
/// ### Examples
/// 5 seconds:
/// ```rust
/// # use sdecay::cst::second;
/// let _ = 5.0 * second;
/// ```
///
/// 3e7 Bq:
/// ```rust
/// # use sdecay::cst::Bq;
/// let _ = 3.0e7 * Bq;
/// ```
///
/// etc
#[forbid(unsafe_code)]
pub mod cst {
    pub use sdecay_sys::sandia_decay::{
        Bq, Ci, MBq, MeV, becquerel, cm, cm2, cm3, curie, day, eV, hour, keV, m, meter, mm, month,
        second, year,
    };
}

#[forbid(unsafe_code)]
pub mod database;
#[cfg(feature = "alloc")]
pub use database::{Database, SharedDatabase, UninitDatabase, UninitSharedDatabase};
pub use database::{LocalDatabase, UninitLocalDatabase};

#[forbid(unsafe_code)]
pub mod nuclide_mixture;
pub use nuclide_mixture::LocalMixture;
#[cfg(feature = "alloc")]
pub use nuclide_mixture::Mixture;

#[forbid(unsafe_code)]
pub mod add_nuclide_spec;

#[forbid(unsafe_code)]
pub mod element_spec;

#[forbid(unsafe_code)]
pub mod nuclide_spec;

#[forbid(unsafe_code)]
pub mod as_cpp_string;

#[cfg(test)]
mod tests;
