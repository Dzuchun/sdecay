//! Defines ways to identify [`Nuclide`] in [`crate::nuclide_mixture::Mixture`] or [`SandiaDecayDataBase`]
//!
//! Unsafe: no

use core::ffi::CStr;

use crate::wrapper::{Nuclide, SandiaDecayDataBase, StdString};

/// Marks type as identifier for [`Nuclide`] in the [`crate::Mixture`] or [`SandiaDecayDataBase`]
pub trait NuclideSpec {
    /// Retrieves described [`Nuclide`] from the database
    fn get_nuclide<'l>(&self, database: &'l SandiaDecayDataBase) -> Option<&'l Nuclide<'l>>;
}

macro_rules! impl_as_cpp_string {
    (<$l:ident> $t:ty) => {
        impl<const $l: usize> NuclideSpec for $t {
            #[inline]
            fn get_nuclide<'l>(
                &self,
                database: &'l SandiaDecayDataBase,
            ) -> Option<&'l Nuclide<'l>> {
                database.nuclide_by_name(*self)
            }
        }
    };
    ($t:ty) => {
        impl_as_cpp_string!(@$t);
        #[cfg(feature = "alloc")]
        impl_as_cpp_string!(@alloc::boxed::Box<$t>);
        #[cfg(feature = "alloc")]
        impl_as_cpp_string!(@alloc::borrow::Cow<'_, $t>);
    };
    (@$t:ty) => {
        impl NuclideSpec for $t {
            #[inline]
            fn get_nuclide<'l>(
                &self,
                database: &'l SandiaDecayDataBase,
            ) -> Option<&'l Nuclide<'l>> {
                database.nuclide_by_name(self)
            }
        }
        impl_as_cpp_string!(@@&$t);
        impl_as_cpp_string!(@@&&$t);
    };
    (@@$t:ty) => {
        impl NuclideSpec for $t {
            #[inline]
            fn get_nuclide<'l>(
                &self,
                database: &'l SandiaDecayDataBase,
            ) -> Option<&'l Nuclide<'l>> {
                database.nuclide_by_name(&**self)
            }
        }
    };
}

impl_as_cpp_string!(@StdString);

impl_as_cpp_string!(CStr);
#[cfg(feature = "alloc")]
impl_as_cpp_string!(@alloc::ffi::CString);

impl_as_cpp_string!(str);
#[cfg(feature = "alloc")]
impl_as_cpp_string!(@alloc::string::String);

impl_as_cpp_string!([u8]);
#[cfg(feature = "alloc")]
impl_as_cpp_string!(@alloc::vec::Vec<u8>);

#[cfg(feature = "std")]
impl_as_cpp_string!(std::ffi::OsStr);
#[cfg(feature = "std")]
impl_as_cpp_string!(@std::ffi::OsString);

#[cfg(feature = "std")]
impl_as_cpp_string!(std::path::Path);
#[cfg(feature = "std")]
impl_as_cpp_string!(@std::path::PathBuf);

impl_as_cpp_string!(<N> [u8; N]);
impl_as_cpp_string!(<N> &[u8; N]);
impl_as_cpp_string!(<N> &&[u8; N]);

impl NuclideSpec for Nuclide<'_> {
    #[inline]
    fn get_nuclide<'l>(&self, database: &'l SandiaDecayDataBase) -> Option<&'l Nuclide<'l>> {
        database.nuclide_by_name(&self.symbol)
    }
}

impl NuclideSpec for &Nuclide<'_> {
    #[inline]
    fn get_nuclide<'l>(&self, database: &'l SandiaDecayDataBase) -> Option<&'l Nuclide<'l>> {
        database.nuclide_by_name(&self.symbol)
    }
}

/// Numeric description of the [`Nuclide`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NumSpec {
    /// Nuclei charge i.e. proton count
    pub z: i32,
    /// Nuclei mass number i.e. nucleon count
    pub mass_number: i32,
    /// Isotope index (seems to be SandiaDecay-specific)
    pub iso: Option<i32>,
}

impl NuclideSpec for NumSpec {
    #[inline]
    fn get_nuclide<'l>(&self, database: &'l SandiaDecayDataBase) -> Option<&'l Nuclide<'l>> {
        database.nuclide_by_num(self.z, self.mass_number, self.iso.unwrap_or(0))
    }
}

/// Simplified constructor for [`NumSpec`], allowing construction via statically checked element symbol
///
/// ### Example
/// ```rust
/// # use sdecay::nuclide;
/// let protium = nuclide!(H-1);
/// let deuterium = nuclide!(H-2);
/// let tritium = nuclide!(H-3);
///
/// let k40 = nuclide!(k-40);
/// let a = 238;
/// let u238 = nuclide!(u-a);
/// let pu239 = nuclide!(pu-239);
/// ```
#[macro_export]
macro_rules! nuclide {
    ($symbol:ident-$a:literal) => {
        $crate::nuclide_spec::NumSpec {
            z: $crate::element_inner!($symbol),
            mass_number: $a,
            iso: None,
        }
    };
    ($symbol:ident-$a:ident) => {
        $crate::nuclide_spec::NumSpec {
            z: $crate::element_inner!($symbol),
            mass_number: $a,
            iso: None,
        }
    };
    ($symbol:ident-$a:literal m) => {
        compile_error!("nspec! macro does not support isomers, sorry");
    };
    ($symbol:ident-$a:ident m) => {
        compile_error!("nspec! macro does not support isomers, sorry");
    };
}
