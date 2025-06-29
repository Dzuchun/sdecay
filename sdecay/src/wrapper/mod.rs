//! This module defines Rust representations of `SandiaDecay`'s types
//!
//! Unsafe: **YES**

use core::ffi::{c_double, c_float, c_short};

use crate::wrapper;

mod database;
pub use database::SandiaDecayDataBase;

mod nuclide;
pub use nuclide::Nuclide;

pub(crate) type BindgenString = sdecay_sys::sdecay::string;

mod stdstring;
pub use stdstring::StdString;

mod exception;
pub use exception::CppException;

mod vec;
use sdecay_sys::sdecay::{
    coincidence_pair_vec, energy_intensity_pair_vec, nuclide_abundance_pair_vec, rad_particle_vec,
    time_evolution_term_vec,
};
pub use vec::{
    VecChar, VecCoincidencePair, VecElementRef, VecEnergyCountPair, VecEnergyIntensityPair,
    VecEnergyRatePair, VecNuclideAbundancePair, VecNuclideRef, VecRadParticle,
    VecTimeEvolutionTerm, VecTransition, VecTransitionPtr,
};

mod enums;
pub use enums::*;

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

wrapper! {
    /// Information representing the nuclear transition (decay channel) for a nuclide
    #[derive(Debug)]
    sdecay_sys::sandia_decay::Transition => Transition['l] {
        /// Parent nuclide of the decay
        pub parent -> parent: *const sdecay_sys::sandia_decay::Nuclide => &'l Nuclide<'l>,
        /// The resultant nuclide after the decay
        ///
        /// May not be present, for example in case of spontaneous fission decay
        pub child -> child: *const sdecay_sys::sandia_decay::Nuclide => Option<&'l Nuclide<'l>>,
        /// Decay mode represented by this transition object
        pub mode -> mode: sdecay_sys::sandia_decay::DecayMode::Type => DecayMode,
        /// A fraction of times the parent nuclide decays through this decay channel
        pub branchRatio -> branch_ratio: c_float => f32,
        /// Particles ($\gamma$, $\beta$, $\alpha$, etc) emitted along this transition
        pub products -> products: rad_particle_vec => VecRadParticle,
        @pin: _pin,
        @no_constr: _no_constr,
    }
}

pub use coincidence_pair::CoincidencePair;
mod coincidence_pair {
    use core::ffi::c_ushort;

    /// I have NO idea what this pair means
    ///
    /// In case you happen to know, here it is -- easily accessible as Rust tuple
    #[derive(Debug)]
    #[repr(C)]
    pub struct CoincidencePair(pub c_ushort, pub f32);

    // assert same size and alignment
    const _: () = const {
        use core::mem::{align_of, size_of};
        assert!(size_of::<CoincidencePair>() == size_of::<sdecay_sys::sdecay::CoincidencePair>());
        assert!(align_of::<CoincidencePair>() == align_of::<sdecay_sys::sdecay::CoincidencePair>());
    };

    const _: () = const {
        use core::mem::{align_of, size_of};
        assert!(size_of::<f32>() == size_of::<core::ffi::c_float>());
        assert!(align_of::<f32>() == align_of::<core::ffi::c_float>());
    };
}

wrapper! {
    #[derive(Debug)]
    /// Represents information of a particle (of a given energy) given off during a nuclear transition.
    #[expect(non_snake_case)]
    sdecay_sys::sandia_decay::RadParticle => RadParticle {
        ///  Particle type
        pub type_ -> r#type: sdecay_sys::sandia_decay::ProductType::Type => ProductType,
        ///  Energy of the particle
        ///
        ///  Applies to all [`ProductType`]s
        pub energy -> energy: c_float => f32,
        /// Intensity of this particle for this decay channel ($\in [0; 1]$)
        ///
        /// Applies to all ProductTypes
        pub intensity -> intensity: c_float => f32,
        /// Hindrance TODO: help link
        ///
        /// Applies only to alpha decays
        pub hindrance -> hindrance: c_float => f32,
        /// is log-10 of fermi integral (F)*parent_half-life (T) TODO: help link
        ///
        /// Applies to beta, positron, and electronCapture decays
        pub logFT -> logFT: c_float => f32,
        /// Forbiddenss of the decay
        ///
        /// Applies to beta, positron, and electron capture decays
        pub forbiddenness -> forbiddenness: sdecay_sys::sandia_decay::ForbiddennessType::Type => ForbiddennessType,
        /// Other radiation particles expected to be detected when this particle is detected. Useful for gamma spectroscopy where you have a chance of detecting two gammas as one detection event (thus you detect the summed energy)
        ///
        /// Currently this information only includes gamma, and has not been well tested
        pub coincidences -> coincidences: coincidence_pair_vec => VecCoincidencePair,
        @pin: _pin,
        @no_constr: _no_constr,
    }
}

wrapper! {
    /// Holds the information about abundance (by mass) of a specific [`Nuclide`] in a [`crate::Mixture`], material, or [`Element`]
    #[derive(Debug)]
    sdecay_sys::sandia_decay::NuclideAbundancePair => NuclideAbundancePair['l] {
        #[expect(missing_docs)]
        pub nuclide -> nuclide: *const sdecay_sys::sandia_decay::Nuclide => &'l Nuclide<'l>,
        /// A fraction of this nuclide by mass
        pub abundance -> abundance: c_double => f64,
    }
}

wrapper! {
    /// Holds the activity of a specific [`Nuclide`] in a [`crate::Mixture`], material, or [`Element`]
    #[derive(Debug)]
    sdecay_sys::sandia_decay::NuclideActivityPair => NuclideActivityPair['l] {
        #[expect(missing_docs)]
        pub nuclide -> nuclide: *const sdecay_sys::sandia_decay::Nuclide => &'l Nuclide<'l>,
        /// Activity, in `SandiaDecay`'s units
        pub activity -> activity: c_double => f64,
    }
}

wrapper! {
    /// Holds the number of atoms of a specific [`Nuclide`] in a [`crate::Mixture`], material, or [`Element`]
    #[derive(Debug)]
    sdecay_sys::sandia_decay::NuclideNumAtomsPair => NuclideNumAtomsPair['l] {
        #[expect(missing_docs)]
        pub nuclide -> nuclide: *const sdecay_sys::sandia_decay::Nuclide => &'l Nuclide<'l>,
        #[expect(missing_docs)]
        pub numAtoms -> num_atoms: c_double => f64,
    }
}

wrapper! {
    /// Used to express the relative (to the number of decays) intensities of a specific-energy decay particles, e.g., specify what fraction of decay event will have a gamma of a certain energy
    #[derive(Debug)]
    sdecay_sys::sandia_decay::EnergyIntensityPair => EnergyIntensityPair {
        #[expect(missing_docs)]
        pub energy -> energy: c_double => f64,
        #[expect(missing_docs)]
        pub intensity -> intensity: c_double => f64,
    }
}

wrapper! {
    /// Used to return the energy and number of particles that are expected for a given time interval
    #[derive(Debug)]
    sdecay_sys::sandia_decay::EnergyCountPair => EnergyCountPair {
        #[expect(missing_docs)]
        pub energy -> energy: c_double => f64,
        #[expect(missing_docs)]
        pub count -> count: c_double => f64,
    }
}

wrapper! {
    /// Used to return the rate of a specific-energy decay particle, e.g., give the rate for a certain energy gamma
    #[derive(Debug)]
    sdecay_sys::sandia_decay::EnergyRatePair => EnergyRatePair {
        #[expect(missing_docs)]
        pub energy -> energy: c_double => f64,
        #[expect(missing_docs)]
        pub numPerSecond -> num_per_second: c_double => f64,
    }
}

wrapper! {
    /// Represents a chemical element, consisting of multiple isotopes
    #[derive(Debug)]
    sdecay_sys::sandia_decay::Element => Element['l] {
        /// Element's symbol, i.e. `H` for Hyddrogen, `Ar` for Argon, etc
        pub symbol -> symbol: BindgenString => StdString,
        /// Element's name, i.e. `Hydrogen` for Hydrogen, `Argon` for Argon, etc
        pub name -> name: BindgenString => StdString,
        /// Proton count in the nuclei of this element
        pub atomicNumber -> atomic_number: c_short => c_short,
        /// The isotopes which make up the natural abundance of an element
        ///
        /// The abundance is the fractional mass of each isotope (e.g. add up to 1.0), it may be the case that abundances are all exactly 0.0 if the element is not naturally occurring
        pub isotopes -> isotopes: nuclide_abundance_pair_vec => VecNuclideAbundancePair<'l>,
        /// Xrays that are caused by flouresence (e.g. not xrays produced as a result of a decay) exciting the element.
        ///
        /// Intensities are relative to the most intense (1.0) xray, and are only approximations to be used. Energies are only approximate as well
        ///
        /// Intended as a rough reference to be used when xrays are wanted without a decay. May not be present if this data wasn't in the XML data file
        ///
        /// Their presence can be checked via [`xml_contained_elemental_xray_info`](crate::wrapper::SandiaDecayDataBase::xml_contained_elemental_xray_info)
        pub xrays -> xrays: energy_intensity_pair_vec => VecEnergyIntensityPair,
    }
}

wrapper! {
    /// TODO: check if my understanding is actually correct
    /// A term in decay formula of the form
    /// $$
    /// \text{term_coeff} \cdot \exp( - \text{exponential_coeff} \cdot t )
    /// $$
    #[derive(Debug)]
    sdecay_sys::sandia_decay::TimeEvolutionTerm => TimeEvolutionTerm {
        #[expect(missing_docs)]
        pub termCoeff -> term_coeff: c_double => f64,
        #[expect(missing_docs)]
        pub exponentialCoeff -> exponential_coeff: c_double => f64,
    }
}

wrapper! {
    /// TODO: check is my understanding is actually correct
    /// A nuclide evolution solution, comprised a several [`TimeEvolutionTerm`]s added together
    #[derive(Debug)]
    sdecay_sys::sandia_decay::NuclideTimeEvolution => NuclideTimeEvolution['l] {
        #[expect(missing_docs)]
        pub nuclide -> nuclide: *const sdecay_sys::sandia_decay::Nuclide => &'l Nuclide<'l>,
        #[expect(missing_docs)]
        pub evolutionTerms -> evolution_terms: time_evolution_term_vec => VecTimeEvolutionTerm,
    }
}
