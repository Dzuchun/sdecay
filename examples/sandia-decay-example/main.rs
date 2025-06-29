//! This is a recreation of `sandia_decay_example` example from the original repo
#![allow(clippy::too_many_lines, clippy::items_after_statements, missing_docs)]

use std::{ffi::CString, mem::MaybeUninit};

use anyhow::{Context, ensure};
use clap::Parser;
use sdecay::{
    LocalDatabase, LocalMixture,
    cst::{Bq, Ci, becquerel, curie, day, hour, keV, second, year},
    nuclide,
    wrapper::{
        CoincidencePair, EnergyCountPair, EnergyRatePair, HowToOrder, Nuclide, NuclideActivityPair,
        NuclideNumAtomsPair, ProductType,
    },
};

fn pmin<T: PartialOrd>(a: T, b: T) -> T {
    if a < b { a } else { b }
}

macro_rules! try_block {
    { $($token:tt)* } => {{
        #[allow(clippy::redundant_closure_call)]
        (|| {
            $($token)*
        })()
    }}
}

#[derive(Debug, clap::Parser)]
struct Args {
    #[arg(long, default_value = "sandia.decay.xml")]
    xml_path: CString,
}

fn main() -> anyhow::Result<()> {
    let args = Args::try_parse().context("parsing clargs")?;
    println!("{args:#?}");

    let mut tmp = MaybeUninit::uninit();
    let database =
        LocalDatabase::from_path_in(&mut tmp, &args.xml_path).context("initializing database")?;

    try_block! {
        let time_to_age = 100.0 * day;
        let initial_parent_activity = 0.001 * curie;
        let nuclide = database.try_nuclide(nuclide![Ba-133]).context("getting nuclide from database")?;

        ensure!(initial_parent_activity > 0.0);
        ensure!(time_to_age > 0.0 && time_to_age < 10.0 * nuclide.half_life);

        let mut tmp = MaybeUninit::uninit();
        let mut mixture = LocalMixture::new_in(&mut tmp);
        ensure!(mixture.add_nuclide_by_activity(nuclide, initial_parent_activity), "adding nuclide to the mixture");
        dbg!(&mixture);

        let mut tmp = MaybeUninit::uninit();
        let activities = mixture.activities_local(&mut tmp, time_to_age);

        for NuclideActivityPair { nuclide, activity } in &activities {
            println!("\t- {nuclide}: {:.9} curie{}", activity / curie, if nuclide.is_stable() {" (stable)"} else {""});
        }

        // Original note:
        //  // Note: do not delete or free `nuclide` as the object it points to is owned by
        //  // SandiaDecayDataBase and will be cleaned up when `database` desctructs.
        //
        // jokes on you, we can't do that in Rust!

        anyhow::Ok(())
    }
    .context("printing the activity of a nuclide and its descendants after some aging")?;

    try_block!{
        let time_to_age = 100.0 * day;
        let initial_parent_activity = 0.001 * curie;

        let nuclide = database.try_nuclide(nuclide![Ba-133]).context("getting nuclide from database")?;

        ensure!(initial_parent_activity > 0.0);
        ensure!(time_to_age > 0.0 && time_to_age < 10.0 * nuclide.half_life);

        let mut tmp = MaybeUninit::uninit();
        let mut mixture = LocalMixture::new_in(&mut tmp);
        mixture.add_nuclide_by_activity(nuclide, initial_parent_activity);

        /// Decide whether to include the 511 keV photons from positron annihilations
        /// included with the gammas.  If you want to get the the 511 keVs separately
        /// call NuclideMixture::betaPlusses(timeToAge, ordering).
        const INCLUDE_ANNIHILLATIONS: bool = true;

        // Decide how you want results ordered, either OrderByEnergy, or OrderByAbundance
        const ORDERING: HowToOrder = HowToOrder::OrderByEnergy;

        //Get the number of gammas you would expect to see, per second, at the desired age
        let mut tmp = MaybeUninit::uninit();
        let gammas = mixture.gammas_local(&mut tmp, time_to_age, ORDERING, INCLUDE_ANNIHILLATIONS);

        //Get the xrays produced by the decays that you would expect, per second, at the desired age
        let mut tmp = MaybeUninit::uninit();
        let xrays = mixture.xrays_local(&mut tmp, time_to_age, ORDERING);

        // If you just care about what your radiation detector might see, you could
        // just call NuclideMixture::photons(timeToAge,ordering) to get all of the
        // x-rays, gammas, and annihilation gammas mixed together
        // let photons = mixture.photons(time_to_age, ordering).collect::<Vec<_>>();

        println!(
            "After aging {:.6} years, {nuclide} will produce:",
            time_to_age / year
        );
        for EnergyRatePair { energy, num_per_second } in &xrays {
            println!("\t- xray {energy:.3} keV: {num_per_second:.2e}/second");
        }
        for EnergyRatePair { energy, num_per_second } in &gammas {
            println!("\t- gamma {energy:.3} keV: {num_per_second:.2e}/second");
        }

        anyhow::Ok(())
    }.context("printing the photon (xrays+gammas) energies and rates produced by a nuclide with the given initial activity, after it ages for the specified time")?;

    try_block!{
        let parent_age = 20.0 * year;
        let current_parent_activity = 1.0E6 * Bq;

        ensure!(parent_age > 0.0);
        ensure!(current_parent_activity > 0.0);

        let nuclide = database.try_nuclide(nuclide![U-238]).context("getting nuclide from database")?;

        let mut tmp = MaybeUninit::uninit();
        let mut mixture = LocalMixture::new_in(&mut tmp);
        mixture.add_aged_nuclide_by_activity(nuclide, current_parent_activity, parent_age).context("adding age nuclide to the mixture")?;

        const TIME_TO_AGE: f64 = 0.0;
        const ORDERING: HowToOrder = HowToOrder::OrderByEnergy;

        println!("{mixture:#?}");
        let mut tmp = MaybeUninit::uninit();
        let photons = mixture.photons_local(&mut tmp, TIME_TO_AGE, ORDERING);

        println!(
            "{} Bq of {nuclide} that is {} years old will produce the following photons:",
            current_parent_activity / Bq,
            parent_age / year
        );
        const MAX_ENERGIES: usize = 36;
        for EnergyRatePair { energy, num_per_second } in photons.iter().take(MAX_ENERGIES) {
            println!("\t- {energy:.3} keV: {num_per_second:.3e}/second");
        }
        if photons.len() > MAX_ENERGIES {
            println!("... skipping {} energies", photons.len() - MAX_ENERGIES);
        }

        anyhow::Ok(())
    }.context("printing the photon (xrays+gammas) energies and rates of a aged nuclide based on the parent isotopes current activity")?;

    try_block! {
        let nuc1_initial_age = 1.0 * year;
        let nuc2_initial_age = 0.0 * second;

        let nuc1_initial_activity = 1.0E-6 * Ci;
        let nuc2_initial_activity = 1.0E-4 * Ci;

        let time_to_age_mixture = 0.5 * year;

        ensure!(nuc1_initial_age >= 0.0);
        ensure!(nuc2_initial_age >= 0.0);
        ensure!(time_to_age_mixture >= 0.0);

        let nuc1 = database.try_nuclide(nuclide![Co-60]).context("getting nuclide 1 from database")?;
        let nuc2 = database.try_nuclide(nuclide![Ba-133]).context("getting nuclide 2 from database")?;

        let mut tmp = MaybeUninit::uninit();
        let mut mixture = LocalMixture::new_in(&mut tmp);
        mixture.add_aged_nuclide_by_activity(nuc1, nuc1_initial_activity, nuc1_initial_age).context("adding aged nuclide to the mixture")?;
        mixture.add_aged_nuclide_by_activity(nuc2, nuc2_initial_activity, nuc2_initial_age).context("adding aged nuclide to the mixture")?;

        const ORDERING: HowToOrder = HowToOrder::OrderByEnergy;
        let mut tmp = MaybeUninit::uninit();
        let photons = mixture.photons_local(&mut tmp, time_to_age_mixture, ORDERING);

        println!(
            concat!(
                "After {:.2e} seconds mixture\n",
                "- {} (initially {:.2e} seconds old with activity {:.3e} curie)\n",
                "- {} (initially {:.2e} seconds old with activity {:.3e} curie)\n",
                "and their descendants will produce following photons (gammas+xrays):"
            ),
            time_to_age_mixture / second,
            nuc1,
            nuc1_initial_age / second,
            nuc1_initial_activity / curie,
            nuc2,
            nuc2_initial_age / second,
            nuc2_initial_activity / curie,
        );

        for EnergyRatePair { energy, num_per_second } in &photons {
            println!("\t- {energy:.3} keV: {num_per_second:.3e}/second");
        }

        anyhow::Ok(())
    }
    .context("aging a mixture of nuclides")?;

    try_block! {
        let nuclide = database.try_nuclide(nuclide![U-238]).context("getting nuclide from database")?;

        println!("{nuclide} decays through (% of time):");
        let mut tmp = MaybeUninit::uninit();
        let descendants = nuclide.descendants_local(&mut tmp);
        for nuc in &descendants {
            println!("- {nuc:<6} ({:>6.2} %)", nuclide.branching_ratio_to_descendant(nuc)*100.0);
        }

        anyhow::Ok(())
    }
    .context("printing the decay chain and branching ratios of a nuclide")?;

    try_block! {
        let nuclide = database.try_nuclide(nuclide![Co-60]).context("getting nuclide from database")?;

        println!("{nuclide} has concident gammas (does not include descendant nuclides):");

        for &trans in &*nuclide.decays_to_children {
            for particle in &*trans.products {
                if particle.coincidences.is_empty() {
                    continue;
                }

                println!(
                    "\t{:.2} keV {} (br={:.2e}) coincident with:",
                    particle.energy,
                    particle.r#type,
                    particle.intensity,
                );
                for &CoincidencePair(part_ind, fraction) in &particle.coincidences {
                    let coinc_part = &trans.products[part_ind as usize];
                    println!(
                        "\t\t{} keV {:.2} at {:.2e}",
                        coinc_part.energy,
                        coinc_part.r#type,
                        fraction
                    );
                }
            }
        }
        println!();

        anyhow::Ok(())
    }
    .context("printing the coincident gammas of a nuclide")?;

    try_block!{
        let nuclide = database.try_nuclide(nuclide![Co-60]).context("getting nuclide from database")?;

        let decays_to_children = &nuclide.decays_to_children;
        let decays_from_parents = &nuclide.decay_from_parents;

        println!("{} Atomic Number {}, Atomic Mass {}, Isomer Number {} {} AMU, HalfLife={} seconds", nuclide.symbol, nuclide.atomic_number, nuclide.mass_number, nuclide.isomer_number, nuclide.atomic_mass, nuclide.half_life/second);

        let n_parents = decays_from_parents.len();
        match n_parents {
            0 => print!("Parents: none"),
            1 => println!("Parent: {}", decays_from_parents[0].parent.symbol),
            _ => {println!("Parents: {}", decays_from_parents[0].parent.symbol);
                for trans in &decays_from_parents[1..] {
                    print!(", {}", trans.parent.symbol);
                }
                println!();
            },
        }
        for trans in decays_to_children {
            println!("- {}", trans.human_str_summary());
        }

        // Instead of doing all the above you could have just called:
        // println!("{}", nuclide.human_str_summary());

        anyhow::Ok(())
    }.context("printing some basic information about a nuclide")?;

    try_block!{
        let lower_energy = 1332.48;
        let upper_energy = 1332.50;
        let allow_aging = false;

        ensure!(upper_energy > lower_energy);

        let all_nuclides = database.nuclides();

        let nuclides = nuclides_with_gamma_in_range(lower_energy, upper_energy, all_nuclides, allow_aging);

        #[expect(clippy::cast_possible_truncation)]
        {print!("Nuclides with gammas in the range {} keV to {} keV:\n\t", lower_energy / keV as f32, upper_energy / keV as f32)};
        if !nuclides.is_empty(){
            print!(": {}", nuclides[0].symbol);
            for nuc in &nuclides[1..] {
                print!(", {}", nuc.symbol);
            }
        }
        println!();

        anyhow::Ok(())
    }.context("finding nuclides with gammas in a specified energy range")?;

    try_block!{

        println!("Will demonstrate correcting for a nuclides decay during a measurement.");

        // In110 example
        {
            let nuclide = database.try_nuclide(nuclide![In-110]).context("getting nuclide from database")?; // (hl=4.9h), over a 2.8 hour measurement,
            let activity_at_meas_start = 1.0 * curie;

            let mut tmp = MaybeUninit::uninit();
            let mut mix = LocalMixture::new_in(&mut tmp);
            mix.add_nuclide_by_activity(nuclide, activity_at_meas_start);

            let initial_age=  0.0;
            let measurement_duration = 2.8 * hour;
            let num_timeslices = 500;
            let mut tmp = MaybeUninit::uninit();
            let photons = mix.decay_photons_in_interval_local(&mut tmp, initial_age, measurement_duration, HowToOrder::OrderByEnergy, num_timeslices);

            println!("During a {} hour measurement of {} with activity at the start of measurement of {} Ci, the number of photons emitted will be:", measurement_duration / hour, nuclide.symbol, activity_at_meas_start / curie);

            for EnergyCountPair { energy, count } in &photons {
                println!("\t- {energy:>8.3} keV: {count:.5e}");
          }
        }


        // Mn56 example
        {
            let nuclide = database.try_nuclide(nuclide![Mn-56]).context("getting nuclide from database")?; // (hl=9283.8s)
            let activity_at_meas_start = 1.0 * curie;

            let mut tmp = MaybeUninit::uninit();
            let mut mix = LocalMixture::new_in(&mut tmp);
            mix.add_nuclide_by_activity(nuclide, activity_at_meas_start);

            let initial_age = 0.0;
            let measurement_duration = 86423.86*second;
            let num_timeslices = 500;
            let mut tmp = MaybeUninit::uninit();
            let _photons = mix.decay_photons_in_interval_local(&mut tmp, initial_age, measurement_duration, HowToOrder::OrderByEnergy, num_timeslices);
        }

        // begin U235 example
        {
            let nuclide = database.try_nuclide(nuclide![U-235]).context("getting nuclide from database")?;
            let activity_at_meas_start = 1.0 * curie;

            let mut tmp = MaybeUninit::uninit();
            let mut mix = LocalMixture::new_in(&mut tmp);
            mix.add_nuclide_by_activity(nuclide, activity_at_meas_start);

            let initial_age = 10.0 * year;
            let measurement_duration = 1.0 * year;
            let num_timeslices = 500;
            let mut tmp = MaybeUninit::uninit();
            let _photons = mix.decay_photons_in_interval_local(&mut tmp, initial_age, measurement_duration, HowToOrder::OrderByEnergy, num_timeslices);
        }

        // begin Tc99 example
        {
            let nuclide = database.try_nuclide(nuclide![Tc-99]).context("getting nuclide from database")?;
            let activity_at_meas_start = 1.0 * curie;

            let mut tmp = MaybeUninit::uninit();
            let mut mix = LocalMixture::new_in(&mut tmp);
            mix.add_nuclide_by_activity(nuclide, activity_at_meas_start);

            let initial_age = 3600.0 * second;
            let measurement_duration = 1.0 * year;
            let num_timeslices = 500;
            let mut tmp = MaybeUninit::uninit();
            let _photons = mix.decay_photons_in_interval_local(&mut tmp, initial_age, measurement_duration, HowToOrder::OrderByEnergy, num_timeslices);
        }

        // begin I125 example
        {
            let nuclide = database.try_nuclide(nuclide![I-125]).context("getting nuclide from database")?;
            let activity_at_meas_start = 1.0 * curie;

            let mut tmp = MaybeUninit::uninit();
            let mut mix = LocalMixture::new_in(&mut tmp);
            mix.add_nuclide_by_activity(nuclide, activity_at_meas_start);

            let initial_age = 0.0;
            let measurement_duration = 14.0 * 24.0 * 3600.0 * year;
            let num_timeslices = 500;
            let mut tmp = MaybeUninit::uninit();
            let _photons = mix.decay_photons_in_interval_local(&mut tmp, initial_age, measurement_duration, HowToOrder::OrderByEnergy, num_timeslices);
        }

        anyhow::Ok(())
    }.context("example_correct_for_decays_during_measurements")?;
    try_block!{

        println!("Demonstrating calculating resulting nuclides from buildup during neutron activation.");

        // Define how long the sample will be irradiated
        let irradiation_time_seconds = 7.0 * 24.0 * 3600.0; //i.e., 1-week

        // Define how long after irradiation it will be before a measurement is started
        let cool_off_time = 2.0*3600.0; //i.e., 2 hours

        // Define how many seconds each time-step should be.
        //  A smaller time step should be more accurate, but you should take into account the
        //  half-lives of the nuclides you care about are.  For example, below Cr51 (HL=27.7d)
        //  dictates the time delta we really care about.  Ni59 is too long (practically doesnt
        //  decay on our relevant time-scale), and Fe55 is too short (doesnt build-up on our
        //  timescale), but for Cr51 we will start seeing some small numerical accuracy issues
        //  if we make the time delta more than a few hours (but the computation is pretty fast,
        //  so we'll just use a minute).
        let time_delta = 60.0;  //i.e., 60 seconds.


        /*
        Define nuclides that will be built up, and their buildup rate, in atoms per second

        We'll only add a few nuclides being produced here, but you can add as many
         as you'de like.
         The below indicated our experiment is expected to generate 63.8 Ni59 atoms,
         per second, from neutron transmutation; if there are any decays to Ni59 from
         other activation products, we dont need to account for them here, they will
         taken care of accumulating during decay calculations.
        */
        let nuclides_rates = vec![
            (database.try_nuclide(nuclide![Ni-59]).context("getting nuclide from database")?, 63.8),  // HL=7.6E4 years
            (database.try_nuclide(nuclide![Cr-51]).context("getting nuclide from database")?, 29.1),  // HL=27.7 days
            (database.try_nuclide(nuclide![Fe-55]).context("getting nuclide from database")?, 24.0),  // HL=0.15s
        ];

        // We will step through the build-up time frame, with what we want to know
        // is how many, of each type of radioactive atom will be left at the end of
        // buildup.  So we will define a `NuclideMixture` to calculate this for us
        // at each time step.
        let mut tmp = MaybeUninit::uninit();
        let mut ager = LocalMixture::new_in(&mut tmp);

        // Add our nuclides to it
        for (nuclide, rate) in &nuclides_rates {
            let num_atoms = time_delta * rate;
            ager.add_nuclide_by_abundance( nuclide, num_atoms );
        }

        // Now we will crate a mixture that will represent, for its t=0, the end of
        //  build up, and after the looping over all the time steps, it will have
        //  all the information we want.
        let mut tmp = MaybeUninit::uninit();
        let mut mixture = LocalMixture::new_in(&mut tmp);

        // This integration is very naive, and could be greatly improved.
        //  However, informal checks show for irradiation time of months, and common
        //  neutron activation products in metals, accuracy and numeric error didn't
        //  become notable issues (checked smaller time deltas, as well as using 128 bit,
        //  instead of 64 bit internal precisions in SandiaDecay, as well as exact
        //  expectations).
        let mut start_time = 0.0;
        while start_time < irradiation_time_seconds{
            let end_time = pmin( start_time + time_delta, irradiation_time_seconds );
            let this_dt = end_time - start_time;
            let mid_time = 0.5*(start_time + end_time);
            let time_until_irad_end = irradiation_time_seconds - mid_time;

            // Get the number of atoms, for all activation products, and their progeny, we expect
            //  at the end of buildup time.
            let mut tmp = MaybeUninit::uninit();
            let num_atoms = ager.num_atoms_local(&mut tmp, time_until_irad_end);
            for NuclideNumAtomsPair { nuclide, num_atoms } in &num_atoms {
                mixture.add_nuclide_by_abundance( nuclide, (this_dt / time_delta)*num_atoms );
            }
            start_time += time_delta;
        }


        let mut tmp = MaybeUninit::uninit();
        let irrad_end_activities = mixture.activities_local(&mut tmp, 0.0);
        println!("At the end of irradiation, the activities are:");
        for NuclideActivityPair{nuclide, activity} in &irrad_end_activities {
            println!("\t{}: {activity:.6e} Bq", nuclide.symbol);
        }

        let mut tmp = MaybeUninit::uninit();
        let after_cool_off_activities = mixture.activities_local(&mut tmp, cool_off_time);
        println!("\n\nAfter cooling off for {} seconds, the activities are:", cool_off_time/second);
        for NuclideActivityPair { nuclide, activity } in &after_cool_off_activities {
            println!("\t {}: {activity:.6e} Bq", nuclide.symbol);
        }

        // We expect A = A_0 * (1 - exp(-lambda * t_activation), so lets check things, but
        // please note that this is only a valid check if no other activation products
        // decay through the activation nuclide of interest.
        print!("\n\n");
        for NuclideActivityPair { nuclide: output_nuc, activity: out_act } in &irrad_end_activities {
            for (input_nuc, input_rate) in &nuclides_rates {
            if input_nuc != output_nuc {
              continue;
                }

            let lambda = input_nuc.decay_constant();
            let expected_act = input_rate * (1.0 - f64::exp(-lambda * irradiation_time_seconds));
            println!("For {} analytically expected {expected_act:.5e} Bq; our calculation is {out_act:.5e} Bq", input_nuc.symbol);
          }
        }

        anyhow::Ok(())
    }.context("example_buildup_calculation")?;

    anyhow::Ok(())
}

fn nuclides_with_gamma_in_range<'l>(
    low_e: f32,
    high_e: f32,
    candidates: &[&'l Nuclide<'l>],
    allow_aging: bool,
) -> Vec<&'l Nuclide<'l>> {
    let mut answer = Vec::new();

    if allow_aging {
        for nuc in candidates {
            let mut tmp = MaybeUninit::uninit();
            let mut mix = LocalMixture::new_in(&mut tmp);
            mix.add_nuclide_by_activity(nuc, 1.0E6 * becquerel);
            let mut tmp = MaybeUninit::uninit();
            let gammas =
                mix.gammas_local(&mut tmp, nuc.half_life, HowToOrder::OrderByAbundance, true);

            for EnergyRatePair {
                energy,
                num_per_second: _,
            } in &*gammas
            {
                #[expect(clippy::cast_possible_truncation)]
                let energy = *energy as f32;
                if energy >= low_e && energy <= high_e && !answer.contains(nuc) {
                    answer.push(nuc);
                }
            }
        }
    } else {
        for nuc in candidates {
            let trans = &nuc.decays_to_children;

            for tran in trans {
                let particles = &tran.products;
                for particle in particles {
                    if particle.r#type != ProductType::GammaParticle {
                        continue;
                    }

                    if particle.energy >= low_e
                        && particle.energy <= high_e
                        && !answer.contains(nuc)
                    {
                        answer.push(nuc);
                    }
                }
            }
        }
    }

    answer
}
