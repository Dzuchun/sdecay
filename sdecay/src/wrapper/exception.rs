use core::{
    ffi::CStr,
    fmt::{Debug, Display},
};

use crate::wrapper::Wrapper;

/// Error type representing C++ exception
#[derive(Error)]
#[repr(C)]
pub struct CppException(pub(crate) sdecay_sys::sdecay::Exception);

impl Wrapper for CppException {
    type CSide = sdecay_sys::sdecay::Exception;
}

impl CppException {
    #[inline]
    fn ptr(&self) -> *const sdecay_sys::sdecay::Exception {
        core::ptr::from_ref(&self.0)
    }

    /// Retrieves exception description provided by C++
    #[inline]
    pub fn what(&self) -> &CStr {
        let self_ptr = self.ptr();
        // SAFETY: ffi call with
        // - statically validated type representations
        // - correct pointer constness (as of bindgen, that is)
        let what_ptr: *const core::ffi::c_char =
            unsafe { sdecay_sys::sdecay::Exception_what(self_ptr) };
        // SAFETY: pointer was returned from C++ side and represents `.what()` message of live C++ exception
        unsafe { CStr::from_ptr(what_ptr) }
    }

    /// Retrieves exception description provided by C++ as Rust [`str`]
    #[cfg(feature = "alloc")]
    #[inline]
    pub fn what_str(&self) -> alloc::borrow::Cow<'_, str> {
        self.what().to_string_lossy()
    }

    /// Retrieves exception description provided by C++ as Rust [`str`]
    ///
    /// ### Panics
    /// If contents are not valid UTF-8
    #[cfg(not(feature = "alloc"))]
    #[inline]
    pub fn what_str(&self) -> &str {
        core::str::from_utf8(self.what().to_bytes()).expect("Should be a valid UTF-8 text")
    }
}

impl Drop for CppException {
    fn drop(&mut self) {
        let rf = &mut self.0;
        let ptr = core::ptr::from_mut(rf);
        // SAFETY:
        // - ffi call for C++ type destructor in Rust type destructor
        // - provided pointer is valid, since it was just created from live reference
        unsafe { sdecay_sys::sdecay::Exception_destruct(ptr) };
    }
}

impl Debug for CppException {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut d = f.debug_struct("Error");
        let message = self.what_str();
        d.field("exception", &message).finish()
    }
}

impl Display for CppException {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.what_str())
    }
}

impl AsRef<CStr> for CppException {
    #[inline]
    fn as_ref(&self) -> &CStr {
        self.what()
    }
}
