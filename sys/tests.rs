#![expect(unsafe_code)]

use core::{ffi::CStr, mem::MaybeUninit};

use crate::sandia_decay::{Ci, year};

macro_rules! string {
    ($name:ident = $cstr:expr) => {
        let mut $name = MaybeUninit::uninit();
        crate::sdecay::std_string_from_cstr($name.as_mut_ptr(), $cstr.as_ptr());
        let $name = $name.as_mut_ptr();
    };
}

macro_rules! string_drop {
    ($name:ident) => {
        crate::sdecay::std_string_destruct($name);
    };
}

#[test]
fn create_std_string() {
    unsafe {
        string!(s = c"abc42");
        assert!(CStr::from_ptr(crate::sdecay::std_string_cstr(s.cast_const())) == c"abc42");
        string_drop!(s);
    }
}

macro_rules! database {
    ($name:ident) => {
        let mut $name = MaybeUninit::uninit();
        crate::sandia_decay::SandiaDecayDataBase_SandiaDecayDataBase1($name.as_mut_ptr());
        string!(path = c"vendor/sandia.decay.xml");
        crate::sandia_decay::SandiaDecayDataBase_initialize($name.as_mut_ptr(), path);
        string_drop!(path);
        let $name = $name.as_mut_ptr();
    };
}

macro_rules! database_drop {
    ($name:ident) => {
        crate::sandia_decay::SandiaDecayDataBase_SandiaDecayDataBase_destructor($name);
    };
}

#[test]
fn create_database() {
    unsafe {
        // create and initialize database
        database!(db);
        // drop the database
        database_drop!(db);
    }
}

#[test]
fn get_nuclide() {
    unsafe {
        database!(db);
        string!(nuc = c"U-238");
        let nuclide = &*crate::sandia_decay::SandiaDecayDataBase_nuclide(db, nuc);
        string_drop!(nuc);
        let symbol = &nuclide.symbol;
        let symbol = crate::sdecay::std_string_cstr(core::ptr::from_ref(symbol));
        assert!(CStr::from_ptr(symbol) == c"U238");
        database_drop!(db);
    }
}

macro_rules! mixture {
    ($name:ident) => {
        let mut $name = MaybeUninit::uninit();
        crate::sandia_decay::NuclideMixture_NuclideMixture($name.as_mut_ptr());
        let $name = $name.as_mut_ptr();
    };
}

macro_rules! mixture_drop {
    ($name:ident) => {
        crate::sandia_decay::NuclideMixture_NuclideMixture_destructor($name);
    };
}

#[test]
fn create_mixture() {
    unsafe {
        // create empty mixture
        mixture!(mx);
        // drop mixture
        mixture_drop!(mx);
    }
}

#[test]
fn populate_mixture() {
    unsafe {
        mixture!(mx);
        database!(db);
        string!(nuc = c"U-238");
        let nuclide = &*crate::sandia_decay::SandiaDecayDataBase_nuclide(db, nuc);
        string_drop!(nuc);
        crate::sandia_decay::NuclideMixture_addNuclideByActivity(mx, nuclide, 1e-3 * Ci);
        database_drop!(db);
        mixture_drop!(mx);
    }
}

#[test]
fn solve_mixture() {
    unsafe {
        mixture!(mx);
        database!(db);
        string!(nuc = c"U-238");
        let nuclide = &*crate::sandia_decay::SandiaDecayDataBase_nuclide(db, nuc);
        string_drop!(nuc);
        crate::sandia_decay::NuclideMixture_addNuclideByActivity(mx, nuclide, 1e-3 * Ci);
        let _activity_20y =
            crate::sandia_decay::NuclideMixture_totalActivity(mx.cast_const(), 20.0 * year);
        database_drop!(db);
        mixture_drop!(mx);
    }
}

#[test]
fn perform_evolution() {
    unsafe {
        mixture!(mx);
        database!(db);
        string!(nuc = c"U-238");
        let nuclide = &*crate::sandia_decay::SandiaDecayDataBase_nuclide(db, nuc);
        string_drop!(nuc);
        crate::sandia_decay::NuclideMixture_addNuclideByActivity(mx, nuclide, 1e-3 * Ci);
        let _evolution =
            crate::sandia_decay::NuclideMixture_decayedToNuclidesEvolutions(mx.cast_const());
        database_drop!(db);
        mixture_drop!(mx);
    }
}

mod layout {
    macro_rules! layout {
        ($name:ident, $type:ty) => {
            mod $name {
                #[test]
                fn size() {
                    let rust_size = core::mem::size_of::<$type>();
                    let c_size = unsafe { crate::sdecay::layout::$name::size };
                    assert_eq!(rust_size, c_size, "Binding type should have correct size");
                }

                #[test]
                fn align() {
                    let rust_align = core::mem::align_of::<$type>();
                    let c_align = unsafe { crate::sdecay::layout::$name::align };
                    assert_eq!(
                        rust_align, c_align,
                        "Binding type should have correct alignment"
                    );
                }
            }
        };
    }

    layout!(std_string, crate::sdecay::string);
    layout!(database, crate::sandia_decay::SandiaDecayDataBase);
    layout!(mixture, crate::sandia_decay::NuclideMixture);
    layout!(nuclide, crate::sandia_decay::Nuclide);
    layout!(transition, crate::sandia_decay::Transition);
    layout!(rad_particle, crate::sandia_decay::RadParticle);
    layout!(
        nuclide_abundance_pair,
        crate::sandia_decay::NuclideAbundancePair
    );
    layout!(
        nuclide_activity_pair,
        crate::sandia_decay::NuclideActivityPair
    );
    layout!(
        nuclide_num_atoms_pair,
        crate::sandia_decay::NuclideNumAtomsPair
    );
    layout!(
        energy_intensity_pair,
        crate::sandia_decay::EnergyIntensityPair
    );
    layout!(energy_count_pair, crate::sandia_decay::EnergyCountPair);
    layout!(energy_rate_pair, crate::sandia_decay::EnergyRatePair);
    layout!(element, crate::sandia_decay::Element);
    layout!(time_evolution_term, crate::sandia_decay::TimeEvolutionTerm);
    layout!(
        nuclide_time_evolution,
        crate::sandia_decay::NuclideTimeEvolution
    );

    macro_rules! layout_sdecay {
        ($name:ident) => {
            layout!($name, crate::sdecay::$name);
        };
    }

    layout_sdecay!(char_vec);
    layout_sdecay!(transition_vec);
    layout_sdecay!(transition_ptr_vec);
    layout_sdecay!(rad_particle_vec);
    layout_sdecay!(nuclide_abundance_pair_vec);
    layout_sdecay!(nuclide_activity_pair_vec);
    layout_sdecay!(nuclide_num_atoms_pair_vec);
    layout_sdecay!(energy_intensity_pair_vec);
    layout_sdecay!(energy_count_pair_vec);
    layout_sdecay!(energy_rate_pair_vec);
    layout_sdecay!(nuclide_vec);
    layout_sdecay!(nuclide_ref_vec);
    layout_sdecay!(nuclide_raw_ptr_vec);
    layout_sdecay!(element_vec);
    layout_sdecay!(element_raw_ptr_vec);
    layout_sdecay!(element_ref_vec);
    layout_sdecay!(coincidence_pair_vec);
    layout_sdecay!(time_evolution_term_vec);
    layout_sdecay!(nuclide_time_evolution_vec);
}
