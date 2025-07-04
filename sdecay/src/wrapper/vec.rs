use core::ffi::c_char;

use crate::{
    container::Container,
    containers, vec_wrapper,
    wrapper::{
        CoincidencePair, Element, EnergyCountPair, EnergyIntensityPair, EnergyRatePair, Nuclide,
        NuclideAbundancePair, NuclideActivityPair, NuclideNumAtomsPair, NuclideTimeEvolution,
        RadParticle, TimeEvolutionTerm, Transition,
    },
};

vec_wrapper! { transition_ptr['l], *const sdecay_sys::sandia_decay::Transition, &'l Transition<'l> }
vec_wrapper! { rad_particle, sdecay_sys::sandia_decay::RadParticle, RadParticle}
vec_wrapper! {coincidence_pair, sdecay_sys::sdecay::CoincidencePair, CoincidencePair}
vec_wrapper! { nuclide_abundance_pair['l], sdecay_sys::sandia_decay::NuclideAbundancePair, NuclideAbundancePair<'l> }
vec_wrapper! { energy_intensity_pair, sdecay_sys::sandia_decay::EnergyIntensityPair, EnergyIntensityPair }
vec_wrapper! { energy_count_pair, sdecay_sys::sandia_decay::EnergyCountPair, EnergyCountPair }
vec_wrapper! { energy_rate_pair, sdecay_sys::sandia_decay::EnergyRatePair, EnergyRatePair }
vec_wrapper! { nuclide_raw_ptr, *const sdecay_sys::sandia_decay::Nuclide, *const sdecay_sys::sandia_decay::Nuclide }
vec_wrapper! { nuclide_ref['l], *const sdecay_sys::sandia_decay::Nuclide, &'l Nuclide<'l> }
vec_wrapper! { element_raw_ptr, *const sdecay_sys::sandia_decay::Element, *const sdecay_sys::sandia_decay::Element }
vec_wrapper! { element_ref['l], *const sdecay_sys::sandia_decay::Element, &'l Element<'l> }
vec_wrapper! { element['l], sdecay_sys::sandia_decay::Element, Element<'l> }
vec_wrapper! { nuclide['l], sdecay_sys::sandia_decay::Nuclide, Nuclide<'l> }
vec_wrapper! { transition['l], sdecay_sys::sandia_decay::Transition, Transition<'l> }
vec_wrapper! { time_evolution_term, sdecay_sys::sandia_decay::TimeEvolutionTerm, TimeEvolutionTerm }
vec_wrapper! { nuclide_time_evolution['l], sdecay_sys::sandia_decay::NuclideTimeEvolution, NuclideTimeEvolution<'l> }

vec_wrapper! { char, c_char, c_char}
impl VecChar {
    /// Allocates new `std::vector<char>` storing provided data
    pub fn from_bytes_in<C: Container<Inner = Self>>(
        allocator: C::Allocator,
        data: impl AsRef<[u8]>,
    ) -> C {
        let data = data.as_ref();
        let ptr = data.as_ptr();
        let len = data.len();
        let init = |vec: *mut VecChar| {
            // SAFETY:
            // - `ptr` and `len` define a valid slice of bytes on Rust side
            // - `vec` points to uninitialized `VecChar` and is valid for writes
            unsafe {
                sdecay_sys::sdecay::std_vector_char_from_data(
                    ptr.cast::<core::ffi::c_char>(),
                    len,
                    vec.cast(),
                );
            }
        };
        // SAFETY: `init` initializes memory on C++ side by moving a valid `VecChar` into it
        unsafe { C::init_ptr(allocator, init) }
    }

    /// Same as [`Self::from_bytes_in`], but obtains `C::Allocator` from it's [`Default`] implementation
    pub fn from_bytes<C: Container<Inner = Self>>(bytes: impl AsRef<[u8]>) -> C
    where
        C::Allocator: Default,
    {
        Self::from_bytes_in(C::Allocator::default(), bytes)
    }
}

vec_wrapper! { nuclide_activity_pair['l], sdecay_sys::sandia_decay::NuclideActivityPair, NuclideActivityPair<'l> }
containers! { VecNuclideActivityPair['l]: sdecay_sys::sdecay::database::decay_activities =>
    /// Simulates decay of a nuclide mixture, returning evolved mixture
    decay(
        time: f64 => time
    ) -> VecNuclideActivityPair['l]
}
containers! { VecNuclideActivityPair['l]: sdecay_sys::sdecay::database::evolution_activities =>
    /// Solves for evolution of a nuclide mixture
    evolution() -> VecNuclideTimeEvolution['l]
}

impl VecNuclideActivityPair<'_> {
    /// Simulates decay of a nuclide mixture, assigning the result to itself
    pub fn decay_assign(self: core::pin::Pin<&mut Self>, time: f64) {
        let self_ptr = self.ptr_mut().cast();
        // SAFETY: ffi call with
        // - statically validated type representations
        // - correct pointer constness (as of bindgen, that is)
        // - `self_ptr` points to a live object, since it was just created from reference
        unsafe { sdecay_sys::sdecay::database::decay_activities_assign(self_ptr, time) };
    }
}

vec_wrapper! { nuclide_num_atoms_pair['l], sdecay_sys::sandia_decay::NuclideNumAtomsPair, NuclideNumAtomsPair<'l> }
containers! { VecNuclideNumAtomsPair['l]: sdecay_sys::sdecay::database::decay_atoms =>
    /// Simulates decay of a nuclide mixture, returning evolved mixture
    decay(
        time: f64 => time
    ) -> VecNuclideActivityPair['l]
}
containers! { VecNuclideNumAtomsPair['l]: sdecay_sys::sdecay::database::evolution_atoms =>
    /// Solves for evolution of a nuclide mixture
    evolution() -> VecNuclideTimeEvolution['l]
}
