use core::{fmt::Debug, iter::FusedIterator, mem::MaybeUninit};

use crate::{
    as_cpp_string::AsCppString,
    container::Container,
    impl_moveable,
    nuclide_spec::{NuclideSpec, NumSpec},
    wrapper::{CppException, Nuclide, NuclideActivityPair, Wrapper},
};

/// Rust representation of `SandiaDecay`'s nuclide mixture
///
pub struct NuclideMixture<'l>(
    sdecay_sys::sandia_decay::NuclideMixture,
    core::marker::PhantomPinned,
    core::marker::PhantomData<&'l ()>,
);

#[expect(elided_lifetimes_in_paths)]
const _: () = const {
    use core::mem::{align_of, offset_of, size_of};
    assert!(size_of::<sdecay_sys::sandia_decay::NuclideMixture>() == size_of::<NuclideMixture>());
    assert!(align_of::<sdecay_sys::sandia_decay::NuclideMixture>() == align_of::<NuclideMixture>());
    assert!(offset_of!(NuclideMixture, 0) == 0);
};

impl Wrapper for NuclideMixture<'_> {
    type CSide = sdecay_sys::sandia_decay::NuclideMixture;
}

impl_moveable!(mixture, NuclideMixture['l]);

impl Debug for NuclideMixture<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut f = &mut f.debug_struct("NuclideMixture");
        for NuclideActivityPair { nuclide, activity } in self.initial_nuclide_activities() {
            /// This is a helper struct to avoid allocating string for each activity
            struct BqActivity(f64);
            impl Debug for BqActivity {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    use crate::cst::becquerel;
                    write!(f, "{:.2e} Bq", self.0 / becquerel)
                }
            }
            let symbol = nuclide.symbol.as_str();
            f = f.field(symbol.as_ref(), &BqActivity(activity));
        }
        f.finish()
    }
}

impl Drop for NuclideMixture<'_> {
    fn drop(&mut self) {
        let self_ptr = core::ptr::from_mut(&mut self.0);
        // SAFETY:
        // - ffi call to C++ side destructor
        // - `self_ptr` points to live valid instance, since it was just created from the reference
        unsafe { sdecay_sys::sandia_decay::NuclideMixture_NuclideMixture_destructor(self_ptr) };
    }
}

impl<'l> NuclideMixture<'l> {
    #[expect(clippy::new_ret_no_self)]
    pub(crate) fn new<C: Container<Inner = Self>>(allocator: C::Allocator) -> C {
        let init = |ptr: *mut Self| {
            let ptr = ptr.cast::<sdecay_sys::sandia_decay::NuclideMixture>();
            // SAFETY:
            // - ffi call to C++ side constructor
            // - statically validated type representations
            // - `ptr` points to `MaybeUninit` of correct size and alignment
            unsafe { sdecay_sys::sandia_decay::NuclideMixture_NuclideMixture(ptr) };
        };
        // SAFETY: ffi call above calls C++ constructor, initializing the type
        unsafe { C::init_ptr(allocator, init) }
    }

    #[inline]
    fn ptr(&self) -> *const sdecay_sys::sandia_decay::NuclideMixture {
        core::ptr::from_ref(&self.0)
    }

    /// Returns the number of nuclides in the mixture at `t = 0`
    #[inline]
    pub fn num_initial_nuclides(&self) -> usize {
        let self_ptr = self.ptr();
        // SAFETY: ffi call with
        // - statically validated type representations
        // - correct pointer constness (as of bindgen, that is)
        // - `self_ptr` points to live object, since it was just created from a reference
        let count =
            unsafe { sdecay_sys::sandia_decay::NuclideMixture_numInitialNuclides(self_ptr) };
        #[expect(
            clippy::cast_sign_loss,
            reason = "Number of nuclides cannot be negative"
        )]
        {
            count as usize
        }
    }

    /// Gets initial nuclide at `index`
    ///
    /// ### Safety
    /// `index` MUST be a valid nuclide index, otherwise it's a UB (specifically - uncaught C++ exception)
    #[inline]
    pub unsafe fn initial_nuclide_unchecked(&self, index: usize) -> &Nuclide<'l> {
        let self_ptr = self.ptr();
        #[expect(
            clippy::cast_possible_wrap,
            clippy::cast_possible_truncation,
            reason = "It's a UB to specify indexes larger than number of nuclides, represented as `int` on C++ side"
        )]
        let index = index as i32;
        // SAFETY: ffi call with
        // - statically validated type representations
        // - correct pointer constness (as of bindgen, that is)
        // - `self_ptr` points to live object, since it was just created from a reference
        // - `index` is a valid index of solution nuclide (function invariant)
        let nuclide_ptr =
            unsafe { sdecay_sys::sandia_decay::NuclideMixture_initialNuclide(self_ptr, index) };
        // SAFETY: ffi call above
        // - never returns nullptr
        // - returns nuclide references living for time `'l` (same or smaller time used to add them)
        unsafe { Nuclide::from_ptr_unchecked(nuclide_ptr) }
    }

    /// Gets initial nuclide at `index`
    ///
    /// ### Returns
    /// [`Option::None`] indicates invalid nuclide index
    #[inline]
    pub fn initial_nuclide(&self, index: usize) -> Option<&Nuclide<'l>> {
        if index < self.num_initial_nuclides() {
            // SAFETY: index was asserted to be valid by the condition above
            Some(unsafe { self.initial_nuclide_unchecked(index) })
        } else {
            None
        }
    }

    /// Returns an iterator over initial nuclides of the mixture
    #[inline]
    pub fn initial_nuclides(
        &self,
    ) -> impl DoubleEndedIterator<Item = &Nuclide<'l>> + FusedIterator + ExactSizeIterator {
        (0..self.num_initial_nuclides()).map(|i| {
            // SAFETY: index `i` is from range 0..num_initial_nuclides
            unsafe { self.initial_nuclide_unchecked(i) }
        })
    }

    /// Gets activity of initial nuclide at `index`
    ///
    /// ### Safety
    /// `index` MUST be a valid nuclide index, otherwise it's a UB (specifically - uncaught C++ exception)
    pub unsafe fn initial_activity_unchecked(&self, index: usize) -> f64 {
        let self_ptr = self.ptr();
        #[expect(
            clippy::cast_possible_wrap,
            clippy::cast_possible_truncation,
            reason = "It's a UB to specify indexes larger than number of nuclides, represented as `int` on C++ side"
        )]
        let index = index as i32;
        // SAFETY: ffi call with
        // - statically validated type representations
        // - correct pointer constness (as of bindgen, that is)
        // - `self_ptr` points to a live object, since it was just created from a reference
        unsafe { sdecay_sys::sandia_decay::NuclideMixture_initialActivity(self_ptr, index) }
    }

    /// Gets activity of initial nuclide at `index`
    ///
    /// ### Returns
    /// [`Option::None`] indicates invalid nuclide `index`
    pub fn initial_activity(&self, index: usize) -> Option<f64> {
        if index < self.num_initial_nuclides() {
            // SAFETY: index was asserted to be valid by the condition above
            Some(unsafe { self.initial_activity_unchecked(index) })
        } else {
            None
        }
    }

    /// Returns an iterator over [`NuclideActivityPair`] representing initial nuclides of the mixture
    pub fn initial_nuclide_activities(
        &self,
    ) -> impl DoubleEndedIterator<Item = NuclideActivityPair<'_>> + FusedIterator + ExactSizeIterator
    {
        (0..self.num_initial_nuclides()).map(|i| {
            // SAFETY: index `i` is from range 0..num_initial_nuclides
            let nuclide = unsafe { self.initial_nuclide_unchecked(i) };
            // SAFETY: index `i` is from range 0..num_initial_nuclides
            let activity = unsafe { self.initial_activity_unchecked(i) };
            NuclideActivityPair { nuclide, activity }
        })
    }

    /// Retrieves [`Nuclide`] activity from the database
    ///
    /// Note, that function expects [`NuclideSpec`], see it's doc for a list of accepted specifications
    #[inline]
    pub fn nuclide_activity(&self, time: f64, spec: impl NuclideSpec) -> Option<f64> {
        spec.mixture_activity(time, self)
    }

    #[expect(missing_docs)]
    #[inline]
    pub fn nuclide_atoms(&self, time: f64, spec: impl NuclideSpec) -> Option<f64> {
        spec.mixture_num_atoms(time, self)
    }

    pub(crate) fn activity_by_nuclide(&self, time: f64, nuclide: &Nuclide<'_>) -> Option<f64> {
        let mut ok = MaybeUninit::<f64>::uninit();
        let mut exception = MaybeUninit::<CppException>::uninit();
        let exception_ptr = exception
            .as_mut_ptr()
            .cast::<sdecay_sys::sdecay::Exception>();
        // SAFETY: ffi call with
        // - statically validated type representations
        // - correct pointer constness (as of bindgen, that is)
        // - pointed objects are live, since all pointers are created from references
        let tag = unsafe {
            sdecay_sys::sdecay::nuclide_mixture::try_activity_nuclide(
                ok.as_mut_ptr(),
                exception_ptr,
                self.ptr(),
                time,
                nuclide.ptr(),
            )
        };
        if tag {
            // SAFETY: `tag == true`, so `ok` was initialized
            let ok = unsafe { ok.assume_init() };
            Some(ok)
        } else {
            // SAFETY: `tag == false`, so `exception` was initialized
            let _ = unsafe { exception.assume_init() };
            None
        }
    }

    pub(crate) fn atoms_by_nuclide(&self, time: f64, nuclide: &Nuclide<'_>) -> Option<f64> {
        let mut ok = MaybeUninit::<f64>::uninit();
        let mut exception = MaybeUninit::<CppException>::uninit();
        let exception_ptr = exception
            .as_mut_ptr()
            .cast::<sdecay_sys::sdecay::Exception>();
        // SAFETY: ffi call with
        // - statically validated type representations
        // - correct pointer constness (as of bindgen, that is)
        // - pointed objects are live, since pointers are created from references
        let tag = unsafe {
            sdecay_sys::sdecay::nuclide_mixture::try_atoms_nuclide(
                ok.as_mut_ptr(),
                exception_ptr,
                self.ptr(),
                time,
                nuclide.ptr(),
            )
        };
        if tag {
            // SAFETY: `tag == true`, so `ok` was initialized
            let ok = unsafe { ok.assume_init() };
            Some(ok)
        } else {
            // SAFETY: `tag == false`, so `exception` was initialized
            let _ = unsafe { exception.assume_init() };
            None
        }
    }
    pub(crate) fn activity_by_num(&self, time: f64, spec: &NumSpec) -> Option<f64> {
        let mut ok = MaybeUninit::<f64>::uninit();
        let mut exception = MaybeUninit::<CppException>::uninit();
        let exception_ptr = exception
            .as_mut_ptr()
            .cast::<sdecay_sys::sdecay::Exception>();
        // SAFETY: ffi call with
        // - statically validated type representations
        // - correct pointer constness (as of bindgen, that is)
        // - pointed objects are live, since pointers are created from references
        let tag = unsafe {
            sdecay_sys::sdecay::nuclide_mixture::try_activity_num(
                ok.as_mut_ptr(),
                exception_ptr,
                self.ptr(),
                time,
                spec.z,
                spec.mass_number,
                spec.iso.unwrap_or(0),
            )
        };
        if tag {
            // SAFETY: `tag == true`, so `ok` was initialized
            let ok = unsafe { ok.assume_init() };
            Some(ok)
        } else {
            // SAFETY: `tag == false`, so `exception` was initialized
            let _ = unsafe { exception.assume_init() };
            None
        }
    }

    pub(crate) fn atoms_by_num(&self, time: f64, spec: &NumSpec) -> Option<f64> {
        let mut ok = MaybeUninit::<f64>::uninit();
        let mut exception = MaybeUninit::<CppException>::uninit();
        let exception_ptr = exception
            .as_mut_ptr()
            .cast::<sdecay_sys::sdecay::Exception>();
        // SAFETY: ffi call with
        // - statically validated type representations
        // - correct pointer constness (as of bindgen, that is)
        // - pointed objects are live, since pointers are created from references
        let tag = unsafe {
            sdecay_sys::sdecay::nuclide_mixture::try_atoms_num(
                ok.as_mut_ptr(),
                exception_ptr,
                self.ptr(),
                time,
                spec.z,
                spec.mass_number,
                spec.iso.unwrap_or(0),
            )
        };
        if tag {
            // SAFETY: `tag == true`, so `ok` was initialized
            let ok = unsafe { ok.assume_init() };
            Some(ok)
        } else {
            // SAFETY: `tag == false`, so `exception` was initialized
            let _ = unsafe { exception.assume_init() };
            None
        }
    }

    pub(crate) fn activity_by_symbol(&self, time: f64, symbol: impl AsCppString) -> Option<f64> {
        symbol.with_cpp_string(|symbol| {
            let mut ok = MaybeUninit::<f64>::uninit();
            let mut exception = MaybeUninit::<CppException>::uninit();
            let exception_ptr = exception
                .as_mut_ptr()
                .cast::<sdecay_sys::sdecay::Exception>();
            // SAFETY: ffi call with
            // - statically validated type representations
            // - correct pointer constness (as of bindgen, that is)
            // - pointed objects are live, since pointers are created from references
            let tag = unsafe {
                sdecay_sys::sdecay::nuclide_mixture::try_activity_symbol(
                    ok.as_mut_ptr(),
                    exception_ptr,
                    self.ptr(),
                    time,
                    symbol.ptr(),
                )
            };
            if tag {
                // SAFETY: `tag == true`, so `ok` was initialized
                let ok = unsafe { ok.assume_init() };
                Some(ok)
            } else {
                // SAFETY: `tag == false`, so `exception` was initialized
                let _ = unsafe { exception.assume_init() };
                None
            }
        })
    }

    pub(crate) fn atoms_by_symbol(&self, time: f64, symbol: impl AsCppString) -> Option<f64> {
        symbol.with_cpp_string(|symbol| {
            let mut ok = MaybeUninit::<f64>::uninit();
            let mut exception = MaybeUninit::<CppException>::uninit();
            let exception_ptr = exception
                .as_mut_ptr()
                .cast::<sdecay_sys::sdecay::Exception>();
            // SAFETY: ffi call with
            // - statically validated type representations
            // - correct pointer constness (as of bindgen, that is)
            // - pointed objects are live, since pointers are created from references
            let tag = unsafe {
                sdecay_sys::sdecay::nuclide_mixture::try_atoms_symbol(
                    ok.as_mut_ptr(),
                    exception_ptr,
                    self.ptr(),
                    time,
                    symbol.ptr(),
                )
            };
            if tag {
                // SAFETY: `tag == true`, so `ok` was initialized
                let ok = unsafe { ok.assume_init() };
                Some(ok)
            } else {
                // SAFETY: `tag == false`, so `exception` was initialized
                let _ = unsafe { exception.assume_init() };
                None
            }
        })
    }
}
