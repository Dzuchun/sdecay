use core::{fmt::Debug, iter::FusedIterator, mem::MaybeUninit, pin::Pin};

use crate::{
    add_nuclide_spec::AddNuclideSpec,
    as_cpp_string::AsCppString,
    container::Container,
    impl_moveable,
    nuclide_spec::{NuclideSpec, NumSpec},
    wrapper::{
        CppException, Nuclide, NuclideActivityPair, NuclideNumAtomsPair, NuclideTimeEvolution,
        VecNuclideTimeEvolution, Wrapper,
    },
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

    /// ### Safety
    /// Obtained pointer should not be used to move out the object
    #[inline]
    unsafe fn ptr_mut(self: Pin<&mut Self>) -> *mut sdecay_sys::sandia_decay::NuclideMixture {
        // SAFETY:
        // - reference will only be used to create a pointer
        // - pointer will not be used to move out of the value (function invariant)
        let ref_mut = unsafe { Pin::into_inner_unchecked(self) };
        core::ptr::from_mut(&mut ref_mut.0)
    }

    pub(crate) fn add_nuclide_num_atoms_pair(self: Pin<&mut Self>, pair: &NuclideNumAtomsPair<'_>) {
        // SAFETY: obtained pointer is only used to add a nuclide into the mixture
        let self_ptr = unsafe { self.ptr_mut() };
        let pair_ptr = pair.ptr();
        // SAFETY: ffi call with
        // - statically validated type representations
        // - correct pointer constness (as of bindgen, that is)
        // - pointed objects are all live, since pointers were created from references
        unsafe { sdecay_sys::sandia_decay::NuclideMixture_addNuclide(self_ptr, pair_ptr) };
    }

    pub(crate) fn add_nuclide_activity_pair(self: Pin<&mut Self>, pair: &NuclideActivityPair<'_>) {
        // SAFETY: obtained pointer is only used to add a nuclide into the mixture
        let self_ptr = unsafe { self.ptr_mut() };
        let pair_ptr = pair.ptr();
        // SAFETY: ffi call with
        // - statically validated type representations
        // - correct pointer constness (as of bindgen, that is)
        // - pointed objects are all live, since pointers were created from references
        unsafe { sdecay_sys::sandia_decay::NuclideMixture_addNuclide1(self_ptr, pair_ptr) };
    }

    #[inline]
    pub(crate) fn add_nuclide_by_activity(
        self: Pin<&mut Self>,
        nuclide: &Nuclide<'l>,
        start_activity: f64,
    ) {
        // SAFETY: obtained pointer is only used to add a nuclide into the mixture
        let self_ptr = unsafe { self.ptr_mut() };
        // SAFETY: ffi call with
        // - statically validated type representations
        // - correct pointer constness (as of bindgen, that is)
        // - `self_ptr` points to live object, since it was just created from a reference
        unsafe {
            sdecay_sys::sandia_decay::NuclideMixture_addNuclideByActivity(
                self_ptr,
                nuclide.ptr(),
                start_activity,
            );
        }
    }

    #[inline]
    pub(crate) fn add_nuclide_by_abundance(
        self: Pin<&mut Self>,
        nuclide: &Nuclide<'_>,
        num_init_atoms: f64,
    ) {
        // SAFETY: obtained pointer is only used to add an aged nuclide into the mixture
        let self_ptr = unsafe { self.ptr_mut() };
        // SAFETY: ffi call with
        // - statically validated type representations
        // - correct pointer constness (as of bindgen, that is)
        // - `self_ptr` points to live object, since it was just created from a reference
        unsafe {
            sdecay_sys::sandia_decay::NuclideMixture_addNuclideByAbundance(
                self_ptr,
                nuclide.ptr(),
                num_init_atoms,
            );
        }
    }

    #[inline]
    pub(crate) fn add_aged_nuclide_by_activity(
        self: Pin<&mut Self>,
        nuclide: &Nuclide<'_>,
        activity: f64,
        age_in_seconds: f64,
    ) -> Result<(), CppException> {
        // SAFETY: obtained pointer will be used to initialize the database, which does not move out the value
        let self_ptr = unsafe { self.ptr_mut() };
        let nuclide_ptr = nuclide.ptr();
        let mut ok = MaybeUninit::<sdecay_sys::sdecay::Unit>::uninit();
        let mut exception = MaybeUninit::<CppException>::uninit();
        let exception_ptr = exception
            .as_mut_ptr()
            .cast::<sdecay_sys::sdecay::Exception>();
        // SAFETY: ffi call with
        // - statically validated type representations
        // - correct pointer constness (as of bindgen, that is)
        // - `self_ptr` and `path_ptr` point to live objects, since they were just created from references
        let tag = unsafe {
            sdecay_sys::sdecay::nuclide_mixture::try_addAgedNuclideByActivity(
                ok.as_mut_ptr(),
                exception_ptr,
                self_ptr,
                nuclide_ptr,
                activity,
                age_in_seconds,
            )
        };
        if tag {
            // call succeeded, assume database is init (`ffi::Unit` is trivially dropped)
            Ok(())
        } else {
            // SAFETY: `tag == false` guarantees that exception occurred and written to `exception`
            let exception = unsafe { exception.assume_init() };
            Err(exception)
        }
    }

    #[inline]
    pub(crate) fn add_aged_nuclide_by_num_atoms(
        self: Pin<&mut Self>,
        nuclide: &Nuclide<'l>,
        number_atoms: f64,
        age_in_seconds: f64,
    ) -> Result<(), CppException> {
        // SAFETY: obtained pointer will be used to initialize the database, which does not move out the value
        let self_ptr = unsafe { self.ptr_mut() };
        let nuclide_ptr = nuclide.ptr();
        let mut ok = MaybeUninit::<sdecay_sys::sdecay::Unit>::uninit();
        let mut exception = MaybeUninit::<CppException>::uninit();
        let exception_ptr = exception
            .as_mut_ptr()
            .cast::<sdecay_sys::sdecay::Exception>();
        // SAFETY: ffi call with
        // - statically validated type representations
        // - correct pointer constness (as of bindgen, that is)
        // - `self_ptr` and `path_ptr` point to live objects, since they were just created from references
        let tag = unsafe {
            sdecay_sys::sdecay::nuclide_mixture::try_addAgedNuclideByNumAtoms(
                ok.as_mut_ptr(),
                exception_ptr,
                self_ptr,
                nuclide_ptr,
                number_atoms,
                age_in_seconds,
            )
        };
        if tag {
            // call succeeded, assume database is init (`ffi::Unit` is trivially dropped)
            Ok(())
        } else {
            // SAFETY: `tag == false` guarantees that exception occurred and written to `exception`
            let exception = unsafe { exception.assume_init() };
            Err(exception)
        }
    }

    #[inline]
    pub(crate) fn add_nuclide_in_secular_equilibrium(
        self: Pin<&mut Self>,
        parent: &Nuclide<'_>,
        parent_activity: f64,
    ) -> bool {
        // SAFETY: obtained pointer is only used to add a nuclide into the mixture
        let self_ptr = unsafe { self.ptr_mut() };
        // SAFETY: ffi call with
        // - statically validated type representations
        // - correct pointer constness (as of bindgen, that is)
        // - `self_ptr` points to live object, since it was just created from a reference
        unsafe {
            sdecay_sys::sandia_decay::NuclideMixture_addNuclideInSecularEquilibrium(
                self_ptr,
                parent.ptr(),
                parent_activity,
            )
        }
    }

    #[inline]
    pub(crate) fn add_nuclide_in_prompt_equilibrium(
        self: Pin<&mut Self>,
        parent: &Nuclide<'_>,
        parent_activity: f64,
    ) {
        // SAFETY: obtained pointer is only used to add a nuclide into the mixture
        let self_ptr = unsafe { self.ptr_mut() };
        // SAFETY: ffi call with
        // - statically validated type representations
        // - correct pointer constness (as of bindgen, that is)
        // - `self_ptr` points to live object, since it was just created from a reference
        unsafe {
            sdecay_sys::sandia_decay::NuclideMixture_addNuclideInPromptEquilibrium(
                self_ptr,
                parent.ptr(),
                parent_activity,
            );
        };
    }

    /// Exposes solutions for number of atoms for each nuclide in the mixture
    pub fn decayed_to_nuclides_evolutions(&self) -> &[NuclideTimeEvolution<'l>] {
        let self_ptr = self.ptr();
        // SAFETY: ffi call with
        // - statically validated type representations
        // - correct pointer constness (as of bindgen, that is)
        // - `self_ptr` points to live object, since it was just created from a reference
        let bindgen_vec = unsafe {
            sdecay_sys::sandia_decay::NuclideMixture_decayedToNuclidesEvolutions(self_ptr)
        };
        // SAFETY: ffi call above returns a pointer to live `std::vector<NuclideTimeEvolution>`
        let vec = unsafe { &*VecNuclideTimeEvolution::from_ptr(bindgen_vec) };
        vec.as_slice()
    }

    /// Returns the number of nuclides given in the solution
    ///
    /// Corresponds to the index for nulides in the vector returned by [`decayed_to_nuclides_evolutions`](NuclideMixture::decayed_to_nuclides_evolutions)
    #[inline]
    pub fn num_solution_nuclides(&self) -> usize {
        let self_ptr = self.ptr();
        // SAFETY: ffi call with
        // - statically validated type representations
        // - correct pointer constness (as of bindgen, that is)
        // - `self_ptr` points to live object, since it was just created from a reference
        let count: core::ffi::c_int =
            unsafe { sdecay_sys::sandia_decay::NuclideMixture_numSolutionNuclides(self_ptr) };
        #[expect(
            clippy::cast_sign_loss,
            reason = "Number of nuclides cannot be negative"
        )]
        {
            count as usize
        }
    }

    /// Gets solution nuclide at `index`
    ///
    /// ### Safety
    /// `index` MUST be a valid nuclide index, otherwise it's a UB (specifically - uncaught C++ exception)
    #[inline]
    pub unsafe fn solution_nuclide_unchecked(&self, index: usize) -> &Nuclide<'l> {
        let self_ptr = self.ptr();
        #[expect(
            clippy::cast_possible_wrap,
            clippy::cast_possible_truncation,
            reason = "It's a UB to specify indexes larger than number of nuclides, represented as `int` on C++ side"
        )]
        let index = index as core::ffi::c_int;
        // SAFETY: ffi call with
        // - statically validated type representations
        // - correct pointer constness (as of bindgen, that is)
        // - `self_ptr` points to live object, since it was just created from a reference
        // - `index` is a valid index of solution nuclide (function invariant)
        let nuclide_ptr =
            unsafe { sdecay_sys::sandia_decay::NuclideMixture_solutionNuclide(self_ptr, index) };
        // SAFETY: ffi call above
        // - never returns nullptr
        // - returns nuclide references living for time `'l` (same or smaller time used to add them)
        unsafe { Nuclide::from_ptr_unchecked(nuclide_ptr) }
    }

    /// Gets solution nuclide at `index`
    ///
    /// ### Returns
    /// [`Option::None`] indicates invalid nuclide index
    #[inline]
    pub fn solution_nuclide(&self, index: usize) -> Option<&Nuclide<'l>> {
        if index < self.num_solution_nuclides() {
            // SAFETY: index was asserted to be valid by the condition above
            Some(unsafe { self.solution_nuclide_unchecked(index) })
        } else {
            None
        }
    }

    /// Returns an iterator over nuclides in [`decayed_to_nuclides_evolutions`](NuclideMixture::decayed_to_nuclides_evolutions)
    #[inline]
    pub fn solution_nuclides(
        &self,
    ) -> impl DoubleEndedIterator<Item = &Nuclide<'l>> + FusedIterator + ExactSizeIterator {
        (0..self.num_solution_nuclides()).map(|i| {
            // SAFETY: index `i` is from range 0..num_solution_nuclides
            unsafe { self.solution_nuclide_unchecked(i) }
        })
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

    /// Gets atom count of initial nuclide at `index`
    ///
    /// ### Safety
    /// `index` MUST be a valid nuclide index, otherwise it's a UB (specifically - uncaught C++ exception)
    pub unsafe fn initial_num_atoms_unchecked(&self, index: usize) -> f64 {
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
        unsafe { sdecay_sys::sandia_decay::NuclideMixture_numInitialAtoms(self_ptr, index) }
    }

    /// Gets atom count of initial nuclide at `index`
    ///
    /// ### Returns
    /// [`Option::None`] indicates invalid nuclide `index`
    pub fn initial_num_atoms(&self, index: usize) -> Option<f64> {
        if index < self.num_initial_nuclides() {
            // SAFETY: index was asserted to be valid by the condition above
            Some(unsafe { self.initial_num_atoms_unchecked(index) })
        } else {
            None
        }
    }

    /// Returns an iterator over [`NuclideNumAtomsPair`] representing initial nuclides of the mixture
    pub fn initial_nuclide_num_atoms(
        &self,
    ) -> impl DoubleEndedIterator<Item = NuclideNumAtomsPair<'_>> + FusedIterator + ExactSizeIterator
    {
        (0..self.num_initial_nuclides()).map(|i| {
            // SAFETY: index `i` is from range 0..num_initial_nuclides
            let nuclide = unsafe { self.initial_nuclide_unchecked(i) };
            // SAFETY: index `i` is from range 0..num_initial_nuclides
            let num_atoms = unsafe { self.initial_num_atoms_unchecked(i) };
            NuclideNumAtomsPair { nuclide, num_atoms }
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

    #[inline]
    pub(crate) fn add_nuclide(self: Pin<&mut Self>, spec: impl AddNuclideSpec) {
        spec.add_nuclide(self);
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

    #[expect(missing_docs)]
    #[inline]
    pub fn total_activity(&self, time: f64) -> f64 {
        let self_ptr = self.ptr();
        // SAFETY: ffi call with
        // - statically validated type representations
        // - correct pointer constness (as of bindgen, that is)
        // - `self_ptr` points to a live object, since it was just created from a reference
        unsafe { sdecay_sys::sandia_decay::NuclideMixture_totalActivity(self_ptr, time) }
    }

    #[expect(missing_docs)]
    #[inline]
    pub fn total_mass_in_grams(&self, time: f64) -> f64 {
        let self_ptr = self.ptr();
        // SAFETY: ffi call with
        // - statically validated type representations
        // - correct pointer constness (as of bindgen, that is)
        // - `self_ptr` points to a live object, since it was just created from a reference
        unsafe { sdecay_sys::sandia_decay::NuclideMixture_totalMassInGrams(self_ptr, time) }
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

    pub(crate) fn clear(self: Pin<&mut Self>) {
        // SAFETY: obtained pointer is only used to clear the mixture
        let self_ptr = unsafe { self.ptr_mut() };
        // SAFETY: ffi call with
        // - statically validated type representations
        // - correct pointer constness (as of bindgen, that is)
        // - pointed objects are live, since pointers are created from references
        unsafe { sdecay_sys::sandia_decay::NuclideMixture_clear(self_ptr) };
    }
}
