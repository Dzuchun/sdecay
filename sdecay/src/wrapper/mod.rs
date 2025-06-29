//! This module defines Rust representations of `SandiaDecay`'s types
//!
//! Unsafe: **YES**

use core::ffi::c_double;

use crate::wrapper;

pub(crate) type BindgenString = sdecay_sys::sdecay::string;

mod stdstring;
pub use stdstring::StdString;

mod exception;
pub use exception::CppException;

mod vec;
pub use vec::{
    VecChar, VecCoincidencePair, VecEnergyCountPair, VecEnergyIntensityPair, VecEnergyRatePair,
    VecTimeEvolutionTerm,
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
