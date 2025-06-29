#![expect(unused_variables)]

#[cfg(feature = "alloc")]
use crate::database::{Database, UninitDatabase};
#[cfg(feature = "std")]
use crate::database::{SharedDatabase, UninitSharedDatabase};
use crate::{
    database::{LocalDatabase, UninitLocalDatabase},
    nuclide,
};

use core::mem::MaybeUninit;
#[cfg(feature = "std")]
use std::{println, sync::LazyLock};
#[cfg(not(feature = "std"))]
macro_rules! println {
    ($($input:tt)*) => {let _ = stringify!($($input),*);};
}

// #[cfg(feature = "alloc")]
// #[test]
// fn verify_valgrind_works() { // it sure does!
//     let buf = vec![0; 1024];
//     core::mem::forget(buf);
// }

const DATABASE_BYTES: &[u8] = include_bytes!("../sandia.decay.xml");

mod create_database {
    use super::*;

    #[test]
    #[cfg(feature = "alloc")]
    fn uninit() {
        let ud = UninitDatabase::default();
        core::mem::drop(ud);
    }

    #[test]
    #[cfg(feature = "std")]
    fn uninit_shared() {
        let ud = UninitSharedDatabase::default();
        core::mem::drop(ud);
    }

    #[test]
    fn uninit_local() {
        let mut tmp = MaybeUninit::uninit();
        let ld = UninitLocalDatabase::new_in(&mut tmp);
        core::mem::drop(ld);
    }
}

mod init {
    use super::*;

    #[test]
    #[cfg(feature = "alloc")]
    fn bad_path_error() {
        let uninit = UninitDatabase::default();
        let res = uninit.init(c"bad_non_existing_database.idk");
        let (_, error) = res.unwrap_err();
        println!("{error}");
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn ok_path() {
        let uninit = UninitDatabase::default();
        let res = uninit.init(c"sandia.decay.xml");
        let database = res.unwrap();
        println!("{database:?}");
    }

    #[test]
    fn ok_bytes() {
        let mut tmp = MaybeUninit::uninit();
        let uninit = UninitLocalDatabase::new_in(&mut tmp);
        let res = uninit.init_bytes(DATABASE_BYTES);
        let database = res.unwrap();
        println!("{database:?}");
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn empty_bytes() {
        let uninit = UninitDatabase::default();
        let res = uninit.init_bytes(b"meow; I'm a kitty, not xml data");
        let database = res.expect("bad xml input should not result in an error");
        println!("{database:?}");
    }

    #[test]
    #[cfg(feature = "std")]
    fn ok_env() {
        let uninit = UninitDatabase::default();
        let database = uninit.init_env().unwrap();
        println!("{database:?}");
    }
}

mod deinit {
    use super::*;

    #[test]
    #[cfg(feature = "alloc")]
    fn ok_deinit() {
        let database =
            Database::from_bytes(DATABASE_BYTES).expect("Should be able to init database");
        database
            .reset()
            .expect("Should be able to de-init database");
    }

    #[test]
    #[cfg(feature = "std")]
    fn ok_deinit_shared() {
        let database = SharedDatabase::from_bytes(DATABASE_BYTES)
            .expect("Should be able to init shared database");
        database
            .reset()
            .expect("Should be able to de-init database");
    }

    #[test]
    #[cfg(feature = "std")]
    fn ok_deinit_shared_cloned() {
        let database = SharedDatabase::from_bytes(DATABASE_BYTES)
            .expect("Should be able to init shared database");
        let database2 = database.clone();
        database
            .reset()
            .expect_err("Should not be able to reset the database through non-exclusive container");
        database2
            .reset()
            .expect("Should be able to de-init database");
    }

    #[test]
    fn ok_deinit_local() {
        let mut tmp = MaybeUninit::uninit();
        let database = LocalDatabase::from_bytes_in(&mut tmp, DATABASE_BYTES)
            .expect("Should be able to init shared database");
        database
            .reset()
            .expect("Should be able to de-init database");
    }
}

#[cfg(feature = "std")]
static DATABASE: LazyLock<Database> = LazyLock::new(|| {
    Database::from_bytes(DATABASE_BYTES)
        .expect("should be able to initialize database from embedded bytes")
});

#[cfg(feature = "std")]
macro_rules! database {
    ($db:ident) => {
        let $db = &*DATABASE;
    };
}

#[cfg(not(feature = "std"))]
macro_rules! database {
    ($db:ident) => {
        let mut $db = MaybeUninit::uninit();
        let $db = LocalDatabase::from_bytes_in(&mut $db, DATABASE_BYTES)
            .expect("should be able to initialize database from embedded bytes");
    };
}

mod get_nuclide {
    use super::*;

    #[test]
    fn get_h1_ok() {
        database!(db);
        let nuc = db.nuclide(nuclide![H - 1]);
        println!("{nuc:#?}");
    }

    #[test]
    fn get_h6_ok() {
        database!(db);
        let nuc = db.nuclide(nuclide![H - 6]);
        println!("{nuc:#?}");
    }

    /// NOTE: H-7 actually was registered, so this result is kinda sus :think:
    #[test]
    fn get_h7_err() {
        database!(db);
        assert!(db.try_nuclide(nuclide![H - 7]).is_none());
    }

    /// Draconium does not exist :(
    #[test]
    fn get_dr282_err() {
        database!(db);
        assert!(db.try_nuclide("Dr282").is_none());
        // NOTE: following actually does not compile:
        // DATABASE.nuclide(nuclide![Dr - 282]).unwrap_err();
        // (since there's no Dr element)
    }
}

macro_rules! h1 {
    ($h1:ident) => {
        database!($h1);
        let $h1 = $h1.nuclide(nuclide!(H - 1));
    };
}

mod nuclide {
    use super::*;

    #[test]
    fn h1_human_str_summary() {
        h1!(h1);
        let mut tmp = MaybeUninit::uninit();
        let summary = h1.human_str_summary_local(&mut tmp);
        println!("{summary:#?}");
    }

    #[test]
    fn h1_descendants() {
        h1!(h1);
        let mut tmp = MaybeUninit::uninit();
        let descendants = h1.descendants_local(&mut tmp);
        println!("{descendants:#?}");
    }

    #[test]
    fn ar42() {
        database!(db);
        let ar42 = db.nuclide(nuclide!(Ar - 42));
        assert!(&ar42.symbol == "Ar42");
    }
}

mod mixture {
    use approx::assert_relative_eq;

    use crate::{
        cst::{Ci, day},
        nuclide_mixture::AgedNuclideError,
        wrapper::{HowToOrder, ProductType},
    };

    use super::*;

    macro_rules! mixture {
        ($name:ident) => {
            let mut $name = core::mem::MaybeUninit::uninit();
            #[allow(unused_mut)]
            let mut $name = crate::LocalMixture::new_in(&mut $name);
        };
    }

    #[test]
    fn create() {
        mixture!(mx);
    }

    #[test]
    fn empty_activity() {
        mixture!(mx);

        mx.total_activity(0.0);
    }

    #[test]
    fn one_activity() {
        database!(db);
        mixture!(mx);

        let activity = 1e-6 * Ci;
        let h3 = db.nuclide(nuclide!(H - 3));
        mx.add_nuclide_by_activity(h3, activity);

        assert_relative_eq!(mx.total_activity(0.0), activity);
    }

    #[test]
    fn two_activity() {
        database!(db);
        mixture!(mx);

        let activity = 1e-6 * Ci;
        let h3 = db.nuclide(nuclide!(H - 3));
        mx.add_nuclide_by_activity(h3, activity);
        let u238 = db.nuclide(nuclide!(U - 238));
        mx.add_nuclide_by_activity(u238, activity);

        assert_relative_eq!(mx.total_activity(0.0), 2.0 * activity, epsilon = 1e-3);
    }

    #[test]
    fn empty_clear_ok() {
        mixture!(mx);
        mx.clear();
    }

    #[test]
    fn empty_evolutions_ok() {
        mixture!(mx);
        let evolutions = mx.decayed_to_nuclides_evolutions();
        println!("{evolutions:?}");
        assert!(evolutions.is_empty());
    }

    #[test]
    fn empty_activity_ok() {
        mixture!(mx);
        let _ = mx.total_activity(0.0);
    }

    #[test]
    fn empty_activities_ok() {
        mixture!(mx);
        let mut tmp = MaybeUninit::uninit();
        let activities = mx.activities_local(&mut tmp, 0.0);
        println!("{activities:?}");
        assert!(activities.is_empty());
    }

    #[test]
    fn empty_pacticles_ok() {
        mixture!(mx);
        let mut tmp = MaybeUninit::uninit();
        let particles = mx.decay_particle_local(
            &mut tmp,
            0.0,
            ProductType::BetaParticle,
            HowToOrder::OrderByEnergy,
        );
        println!("{particles:?}");
        assert!(particles.is_empty());
    }

    #[test]
    fn empty_photons_ok() {
        mixture!(mx);
        let mut tmp = MaybeUninit::uninit();
        let photons = mx.photons_local(&mut tmp, 0.0, HowToOrder::OrderByEnergy);
        println!("{photons:?}");
        assert_eq!(photons.len(), 1, "should include annihilation gammas only");
    }

    #[test]
    fn empty_particles_interval_ok() {
        mixture!(mx);
        let mut tmp = MaybeUninit::uninit();
        let photons = mx.decay_photons_in_interval_local(
            &mut tmp,
            0.0,
            day,
            HowToOrder::OrderByAbundance,
            1000,
        );
        println!("{photons:?}");
        assert!(photons.is_empty());
    }

    #[test]
    fn empty_photons_interval_ok() {
        mixture!(mx);
        let mut tmp = MaybeUninit::uninit();
        let particles = mx.decay_particles_in_interval_local(
            &mut tmp,
            0.0,
            day,
            ProductType::XrayParticle,
            HowToOrder::OrderByEnergy,
            1000,
        );
        println!("{particles:?}");
        assert!(particles.is_empty());
    }

    #[test]
    fn empty_num_atoms_ok() {
        mixture!(mx);
        let mut tmp = MaybeUninit::uninit();
        let _ = mx.num_atoms_local(&mut tmp, 0.0);
    }

    #[test]
    fn absent_nuclide() {
        h1!(h1);
        mixture!(mx);

        mx.add_nuclide_by_activity(h1, 1e-6 * Ci);

        assert!(mx.nuclide_activity(0.0, nuclide!(U - 238)).is_none());
    }

    /// Aims to ensure that `CppException::what` can be called more than once
    ///
    /// This is not obvious, since at the moment it uses `std::rethrow_exception`
    #[test]
    fn read_error_twice_ok() {
        database!(db);
        mixture!(mx);
        let nuclide = db.nuclide(nuclide!(U - 235));
        println!("{nuclide:?}");
        let Err(AgedNuclideError::Exception(exception)) = dbg!(mx.add_aged_nuclide_by_num_atoms(
            nuclide,
            nuclide.atoms_per_gram(), // 1g
            nuclide.half_life * 1000.0
        )) else {
            panic!("Should result in an exception");
        };
        println!("{}", exception.what_str());
        println!("{}", exception.what_str());
    }
}
