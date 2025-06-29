#![allow(clippy::too_many_lines, clippy::items_after_statements, missing_docs)]

use sdecay::{
    Database, Mixture,
    cst::{Ci, day, hour, keV, second},
    element, nuclide,
    wrapper::{
        DecayModeD, EnergyCountPair, EnergyIntensityPair, EnergyRatePair, HowToOrder,
        NuclideAbundancePair, NuclideActivityPair, NuclideTimeEvolution, ProductType, RadParticle,
        TimeEvolutionTerm, Transition,
    },
};

fn main() {
    const DATABASE_PATH: &str = env!("SANDIA_DATABASE_PATH");
    let database = Database::from_path(DATABASE_PATH).unwrap();

    println!("constructing mixture");
    {
        // for nuc in database.nuclides() {
        //     let sec_hl = nuc.secular_equilibrium_half_life();
        //     assert!(
        //         sec_hl >= nuc.half_life || sec_hl <= 0.0 || nuc.isomer_number != 0,
        //         "{nuc:?}"
        //     );
        // }

        let mut mixture = Mixture::new();
        let k40 = database.nuclide(nuclide!(k - 40));
        assert!(mixture.add_nuclide(NuclideActivityPair {
            nuclide: k40,
            activity: 1e-6 * Ci
        }));
        println!("{mixture:?}");

        mixture.clear();
        let rn220 = database.nuclide(nuclide!(Rn - 220));
        mixture
            .add_aged_nuclide_by_activity(rn220, 1e-6 * Ci, 10.0 * second)
            .unwrap();
        println!("{mixture:?}");

        mixture.clear();
        let ar42 = database.nuclide(nuclide!(Ar - 42));
        mixture
            .add_nuclide_in_secular_equilibrium(ar42, 1e-6 * Ci)
            .unwrap();
        println!("{mixture:?}");

        mixture.clear();
        let u238 = database.nuclide(nuclide!(U - 238));
        assert!(mixture.add_nuclide_in_prompt_equilibrium(u238, 1e-6 * Ci));
        println!("{mixture:?}");
    }
    println!("activities");
    {
        let mut mixture = Mixture::new();
        let rn221 = database.nuclide(nuclide!(Rn - 221));
        assert!(mixture.add_nuclide(NuclideActivityPair {
            nuclide: rn221,
            activity: 1e-6 * Ci
        }));
        for NuclideActivityPair { nuclide, activity } in mixture.activities(hour).iter() {
            println!("{:9.3e} Ci of {}", activity / Ci, nuclide.symbol);
        }
    }
    println!("decay particles");
    {
        let mut mixture = Mixture::new();
        let ar42 = database.nuclide(nuclide!(Ar - 42));
        assert!(mixture.add_nuclide(NuclideActivityPair {
            nuclide: ar42,
            activity: 1e-6 * Ci
        }));
        println!("(gamma)");
        for EnergyRatePair {
            energy,
            num_per_second,
        } in &mixture.decay_particle(
            hour,
            ProductType::GammaParticle,
            HowToOrder::OrderByAbundance,
        ) {
            println!("{:7.3e} keV at {:.2e}/second", energy / keV, num_per_second);
        }
        println!("(e-)");
        for EnergyRatePair {
            energy,
            num_per_second,
        } in &mixture.decay_particle(
            hour,
            ProductType::BetaParticle,
            HowToOrder::OrderByAbundance,
        ) {
            println!("{:7.3e} keV at {:.2e}/second", energy / keV, num_per_second);
        }
    }
    println!("decay particles in interval");
    {
        let mut mixture = Mixture::new();
        let c10 = database.nuclide(nuclide!(c - 9));
        assert!(mixture.add_nuclide(NuclideActivityPair {
            nuclide: c10,
            activity: 1e-6 * Ci
        }));
        println!("(nu)");
        for EnergyCountPair { energy, count } in &mixture.decay_particles_in_interval(
            0.0,
            day,
            ProductType::CaptureElectronParticle,
            HowToOrder::OrderByEnergy,
            1000,
        ) {
            println!("{:7.3e} keV at {:.2e}", energy / keV, count);
        }
        println!("(e+)");
        for EnergyCountPair { energy, count } in &mixture.decay_particles_in_interval(
            0.0,
            day,
            ProductType::PositronParticle,
            HowToOrder::OrderByEnergy,
            1000,
        ) {
            println!("{:7.3e} keV at {:.2e}", energy / keV, count);
        }
    }
    println!("nuclide evolutions");
    {
        let mut mixture = Mixture::new();
        let ne24 = database.nuclide(nuclide!(ne - 24));
        assert!(mixture.add_nuclide(NuclideActivityPair {
            nuclide: ne24,
            activity: 1e-6 * Ci
        }));
        for NuclideTimeEvolution {
            nuclide,
            evolution_terms,
        } in mixture.decayed_to_nuclides_evolutions()
        {
            print!("N({}, t) = ", nuclide.symbol);
            let mut terms = evolution_terms.into_iter();
            if let Some(TimeEvolutionTerm {
                term_coeff,
                exponential_coeff,
            }) = terms.next()
            {
                print!(
                    "{}{term_coeff:.3e} * exp({:+.3e} * t)",
                    if exponential_coeff.is_sign_negative() {
                        "-"
                    } else {
                        ""
                    },
                    exponential_coeff.abs()
                );
            } else {
                print!("(nothing?)");
            }
            for TimeEvolutionTerm {
                term_coeff,
                exponential_coeff,
            } in evolution_terms
            {
                print!(
                    " {} {term_coeff:.3e} * exp({:+.3e} * t)",
                    if exponential_coeff.is_sign_positive() {
                        "+"
                    } else if exponential_coeff.is_sign_negative() {
                        "-"
                    } else {
                        "+"
                    },
                    exponential_coeff.abs()
                );
            }
            println!();
        }
    }

    println!("element abundances");
    {
        let el = database.element(element!(W));
        for NuclideAbundancePair { nuclide, abundance } in &el.isotopes {
            println!("{:6.3}% of {}", abundance * 100.0, nuclide.symbol);
        }
    }

    println!("element xrays");
    {
        let el = database.element(element!(W));
        for EnergyIntensityPair { energy, intensity } in &el.xrays {
            println!(
                "{:.1} keV ({:.3} relative intensity)",
                energy / keV,
                intensity
            );
        }
    }

    println!("nuclide transitions");
    {
        fn mode_additions(mode: DecayModeD) -> (&'static str, &'static str) {
            match mode {
                DecayModeD::AlphaDecay => ("", " + \\alpha"),
                DecayModeD::BetaDecay => ("", " + e^{-} + \\tilde{\\nu}"),
                DecayModeD::BetaPlusDecay => ("", " + e^{+} + \\nu"),
                DecayModeD::ProtonDecay => ("", " + p"),
                DecayModeD::IsometricTransitionDecay => ("", " + \\gamma"),
                DecayModeD::BetaAndNeutronDecay => ("", " + n + e^{-} + \\tilde{\\nu}"),
                DecayModeD::BetaAndTwoNeutronDecay => ("", " + 2n + e^{-} + \\tilde{\\nu}"),
                DecayModeD::ElectronCaptureDecay => ("(EC)", " + \\nu"),
                DecayModeD::ElectronCaptureAndProtonDecay => ("(EC)", " + p + \\nu"),
                DecayModeD::ElectronCaptureAndAlphaDecay => ("(EC)", " + \\alpha + \\nu"),
                DecayModeD::ElectronCaptureAndTwoProtonDecay => ("(EC)", " + 2p + \\nu"),
                DecayModeD::BetaAndAlphaDecay => ("", " + \\alpha + e^{-} + \\tilde{\\nu}"),
                DecayModeD::BetaPlusAndProtonDecay => ("", " + p + e^{+} + \\nu"),
                DecayModeD::BetaPlusAndTwoProtonDecay => ("", " + 2p + e^{+} + \\nu"),
                DecayModeD::BetaPlusAndThreeProtonDecay => ("", " + 3p + e^{+} + \\nu"),
                DecayModeD::BetaPlusAndAlphaDecay => ("", " + \\alpha + e^{+} + \\nu"),
                DecayModeD::DoubleBetaDecay => ("", " + 2e^{-} + 2\\tilde{\\nu}"),
                DecayModeD::DoubleElectronCaptureDecay => ("(2EC)", " + 2\\nu"),
                DecayModeD::Carbon14Decay => ("", " + ?"),
                DecayModeD::DoubleProton => ("", " + 2p"),
                DecayModeD::SpontaneousFissionDecay
                | DecayModeD::ClusterDecay
                | DecayModeD::UndefinedDecay
                | DecayModeD::Unknown => ("", ""),
            }
        }
        for nuc in database.nuclides() {
            if nuc.decays_to_children.len() > 2 && nuc.decay_from_parents.len() > 2 {
                println!("{}", nuc.symbol);
            }
        }

        let nuc = database.nuclide(nuclide!(Es - 247));
        for Transition {
            parent,
            mode,
            branch_ratio,
            products,
            ..
        } in &nuc.decay_from_parents
        {
            let (from_eq, to_eq) = mode_additions(mode.d());
            println!(
                "> {branch_ratio:.2}: \\text{{{}}}{} -> \\text{{{}}}{}",
                parent.symbol, from_eq, nuc.symbol, to_eq,
            );
            for RadParticle {
                r#type,
                energy,
                intensity,
                ..
            } in products
            {
                println!(
                    "| {intensity:.2} of {} at {:.2} keV",
                    r#type,
                    energy / (keV as f32)
                );
            }
        }
        for Transition {
            child,
            mode,
            branch_ratio,
            products,
            ..
        } in &nuc.decays_to_children
        {
            let (from_eq, to_eq) = mode_additions(mode.d());
            println!(
                "< {branch_ratio:.2}: \\text{{{}}}{} -> \\text{{{}}}{}",
                nuc.symbol,
                from_eq,
                child
                    .as_ref()
                    .map_or("(fission)".into(), |n| n.symbol.as_str()),
                to_eq,
            );
            for RadParticle {
                r#type,
                energy,
                intensity,
                ..
            } in products
            {
                println!(
                    "| {intensity:.2} of {} at {:.2} keV",
                    r#type,
                    energy / (keV as f32)
                );
            }
        }
    }
}
