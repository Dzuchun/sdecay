use core::{ffi::c_int, mem::MaybeUninit, pin::Pin};

use crate::{
    as_cpp_string::AsCppString,
    container::{Container, ExclusiveContainer, RefContainer},
    element_spec::ElementSpec,
    impl_moveable,
    nuclide_spec::NuclideSpec,
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

    pub(crate) fn element_by_atomic_number(&self, atomic_number: c_int) -> Option<&Element<'_>> {
        let self_ptr = self.ptr();
        // SAFETY: ffi call with
        // - statically validated type representations
        // - correct pointer constness (as of bindgen, that is)
        // - pointed objects are all live, since pointers were created from references
        let ptr = unsafe {
            sdecay_sys::sandia_decay::SandiaDecayDataBase_element(self_ptr, atomic_number)
        };
        // SAFETY: SandiaDecay always returns a pointer to a live `Element` or a null pointer
        unsafe { Element::from_ptr(ptr) }
    }

    pub(crate) fn element_by_label(&self, label: impl AsCppString) -> Option<&Element<'_>> {
        label.with_cpp_string(|label| {
            let self_ptr = self.ptr();
            let label_ptr = label.ptr();
            // SAFETY: ffi call with
            // - statically validated type representations
            // - correct pointer constness (as of bindgen, that is)
            // - pointed objects are all live, since pointers were created from references
            let ptr = unsafe {
                sdecay_sys::sandia_decay::SandiaDecayDataBase_element1(self_ptr, label_ptr)
            };
            // SAFETY: SandiaDecay always returns a pointer to a live `Element` or a null pointer
            unsafe { Element::from_ptr(ptr) }
        })
    }

    pub(crate) fn nuclide_by_name(&self, name: impl AsCppString) -> Option<&Nuclide<'_>> {
        name.with_cpp_string(|name| {
            let self_ptr = self.ptr();
            let name_ptr = name.ptr();
            // SAFETY: ffi call with
            // - statically validated type representations
            // - correct pointer constness (as of bindgen, that is)
            // - pointed objects are live, since pointers were created from references
            let ptr = unsafe {
                sdecay_sys::sandia_decay::SandiaDecayDataBase_nuclide(self_ptr, name_ptr)
            };
            // SAFETY: SandiaDecay returns a pointer to a valid `Nuclide` or a null pointer
            unsafe { Nuclide::from_ptr(ptr) }
        })
    }

    pub(crate) fn nuclide_by_num(
        &self,
        z: i32,
        mass_number: i32,
        iso: i32,
    ) -> Option<&Nuclide<'_>> {
        // SAFETY: ffi call with
        // - statically validated type representations
        // - correct pointer constness (as of bindgen, that is)
        // - pointed objects are live, since pointers are created from references
        let ptr = unsafe { self.0.nuclide1(z, mass_number, iso) };
        // SAFETY: SandiaDecay returns a pointer to a valid `Nuclide` or a null pointer
        unsafe { Nuclide::from_ptr(ptr) }
    }
}

impl core::fmt::Debug for SandiaDecayDataBase {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Database(...)")
    }
}

impl SandiaDecayDataBase {
    /// Retrieves [`Nuclide`] from the database, if present
    ///
    /// Note, that [`Nuclide`] is described as [`NuclideSpec`], see it's doc to find the best description for you
    ///
    /// ### Example
    /// ```rust
    /// # #[cfg(feature = "std")] {
    /// # use sdecay::database::Database;
    /// let database = Database::from_env().unwrap();
    /// # use sdecay::nuclide;
    /// // using `NumSpec` (created through `nuclide` macro)
    /// let tritium = database.try_nuclide(nuclide!(H-3)).unwrap();
    /// // (nucleus mass can be stored in a variable)
    /// let famous_mass = 56;
    /// let fe_56 = database.try_nuclide(nuclide!(fe-famous_mass)).unwrap();
    /// // note, that non-existing elements cannot be described via `nuclide` macro:
    /// // database.try_nuclide(nuclide!(Mi-348)).unwrap_err(); // no Mimicium :(
    /// // you can try doing that manually:
    /// # use sdecay::nuclide_spec::NumSpec;
    /// assert!(database.try_nuclide(NumSpec { z: 152, mass_number: 348, iso: None }).is_none()); // is this Mimicium? maybe?
    ///
    /// // using str
    /// assert!(database.try_nuclide("Dr-358").is_none()); // no draconium :(
    /// // using cstr
    /// let tungsten_184 = database.try_nuclide(c"W-184").unwrap();
    /// // using bytes
    /// assert!(database.try_nuclide(b"C-5").is_none()); // no carbon 5 (they must hide it really well!)
    /// // using other text types
    /// let uranium_235 = database.try_nuclide("U-235".to_string()).unwrap();
    /// # }
    /// ```
    #[inline]
    pub fn try_nuclide(&self, spec: impl NuclideSpec) -> Option<&Nuclide<'_>> {
        spec.get_nuclide(self)
    }

    /// Retrieves [`Nuclide`] from the database
    ///
    /// Note, that [`Nuclide`] is described as [`NuclideSpec`], see it's doc to find the best description for you
    ///
    /// ### Panics
    /// If described [`Nuclide`] is not present in the database
    ///
    /// ### Example
    /// ```rust
    /// # #[cfg(feature = "std")] {
    /// # use sdecay::database::Database;
    /// let database = Database::from_env().unwrap();
    /// # use sdecay::nuclide;
    /// // using `NumSpec` (created through `nuclide` macro)
    /// let tritium = database.nuclide(nuclide!(H-3));
    /// // (nucleus mass can be stored in a variable)
    /// let famous_mass = 56;
    /// let fe_56 = database.nuclide(nuclide!(fe-famous_mass));
    ///
    /// // using str
    /// // database.nuclide("Dr-358"); // (panics) no draconium :(
    /// // using cstr
    /// let tungsten_184 = database.nuclide(c"W-184");
    /// // using bytes
    /// // database.nuclide(b"C-5"); // (panics) no carbon 5 (they must hide it really well!)
    /// // using other text types
    /// let uranium_235 = database.nuclide("U-235".to_string());
    /// # }
    /// ```
    #[inline]
    pub fn nuclide(&self, spec: impl NuclideSpec) -> &Nuclide<'_> {
        spec.get_nuclide(self)
            .expect("Nuclide is not present in the database")
    }

    /// Retrieves [`Element`] from the database, if present
    ///
    /// Note, that [`Element`] is described as [`ElementSpec`], see it's doc to find the best description for you
    ///
    /// ### Example
    /// ```rust
    /// # #[cfg(feature = "std")] {
    /// # use sdecay::database::Database;
    /// let database = Database::from_env().unwrap();
    /// # use sdecay::element;
    /// // via integer (created through `element` macro)
    /// let hydrogen = database.try_element(element!(H)).unwrap();
    /// let ferrum = database.try_element(element!(fe)).unwrap();
    /// // note, that non-existing elements cannot be described via `element` macro:
    /// // database.element(element!(Mi)).unwrap_err(); // no Mimicium :(
    /// // you can try doing that manually:
    /// # use sdecay::element_spec::ElementNum;
    /// assert!(database.try_element(ElementNum(152)).is_none()); // is this Mimicium? maybe?
    ///
    /// // using str
    /// assert!(database.try_element("Dr").is_none()); // no draconium :(
    /// // using cstr
    /// let tungsten = database.try_element(c"W").unwrap();
    /// // using bytes
    /// let carbon = database.try_element(b"C").unwrap();
    /// // using other text types
    /// let uranium = database.try_element("U".to_string()).unwrap();
    /// # }
    /// ```
    #[inline]
    pub fn try_element(&self, spec: impl ElementSpec) -> Option<&Element<'_>> {
        spec.get_element(self)
    }

    /// Retrieves [`Element`] from the database
    ///
    /// Note, that [`Element`] is described as [`ElementSpec`], see it's doc to find the best description for you
    ///
    /// ### Panics
    /// If described [`Element`] is not present in the database
    ///
    /// ### Example
    /// ```rust
    /// # #[cfg(feature = "std")] {
    /// # use sdecay::database::Database;
    /// let database = Database::from_env().unwrap();
    /// # use sdecay::element;
    /// // via integer (created through `element` macro)
    /// let hydrogen = database.element(element!(H));
    /// let ferrum = database.element(element!(fe));
    /// // you can try doing that manually:
    /// // assert!(database.element(ElementNum(152))); // (panics) is this Mimicium? maybe?
    ///
    /// // using str
    /// // assert!(database.element("Dr").is_none()); // (panics) no draconium :(
    /// // using cstr
    /// let tungsten = database.element(c"W");
    /// // using bytes
    /// let carbon = database.element(b"C");
    /// // using other text types
    /// let uranium = database.element("U".to_string());
    /// # }
    /// ```
    #[inline]
    pub fn element(&self, spec: impl ElementSpec) -> &Element<'_> {
        spec.get_element(self)
            .expect("Element is not present in the database")
    }
}
