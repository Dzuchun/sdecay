use core::{mem::MaybeUninit, pin::Pin};

use crate::{
    as_cpp_string::AsCppString,
    container::{Container, ExclusiveContainer, RefContainer},
    impl_moveable,
    wrapper::{
        CppException, Element, Nuclide, Transition, VecChar, VecElementRef, VecNuclideRef,
        VecTransition,
    },
};

/// Rust representation of `SandiaDecay`'s database
///
/// You should not try to construct this type, and any of the info regarding creation and storing of this type is described in [`crate::database::GenericDatabase`]
///
/// This doc can still be useful, since all of the documented functions are visible on [`crate::database::GenericDatabase`] through [`core::ops::Deref`] implementation
#[repr(C)]
pub struct SandiaDecayDataBase(
    pub(crate) sdecay_sys::sandia_decay::SandiaDecayDataBase,
    core::marker::PhantomPinned,
);

const _: () = const {
    use core::mem::{align_of, offset_of, size_of};
    assert!(
        size_of::<sdecay_sys::sandia_decay::SandiaDecayDataBase>()
            == size_of::<SandiaDecayDataBase>()
    );
    assert!(
        align_of::<sdecay_sys::sandia_decay::SandiaDecayDataBase>()
            == align_of::<SandiaDecayDataBase>()
    );
    assert!(offset_of!(SandiaDecayDataBase, 0) == 0);
};

impl_moveable!(database, SandiaDecayDataBase);

impl Drop for SandiaDecayDataBase {
    #[inline]
    fn drop(&mut self) {
        let self_ptr = core::ptr::from_mut(self).cast();
        // SAFETY: ffi call forwarded to `SandiaDecayDataBase`'s destructor
        unsafe {
            sdecay_sys::sandia_decay::SandiaDecayDataBase_SandiaDecayDataBase_destructor(self_ptr);
        };
    }
}

impl SandiaDecayDataBase {
    #[inline]
    #[expect(clippy::new_ret_no_self)]
    pub(crate) fn new<C: Container<Inner = Self>>(allocator: C::Allocator) -> C {
        let init = |ptr: *mut SandiaDecayDataBase| {
            let ptr = ptr.cast::<sdecay_sys::sandia_decay::SandiaDecayDataBase>();
            // SAFETY: ffi call with
            // - statically validated type representations
            // - correct pointer constness (as of bindgen, that is)
            // - `ptr` points to a memory location valid for writes
            unsafe { sdecay_sys::sandia_decay::SandiaDecayDataBase_SandiaDecayDataBase1(ptr) };
        };
        // SAFETY: ffi call above called C++ constructor, initializing the memory
        unsafe { C::init_ptr(allocator, init) }
    }

    #[inline]
    pub(super) fn ptr(&self) -> *const sdecay_sys::sandia_decay::SandiaDecayDataBase {
        core::ptr::from_ref(&self.0)
    }

    /// ### Safety
    /// Obtained pointer should not be used to move out the object
    #[inline]
    pub(super) unsafe fn ptr_mut(
        self: Pin<&mut Self>,
    ) -> *mut sdecay_sys::sandia_decay::SandiaDecayDataBase {
        // SAFETY:
        // - reference will only be used to create a pointer
        // - pointer will not be used to move out of the value (function invariant)
        let ref_mut = unsafe { Pin::into_inner_unchecked(self) };
        core::ptr::from_mut(&mut ref_mut.0)
    }

    #[inline]
    pub(crate) fn reset(self: Pin<&mut Self>) {
        // SAFETY: pointer will only be used to reset the database on C side
        let self_ptr = unsafe { self.ptr_mut() };
        // SAFETY: ffi call with
        // - statically validated type representations
        // - correct pointer constness (as of bindgen, that is)
        // - `self_ptr` points to live object, since it was just created from reference
        unsafe { sdecay_sys::sandia_decay::SandiaDecayDataBase_reset(self_ptr) };
    }

    pub(crate) fn init_path(
        self: Pin<&mut Self>,
        path: impl AsCppString,
    ) -> Result<(), CppException> {
        path.with_cpp_string(|path| {
            // SAFETY: obtained pointer will be used to initialize the database, which does not move out the value
            let self_ptr = unsafe { self.ptr_mut() };
            let mut ok = MaybeUninit::<sdecay_sys::sdecay::Unit>::uninit();
            let mut exception = MaybeUninit::<sdecay_sys::sdecay::Exception>::uninit();
            let path_ptr = path.ptr();
            // SAFETY: ffi call with
            // - statically validated type representations
            // - correct pointer constness (as of bindgen, that is)
            // - `self_ptr` and `path_ptr` point to live objects, since they were just created from references
            let tag = unsafe {
                sdecay_sys::sdecay::database::try_init_database(
                    ok.as_mut_ptr(),
                    exception.as_mut_ptr(),
                    self_ptr,
                    path_ptr,
                )
            };
            if tag {
                // call succeeded, assume database is init (`ffi::Unit` is trivially dropped)
                Ok(())
            } else {
                // SAFETY: `tag == false` guarantees that exception occurred and written to `exception`
                let exception = unsafe { exception.assume_init() };
                Err(CppException(exception))
            }
        })
    }

    pub(crate) fn init_bytes(
        self: Pin<&mut Self>,
        bytes: impl AsRef<[u8]>,
    ) -> Result<(), CppException> {
        let mut tmp = MaybeUninit::uninit();
        let mut bytes_vec = VecChar::from_bytes_in::<RefContainer<'_, _>>(&mut tmp, bytes);
        // `SandiaDecay` requires data vector to be null-terminated:
        if bytes_vec.as_slice().last().is_none_or(|&b| b != 0) {
            bytes_vec.inner().push(0);
        }
        // SAFETY: obtained pointer is only used for database initialization; this operation does not move object out of it
        let self_ptr = unsafe { self.ptr_mut() };
        // SAFETY: (yes, Ivan, it had come to this) **I HOPE C++ SIDE WON'T DO STUPID THINGS**
        let bytes_ptr = unsafe { bytes_vec.inner().bindgen_ptr_mut() };
        let mut ok = MaybeUninit::<sdecay_sys::sdecay::Unit>::uninit();
        let mut exception = MaybeUninit::<sdecay_sys::sdecay::Exception>::uninit();
        // SAFETY: ffi call with
        // - statically validated type representations
        // - correct pointer constness (as of bindgen, that is)
        // - `self_ptr` and `path_ptr` point to live objects, since they were just created from references
        let tag = unsafe {
            sdecay_sys::sdecay::database::try_init_database_bytes(
                ok.as_mut_ptr(),
                exception.as_mut_ptr(),
                self_ptr,
                bytes_ptr.cast(),
            )
        };
        // bytes vector will be dropped by `RefContainer`
        if tag {
            // call succeeded, assume database is init (`ffi::Unit` is trivially dropped)
            Ok(())
        } else {
            // SAFETY: `tag == false` guarantees that exception occurred and written to `exception`
            let exception = unsafe { exception.assume_init() };
            Err(CppException(exception))
        }
    }

    /// Retrieves all [`Nuclide`]s from the database
    ///
    /// ### Example
    /// ```rust
    ///
    /// # #[cfg(feature = "std")] {
    /// # use sdecay::database::Database;
    /// let database = Database::from_env().unwrap();
    /// println!("All of the nuclides contained in a database:");
    /// for nuclide in database.nuclides() {
    ///     println!("- {}", nuclide.symbol);
    /// }
    /// # }
    /// ```
    pub fn nuclides(&self) -> &[&Nuclide<'_>] {
        let self_ptr = self.ptr();
        // SAFETY: ffi call with
        // - statically validated type representations
        // - correct pointer constness (as of bindgen, that is)
        // - `self_ptr` points to live object, since it was just created from a reference
        let bingen_vec =
            unsafe { sdecay_sys::sandia_decay::SandiaDecayDataBase_nuclides2(self_ptr) };
        // SAFETY: ffi call above returns a pointer to live `std::vector<Nuclide const*>`
        let vec = unsafe { &*VecNuclideRef::from_ptr(bingen_vec) };
        vec.as_slice()
    }

    /// Check if the XML file contained decay x-ray information (e.g., the x-rays that are given off during nuclear decays).
    #[inline]
    pub fn xml_contained_decay_xray_info(&self) -> bool {
        let self_ptr = self.ptr();
        // SAFETY: ffi call with
        // - statically validated type representations
        // - correct pointer constness (as of bindgen, that is)
        // - `self_ptr` points to live object, since it was just created from a reference
        unsafe { sdecay_sys::sandia_decay::SandiaDecayDataBase_xmlContainedDecayXRayInfo(self_ptr) }
    }

    /// Check if the XML file contained elemental x-ray information (e.g., xrays that are caused by flouresence)
    #[inline]
    pub fn xml_contained_elemental_xray_info(&self) -> bool {
        let self_ptr = self.ptr();
        // SAFETY: ffi call with
        // - statically validated type representations
        // - correct pointer constness (as of bindgen, that is)
        // - `self_ptr` points to live object, since it was just created from a reference
        unsafe {
            sdecay_sys::sandia_decay::SandiaDecayDataBase_xmlContainedElementalXRayInfo(self_ptr)
        }
    }

    /// Retrieves all [`Element`]s from the database
    ///
    /// ### Example
    /// ```rust
    /// # #[cfg(feature = "std")] {
    /// # use sdecay::database::Database;
    /// let database = Database::from_env().unwrap();
    /// println!("All of the elements contained in a database:");
    /// for element in database.elements() {
    ///     println!("- {}", element.symbol);
    /// }
    /// # }
    /// ```
    #[inline]
    pub fn elements(&self) -> &[&Element<'_>] {
        let self_ptr = self.ptr();
        // SAFETY: ffi call with
        // - statically validated type representations
        // - correct pointer constness (as of bindgen, that is)
        // - `self_ptr` points to live object, since it was just created from a reference
        let bindgen_vec =
            unsafe { sdecay_sys::sandia_decay::SandiaDecayDataBase_elements(self_ptr) };
        // SAFETY: ffi call above returns a pointer to live `std::vector<Element const*>`
        let vec = unsafe { &*VecElementRef::from_ptr(bindgen_vec) };
        vec.as_slice()
    }

    /// Retrieves all [`Transition`]s from the database
    ///
    /// ### Example
    /// ```rust
    /// # #[cfg(feature = "std")] {
    /// # use sdecay::Database;
    /// let database = Database::from_env().unwrap();
    /// println!("All of the transitions contained in a database:");
    /// for transition in database.transitions() {
    ///     println!("- {} -({})-> {} ({}%)", transition.parent.symbol, transition.mode, transition.child.as_ref().map(|child| child.symbol.as_str()).unwrap_or("(nothing)".into()), transition.branch_ratio);
    /// }
    /// # }
    /// ```
    #[inline]
    pub fn transitions(&self) -> &[Transition<'_>] {
        let self_ptr = self.ptr();
        // SAFETY: ffi call with
        // - statically validated type representations
        // - correct pointer constness (as of bindgen, that is)
        // - `self_ptr` points to live object, since it was just created from a reference
        let bindgen_vec =
            unsafe { sdecay_sys::sandia_decay::SandiaDecayDataBase_transitions(self_ptr) };
        // SAFETY: ffi call above returns a pointer to live `std::vector<Transition const*>`
        let vec = unsafe { &*VecTransition::from_ptr(bindgen_vec) };
        vec.as_slice()
    }
}

impl core::fmt::Debug for SandiaDecayDataBase {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Database(...)")
    }
}
