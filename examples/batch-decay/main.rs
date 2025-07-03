//! This is a recreation of `batch_decay` example from the original repo
#![allow(clippy::too_many_lines, clippy::items_after_statements, missing_docs)]

use std::{
    ffi::CString,
    fs::OpenOptions,
    io::{BufRead, BufReader, BufWriter, Write, stdout},
    mem::MaybeUninit,
    path::PathBuf,
};

use anyhow::{Context, bail, ensure};
use clap::Parser;

use sdecay::{
    LocalDatabase, LocalMixture,
    cst::{curie, second},
    wrapper::Nuclide,
};

macro_rules! try_block {
    { $($token:tt)* } => {{
        (|| {
            $($token)*
        })()
    }}
}

fn parse_time(s: &str) -> anyhow::Result<f64> {
    let Some((end, last_ch)) = s.char_indices().last() else {
        bail!("empty input");
    };
    let mul = match last_ch {
        's' => 1.0,
        'm' => 60.0,
        'h' => 60.0 * 60.0,
        'd' => 24.0 * 60.0 * 60.0,
        'y' => 365.2425 * 24.0 * 60.0 * 60.0,
        _ => bail!("unknown time suffix: {last_ch}"),
    };
    let val = s[..end].parse::<f64>().context("parsing value")?;
    Ok(val * mul)
}

#[derive(Debug, Parser)]
struct Args {
    #[arg(long("decay-data"), default_value = "sandia.decay.xml")]
    sandia_decay_xml: CString,
    #[arg(long("input"))]
    input_csv: Option<PathBuf>,
    #[arg(long("output"))]
    output_csv: Option<PathBuf>,
    #[arg(long("time"), value_parser = parse_time)]
    time: f64,
    #[arg(long("steps"), default_value_t = 1)]
    steps: u32,
    #[arg(long("mix-input"))]
    mix_input: bool,
    #[arg(long("show-children"))]
    show_children: bool,
    #[arg(long("print-header"))]
    print_header: bool,
    #[arg(long("nuclide"), long("isotope"), long("iso"))]
    nuclides: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
struct NuclideInput<'db> {
    nuclide: &'db Nuclide<'db>,
    // is nonzero
    start_activity: f64,
}

fn parse_nuclide_activity<'db>(
    s: &str,
    database: &'db LocalDatabase<'_>,
) -> anyhow::Result<NuclideInput<'db>> {
    let (nuclide_name, start_activity) = if let Some(comma) = s.find(',') {
        (
            &s[..comma],
            s[(comma + 1)..]
                .parse::<f64>()
                .context("parsing activity")?,
        )
    } else {
        (s, 1.0)
    };

    ensure!(start_activity != 0.0, "activity should not be 0");

    let nuclide = database
        .try_nuclide(nuclide_name)
        .context("finding nuclide")?;
    Ok(NuclideInput {
        nuclide,
        start_activity,
    })
}

fn main() -> anyhow::Result<()> {
    let args = Args::try_parse().context("parsing clargs")?;
    println!("{args:#?}");

    let mut tmp = MaybeUninit::uninit();
    let database = LocalDatabase::from_path_in(&mut tmp, args.sandia_decay_xml)
        .context("initializing sandia database")?;

    let mut inputs = args
        .nuclides
        .into_iter()
        .enumerate()
        .try_fold(Vec::new(), |mut acc, (no, next)| {
            acc.push(
                parse_nuclide_activity(&next, &database)
                    .with_context(|| format!("parsing nuclide {no}: {next}"))?,
            );
            anyhow::Ok(acc)
        })
        .context("parsing input lines")?;

    ensure!(
        !inputs.is_empty() || args.input_csv.is_some(),
        "No input nuclides or input CSV file specified"
    );

    ensure!(args.steps > 0); // ?

    if let Some(input) = args.input_csv {
        let mut input = BufReader::with_capacity(
            1 << 20,
            OpenOptions::new()
                .read(true)
                .open(input)
                .context("opening file")?,
        );
        let mut line = String::new();
        while {
            line.clear();
            input.read_line(&mut line).context("reading input line")? != 0
        } {
            let line = line.trim();
            try_block! {
                if line.len() < 2 {
                    return anyhow::Ok(());
                }

                if line.starts_with('#') {
                    return anyhow::Ok(());
                }

                let Some(delim_pos) = line.find([',', '\t']) else {
                    bail!("Could not find a comma or tab");
                };

                if line[(delim_pos + 1)..].find([',', '\t']).is_some() {
                    bail!("Line has more than two columns");
                }

                let nuc_str = line[..delim_pos].trim();
                let act_str = line[(delim_pos + 1)..].trim();

                let nuclide = database.try_nuclide(nuc_str).context("getting nuclide from database")?;

                let start_activity = act_str.parse::<f64>().context("parsing activity")?;

                inputs.push(NuclideInput {
                    nuclide,
                    start_activity,
                });
                anyhow::Ok(())
            }
            .with_context(|| format!("parsing line \"{line}\""))?;
        }
    }

    let output: Box<dyn Write> = if let Some(output) = args.output_csv {
        Box::new(
            OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(output)
                .context("opening output file")?,
        ) as _
    } else {
        Box::new(stdout()) as _
    };
    let mut output = BufWriter::with_capacity(1 << 20, output);

    let mut tmp = MaybeUninit::uninit();
    let mut sum_mix = LocalMixture::new_in(&mut tmp);

    if args.print_header {
        if args.mix_input || !args.show_children {
            output.write_all(b"Nuclide")?;
            for step in 0..args.steps {
                let t = if args.steps == 1 {
                    args.time
                } else {
                    f64::from(step) * args.time / f64::from(args.steps - 1)
                };
                write!(output, ",Act {t:.0} seconds")?;
            }
            output.write_all(b"\n")?;
        } else {
            ensure!(args.show_children);
            output.write_all(b"ParentNuclide,ProgenyNuclide")?;
            for step in 0..args.steps {
                let t = if args.steps == 1 {
                    args.time
                } else {
                    f64::from(step) * args.time / f64::from(args.steps - 1)
                };
                write!(output, ",Act {t:.0} seconds")?;
            }
            output.write_all(b"\n")?;
        }
    }

    println!("{inputs:#?}");

    for input in inputs {
        // if( !input.nuclide )
        // {
        //   cerr << "Warning: " << input.nuc_str << " does not appear to be a valid nuclide" << endl;
        //   if( !mix_input )
        //     output << input.nuc_str << ",InvalidNuclide" << endl;
        //   continue;
        // }

        if input.start_activity == 0.0 {
            eprintln!(
                "Warning: zero initial activity for {}",
                input.nuclide.symbol,
            );
        }

        // if args.mix_input {
        //     writeln!(
        //         output,
        //         "{},{}",
        //         input.nuclide.symbol_str(),
        //         input.start_activity
        //     )?;
        //     continue;
        // }

        if input.nuclide.is_stable() && !args.mix_input {
            writeln!(output, "{},Stable", input.nuclide.symbol)?;
            continue;
        }

        if args.mix_input {
            sum_mix.add_nuclide_by_activity(input.nuclide, input.start_activity * curie);
            continue;
        }

        const FAKE_ACTIVITY: f64 = 0.01 * curie;
        let scale: f64 = input.start_activity / FAKE_ACTIVITY;

        let mut tmp = MaybeUninit::uninit();
        let mut mix = LocalMixture::new_in(&mut tmp);
        ensure!(
            mix.add_nuclide_by_activity(input.nuclide, FAKE_ACTIVITY),
            "adding nuclide with fake activity to database"
        );
        dbg!(&mix);

        if args.show_children {
            output.write_all(input.nuclide.symbol.as_bytes())?;
            for nuc in mix.solution_nuclides() {
                write!(output, ",{}", nuc.symbol)?;
                for step in 0..args.steps {
                    let t = if args.steps == 1 {
                        args.time
                    } else {
                        f64::from(step) * args.time / f64::from(args.steps - 1)
                    };
                    let fake_decayed_act = mix.nuclide_activity(t, nuc).unwrap();
                    write!(output, ",{:.10}", fake_decayed_act * scale)?;
                }
                output.write_all(b"\n")?;
            }
        } else {
            output.write_all(input.nuclide.symbol.as_bytes())?;
            for step in 0..args.steps {
                //We could actually do the simple thing of
                //const double expCoeff = input.nuclide->decayConstant();
                //const double decayact = input.start_activity * exp( -decay_time * expCoeff );
                //cout << input.nuclide->symbol << endl;
                //output << input.nuc_str << "," << decayact << endl;

                let t = if args.steps == 1 {
                    args.time
                } else {
                    f64::from(step) * args.time / f64::from(args.steps - 1)
                };
                let act = mix.nuclide_activity(t * second, input.nuclide).unwrap();
                write!(output, ",{act:.10}")?;
            } //for( size_t step = 0; step < numhistories; ++step )

            output.write_all(b"\n")?;
        }
    }

    dbg!(&sum_mix);

    if args.mix_input {
        for nuc in sum_mix.solution_nuclides() {
            output.write_all(nuc.symbol.as_bytes())?;
            for step in 0..=args.steps {
                let t = if args.steps == 1 {
                    args.time
                } else {
                    f64::from(step) * args.time / f64::from(args.steps - 1)
                };
                let act = sum_mix.nuclide_activity(t, nuc).unwrap();
                write!(output, ",{:.10}", act / curie)?;
            }
            output.write_all(b"\n")?;
        }
    }

    Ok(())
}
