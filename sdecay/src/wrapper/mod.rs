//! This module defines Rust representations of `SandiaDecay`'s types
//!
//! Unsafe: **YES**

pub(crate) type BindgenString = sdecay_sys::sdecay::string;

mod stdstring;
pub use stdstring::StdString;

mod exception;
pub use exception::CppException;

pub(crate) trait Wrapper {
    type CSide;
}

impl<'l, T: Wrapper> Wrapper for &'l T {
    type CSide = &'l T::CSide;
}

impl<'l, T: Wrapper> Wrapper for &'l mut T {
    type CSide = &'l mut T::CSide;
}

impl<T: Wrapper> Wrapper for *const T {
    type CSide = *const T::CSide;
}

impl<T: Wrapper> Wrapper for *mut T {
    type CSide = *mut T::CSide;
}

macro_rules! impl_wrapper_shared {
    ($t:ty) => {
        impl Wrapper for $t {
            type CSide = $t;
        }
    };
}

impl_wrapper_shared!(bool);
impl_wrapper_shared!(usize);
impl_wrapper_shared!(f32);
impl_wrapper_shared!(f64);
