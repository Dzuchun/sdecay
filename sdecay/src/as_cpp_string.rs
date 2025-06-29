//! Defines types that can produce temporary `&`[`StdString`]
//!
//! Unsafe: no

use core::{ffi::CStr, mem::MaybeUninit};

use crate::wrapper::StdString;

/// Defines type's ability to produce temporary `&`[`StdString`]
///
/// This ensures the most efficient interface, since existing [`StdString`] can just provide reference to itself, while `&`[`CStr`], `&`[`str`], [`std::string::String`], etc. can allocate temporary [`StdString`] on stack via [`crate::container::RefContainer`]
pub trait AsCppString {
    /// Consumes the value, providing reference to temporary [`StdString`]
    fn with_cpp_string<O>(&self, op: impl FnOnce(&StdString) -> O) -> O;
}

macro_rules! impl_as_deref {
    ($t:ty) => {
        impl_as_deref!(@@&$t);
        impl_as_deref!(@@&&$t);
        #[cfg(feature = "alloc")]
        impl_as_deref!(@alloc::boxed::Box<$t>);
        #[cfg(feature = "alloc")]
        impl_as_deref!(@alloc::borrow::Cow<'_, $t>);
    };
    (@$t:ty) => {
        impl_as_deref!(@@$t);
        impl_as_deref!(@@&$t);
        impl_as_deref!(@@&&$t);
    };
    (@@$t:ty) => {
        impl AsCppString for $t {
            #[inline]
            fn with_cpp_string<O>(&self, op: impl FnOnce(&StdString) -> O) -> O {
                <$t as core::ops::Deref>::Target::with_cpp_string(self, op)
            }
        }
    };
}

impl AsCppString for StdString {
    #[inline]
    fn with_cpp_string<O>(&self, op: impl FnOnce(&StdString) -> O) -> O {
        op(self)
    }
}

impl_as_deref!(@@&StdString);
impl_as_deref!(@@&&StdString);

impl AsCppString for CStr {
    #[inline]
    fn with_cpp_string<O>(&self, op: impl FnOnce(&StdString) -> O) -> O {
        let mut tmp = MaybeUninit::uninit();
        let string_ref = &*StdString::from_cstr_local(&mut tmp, self);
        op(string_ref)
    }
}

impl_as_deref!(CStr);
#[cfg(feature = "alloc")]
impl_as_deref!(@alloc::ffi::CString);

impl AsCppString for str {
    #[inline]
    fn with_cpp_string<O>(&self, op: impl FnOnce(&StdString) -> O) -> O {
        let bytes = self.as_bytes();
        bytes.with_cpp_string(op)
    }
}

impl_as_deref!(str);
#[cfg(feature = "alloc")]
impl_as_deref!(@alloc::string::String);

impl AsCppString for [u8] {
    #[inline]
    fn with_cpp_string<O>(&self, op: impl FnOnce(&StdString) -> O) -> O {
        let mut tmp = MaybeUninit::uninit();
        let string_ref = &*StdString::from_bytes_local(&mut tmp, self);
        op(string_ref)
    }
}

impl_as_deref!([u8]);
#[cfg(feature = "alloc")]
impl_as_deref!(@alloc::vec::Vec<u8>);

#[cfg(feature = "std")]
impl AsCppString for std::ffi::OsStr {
    #[inline]
    fn with_cpp_string<O>(&self, op: impl FnOnce(&StdString) -> O) -> O {
        use std::os::unix::ffi::OsStrExt;
        let bytes = self.as_bytes();
        bytes.with_cpp_string(op)
    }
}

#[cfg(feature = "std")]
impl_as_deref!(std::ffi::OsStr);
#[cfg(feature = "std")]
impl_as_deref!(@std::ffi::OsString);

#[cfg(feature = "std")]
impl AsCppString for std::path::Path {
    #[inline]
    fn with_cpp_string<O>(&self, op: impl FnOnce(&StdString) -> O) -> O {
        use std::os::unix::ffi::OsStrExt;
        let bytes = self.as_os_str().as_bytes();
        bytes.with_cpp_string(op)
    }
}

#[cfg(feature = "std")]
impl_as_deref!(std::path::Path);
#[cfg(feature = "std")]
impl_as_deref!(@std::path::PathBuf);

impl<const N: usize> AsCppString for [u8; N] {
    #[inline]
    fn with_cpp_string<O>(&self, op: impl FnOnce(&StdString) -> O) -> O {
        self.as_slice().with_cpp_string(op)
    }
}

impl<const N: usize> AsCppString for &[u8; N] {
    #[inline]
    fn with_cpp_string<O>(&self, op: impl FnOnce(&StdString) -> O) -> O {
        self.as_slice().with_cpp_string(op)
    }
}

impl<const N: usize> AsCppString for &&[u8; N] {
    #[inline]
    fn with_cpp_string<O>(&self, op: impl FnOnce(&StdString) -> O) -> O {
        self.as_slice().with_cpp_string(op)
    }
}
