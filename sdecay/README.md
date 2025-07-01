A Rust interface for [SandiaDecay] C++ library, used to calculate nuclear decays and emissions.

# Disclaimer

This crate is not coordinated nor endorsed by [SandiaDecay]. This is a completely separate effort.

# This library can

- Query [`Element`](crate::wrapper::Element) information:
    - Identification (symbol, name)
    - Naturally occurring isotopes and their abundances
    - List of flouresence x-rays (if database happen to contain them)
- Query [`Nuclide`](crate::wrapper::Nuclide`) information:
    - Identification (symbol, mass number $A$)
    - Mass
    - Half-life $T_{1/2}$
    - Children [`Transition`](crate::wrapper::Transition)s (i.e. decay branches)
    - Parents [`Transition`](crate::wrapper::Transition)s (i.e. creating branches)
- Predict [nuclide mixture](crate::nuclide_mixture::GenericMixture) evolution:
    - [nuclide activities](crate::wrapper::NuclideMixture::activities_in) at time $t$
    - [energies and intensities](crate::wrapper::NuclideMixture::decay_particle) of various [`ProductTypes`](crate::wrapper::ProductType)s (notably, [$\gamma$](crate::wrapper::NuclideMixture::gammas_in) and [x-ray](crate::wrapper::NuclideMixture::xrays_in)) at the time $t$ 
    - [energies and counts](crate::wrapper::NuclideMixture::decay_particles_in_interval) of various [`ProductTypes`](crate::wrapper::ProductType)s in the time interval $[t; t + l]$
    - [evolution equation](crate::wrapper::NuclideMixture::decayed_to_nuclides_evolutions) (functions describing abundance of each nuclide over time)

# Build

This crate has multiple build options. See detailed instructions at [`building`].

After choosing your option, add
```toml
# Cargo.toml
sdecay = "0.1"
```

# Safety

As an FFI crate, safety is a big concern. See [`safety`] for notes on safety design.

# Walkthrough

## Database file

[SandiaDecay] uses it's own database xml-based database format, and I'm not really sure how you'd go about creating it yourself. Library's repository provides several versions, so you need to pick one before using the crate.

## Constructing a database

[Database file](#database-file) is required in some form to construct a `Database`. There are several options to go about this:

- Download database yourself, and
    - obtain database file bytes[^1] [^2], and use [`from_bytes`](crate::database::GenericDatabase::from_bytes_in)-like constructor
    - get a path to database file, and use [`from_path`](crate::database::GenericDatabase::from_path_in)-like constructor
    - store a path to database file in the `SANDIA_DATABASE_PATH` environment variable, and use [`from_env`](crate::database::GenericDatabase::from_env_in)-like constructor
- Enable one of `database*` features and construct it directly via corresponding method. Note, that this approach will download the database from GitHub before compilation, and embed it into your binary, so expect for build to take some time.

[^1]: If you wish to embed database file into your binary, see [`macro@include_bytes`]
[^2]: Unfortunately, separate allocation has to be performed on C++ side, as [SandiaDecay]'s interface expects `std::vector<char> &`

For example, assuming you have a database (or a symlink to it) in your current directory named `sandia.decay.xml`, here is how you would construct it:
```rust,no_run
# #[cfg(feature = "alloc")] {
# use sdecay::Database;
let database = Database::from_path("sandia.decay.xml").unwrap();
# }
```

## Querying for info

### Element

See [`element`](crate::wrapper::SandiaDecayDataBase::element) function

[`Element`](crate::wrapper::Element) is queried for with [`ElementSpec`](crate::element_spec::ElementSpec) implementors, notably:
- by name as `&`[`str`] \(or other text-related type, like `&`[`CStr`](core::ffi::CStr), `&`[`OsStr`](std::ffi::OsStr)\)
- atomic number (i.e. number of protons) as `i32` (or smaller integer, like `i16`, `u16`, etc)
- atomic number, but constructed by [`element`] macro:
```rust
# use sdecay::element;
element!(H);
element!(k);
element!(Pu);
element!(mo);
```

Database returns [`Element`](crate::wrapper::Element) structure, containing all of the element info:
- Symbol
- Name
- Naturally occurring isotopes and their abundances (as a list of [`NuclideAbundancePair`](crate::wrapper::NuclideAbundancePair))
- List of flouresence x-rays (as a list of [`EnergyIntensityPair`](crate::wrapper::EnergyIntensityPair))

Code pretty-printing `isotopes` and `xrays` fields can be found in the `doc-example`. Here's their output for `W` (Tungsten):
```text
 0.130% of W180
26.300% of W182
14.300% of W183
30.670% of W184
28.600% of W186
```
```text
58.0 keV (0.578 relative intensity)
59.3 keV (1.000 relative intensity)
67.2 keV (0.338 relative intensity)
69.1 keV (0.086 relative intensity)
```

### Nuclide

See [`nuclide`](crate::wrapper::SandiaDecayDataBase::nuclide) function

[`Nuclide`](crate::wrapper::Nuclide) is queried for with [`NuclideSpec`](crate::nuclide_spec::NuclideSpec) implementors, notably:
- by symbol as `&`[`str`] \(or other text-related type, like `&`[`CStr`](core::ffi::CStr), `&`[`OsStr`](std::ffi::OsStr)\)
- [`NumSpec`](crate::nuclide_spec::NumSpec) (i.e. numeric identification on nuclide)
- [`NumSpec`](crate::nuclide_spec::NumSpec), but constructed by [`nuclide`] macro:
```rust
# use sdecay::nuclide;
nuclide!(H-2);
nuclide!(k-40);
nuclide!(Pu-239);
nuclide!(mo-95);
```

Database returns [`Nuclide`](crate::wrapper::Nuclide) structure, containing all of the nuclide info:
- Symbol
- Mass number $A$
- Mass (in a.m.u.)
- Half-life $T_{1/2}$
- Children [`Transition`](crate::wrapper::Transition)s
- Parent [`Transition`](crate::wrapper::Transition)s

Code for pretty-printing `decays_to_children` and `decay_from_parents` fields can be found in the `doc-example`. Here's their output for ${}^{247}\text{Es}$:
```text
> 0.50: \text{Fm247m}(EC) -> \text{Es247} + \nu
> 0.10: \text{Md251} -> \text{Es247} + \alpha
| 1.00 of AlphaParticle at 7550.00 keV
> 0.50: \text{Fm247}(EC) -> \text{Es247} + \nu
< 0.93: \text{Es247}(EC) -> \text{Cf247} + \nu
< 0.07: \text{Es247} -> \text{Bk243m} + \alpha
| 0.02 of AlphaParticle at 7213.00 keV
| 0.12 of AlphaParticle at 7275.00 keV
| 0.86 of AlphaParticle at 7323.00 keV
< 0.07: \text{Es247} -> \text{Bk243} + \alpha
| 0.02 of AlphaParticle at 7213.00 keV
| 0.12 of AlphaParticle at 7275.00 keV
| 0.86 of AlphaParticle at 7323.00 keV
```

## Constructing a mixture

Empty mixture can be constructed with [`new`/`new_in`](crate::nuclide_mixture::GenericMixture::new_in) functions:
```rust
# #[cfg(feature = "alloc")] {
# use sdecay::Mixture;
let mixture = Mixture::new();
# }
```

Next, mixture should be populated with nuclides by calling [`add_nuclide`](crate::nuclide_mixture::GenericMixture::add_nuclide) (alternatively, [`add_aged_nuclide_by_activity`](crate::nuclide_mixture::GenericMixture::add_aged_nuclide_by_activity) or [`add_nuclide_by_abundance`](crate::nuclide_mixture::GenericMixture::add_nuclide_by_abundance)).

Let's add $1 \mu \text{Ci}$ of ${}^{40}\text{K}$ to it:
```rust
# #[cfg(feature = "alloc")] {
# use sdecay::{Database, Mixture, nuclide, wrapper::NuclideActivityPair, cst::Ci};
# const DATABASE_PATH: &str = env!("SANDIA_DATABASE_PATH");
# let database = Database::from_path(DATABASE_PATH).unwrap();
# let mut mixture = Mixture::new();
let k40 = database.nuclide(nuclide!(k - 40));
assert!(mixture.add_nuclide(NuclideActivityPair {
    nuclide: k40,
    activity: 1e-6 * Ci
}));
# }
```
If debug-printed now, mixture would show the following:
```text
GenericMixture(BoxContainer(NuclideMixture { K40: 3.70e4 Bq }))
```

There are other ways to add a nuclide to the mixture:
- nuclide can be pre-aged ([`add_aged_nuclide_by_activity`](crate::nuclide_mixture::GenericMixture::add_aged_nuclide_by_activity), [`add_aged_nuclide_by_num_atoms`](crate::nuclide_mixture::GenericMixture::add_aged_nuclide_by_num_atoms))
```rust
# #[cfg(feature = "alloc")] {
# use sdecay::{Database, Mixture, nuclide, wrapper::NuclideActivityPair, cst::{Ci, second}};
# const DATABASE_PATH: &str = env!("SANDIA_DATABASE_PATH");
# let database = Database::from_path(DATABASE_PATH).unwrap();
# let mut mixture = Mixture::new();
let rn220 = database.nuclide(nuclide!(Rn - 220));
mixture
    .add_aged_nuclide_by_activity(rn220, 1e-6 * Ci, 10.0 * second)
    .unwrap();
# }
```
```text
GenericMixture(BoxContainer(NuclideMixture { Tl208: 2.96e-5 Bq, Pb208: NaN Bq, Pb212: 6.99e0 Bq, Bi212: 6.66e-3 Bq, Po212: 4.
```
- nuclide can be added in a "secular equilibrium" ([`add_nuclide_in_secular_equilibrium`](crate::nuclide_mixture::GenericMixture::add_nuclide_in_secular_equilibrium))
```rust
# #[cfg(feature = "alloc")] {
# use sdecay::{Database, Mixture, nuclide, wrapper::NuclideActivityPair, cst::Ci};
# const DATABASE_PATH: &str = env!("SANDIA_DATABASE_PATH");
# let database = Database::from_path(DATABASE_PATH).unwrap();
# let mut mixture = Mixture::new();
let ar42 = database.nuclide(nuclide!(Ar - 42));
mixture
    .add_nuclide_in_secular_equilibrium(ar42, 1e-6 * Ci)
    .unwrap();
# }
```
```text
GenericMixture(BoxContainer(NuclideMixture { Ar42: 3.70e4 Bq }))
```
- nuclide can be added in a "prompt equilibrium" ([`add_nuclide_in_prompt_equilibrium`](crate::nuclide_mixture::GenericMixture::add_nuclide_in_prompt_equilibrium))
```rust
# #[cfg(feature = "alloc")] {
# use sdecay::{Database, Mixture, nuclide, wrapper::NuclideActivityPair, cst::Ci};
# const DATABASE_PATH: &str = env!("SANDIA_DATABASE_PATH");
# let database = Database::from_path(DATABASE_PATH).unwrap();
# let mut mixture = Mixture::new();
let u238 = database.nuclide(nuclide!(U - 238));
assert!(mixture.add_nuclide_in_prompt_equilibrium(u238, 1e-6 * Ci));
# }
```
```text
GenericMixture(BoxContainer(NuclideMixture { U238: 3.70e4 Bq, Th234: 3.70e4 Bq, Pa234m: 3.70e4 Bq }))
```

## Calculating mixture evolution

### Activities
See [`activities`](crate::wrapper::NuclideMixture::activities_in)
```rust
# #[cfg(feature = "alloc")] {
# use sdecay::{Database, Mixture, nuclide, wrapper::NuclideActivityPair, cst::{Ci, hour}};
# const DATABASE_PATH: &str = env!("SANDIA_DATABASE_PATH");
# let database = Database::from_path(DATABASE_PATH).unwrap();
# let mut mixture = Mixture::new();
let rn221 = database.nuclide(nuclide!(Rn - 221));
assert!(mixture.add_nuclide(NuclideActivityPair {
    nuclide: rn221,
    activity: 1e-6 * Ci
}));
for NuclideActivityPair { nuclide, activity } in mixture.activities(hour).iter() {
    println!("{:9.3e} Ci of {}", activity / Ci, nuclide.symbol);
}
# }
```
```text
  0.000e0 Ci of Tl205
 4.244e-9 Ci of Tl209
 2.675e-8 Ci of Pb209
4.127e-32 Ci of Bi209
 5.541e-8 Ci of Pb213
 2.030e-7 Ci of Bi213
 1.985e-7 Ci of Po213
 3.793e-8 Ci of Po217
 1.836e-7 Ci of At217
 1.895e-7 Ci of Rn221
 1.836e-7 Ci of Fr221
```

### Decay particles
See [`decay_particles`](crate::wrapper::NuclideMixture::decay_particle_in)

For $\gamma$:
```rust
# #[cfg(feature = "alloc")] {
# use sdecay::{Database, Mixture, nuclide, wrapper::{NuclideActivityPair, EnergyRatePair, HowToOrder, ProductType}, cst::{Ci, keV, hour}};
# const DATABASE_PATH: &str = env!("SANDIA_DATABASE_PATH");
# let database = Database::from_path(DATABASE_PATH).unwrap();
# let mut mixture = Mixture::new();
let ar42 = database.nuclide(nuclide!(Ar - 42));
assert!(mixture.add_nuclide(NuclideActivityPair {
    nuclide: ar42,
    activity: 1e-6 * Ci
}));
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
# }
```
```text
1.525e3 keV at 3.79e2/second
3.126e2 keV at 7.06e0/second
8.997e2 keV at 1.09e0/second
1.921e3 keV at 8.68e-1/second
1.021e3 keV at 4.24e-1/second
2.424e3 keV at 4.24e-1/second
6.920e2 keV at 6.57e-2/second
1.228e3 keV at 4.74e-2/second
```

For $e^{-}$:
```rust
# #[cfg(feature = "alloc")] {
# use sdecay::{Database, Mixture, nuclide, wrapper::{NuclideActivityPair, EnergyRatePair, HowToOrder, ProductType}, cst::{Ci, keV, hour}};
# const DATABASE_PATH: &str = env!("SANDIA_DATABASE_PATH");
# let database = Database::from_path(DATABASE_PATH).unwrap();
# let mut mixture = Mixture::new();
# let ar42 = database.nuclide(nuclide!(Ar - 42));
# assert!(mixture.add_nuclide(NuclideActivityPair {
#     nuclide: ar42,
#     activity: 1e-6 * Ci
# }));
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
# }
```
```text
5.990e2 keV at 3.70e4/second
3.525e3 keV at 1.65e3/second
2.001e3 keV at 3.56e2/second
1.688e3 keV at 6.86e0/second
7.851e1 keV at 1.41e0/second
1.101e3 keV at 1.01e0/second
```

### Decay particles in interval
See [`decay_particles_in_interval`](crate::wrapper::NuclideMixture::decay_particles_in_interval_in)

For $\tilde{\nu}$:
```rust
# #[cfg(feature = "alloc")] {
# use sdecay::{Database, Mixture, nuclide, wrapper::{NuclideActivityPair, EnergyCountPair, HowToOrder, ProductType}, cst::{Ci, keV, day}};
# const DATABASE_PATH: &str = env!("SANDIA_DATABASE_PATH");
# let database = Database::from_path(DATABASE_PATH).unwrap();
# let mut mixture = Mixture::new();
let c10 = database.nuclide(nuclide!(c - 9));
assert!(mixture.add_nuclide(NuclideActivityPair {
    nuclide: c10,
    activity: 1e-6 * Ci
}));
for EnergyCountPair { energy, count } in &mixture.decay_particles_in_interval(
    0.0,
    day,
    ProductType::CaptureElectronParticle,
    HowToOrder::OrderByEnergy,
    1000,
) {
    println!("{:7.3e} keV at {:.2e}", energy / keV, count);
}
# }
```
```text
1.840e3 keV at 2.41e-52
2.485e3 keV at 6.35e-52
4.335e3 keV at 2.09e-51
1.371e4 keV at 3.87e-53
1.415e4 keV at 1.84e-52
1.649e4 keV at 1.97e-52
```

For $e^{+}$:
```rust
# #[cfg(feature = "alloc")] {
# use sdecay::{Database, Mixture, nuclide, wrapper::{NuclideActivityPair, EnergyCountPair, HowToOrder, ProductType}, cst::{Ci, keV, day}};
# const DATABASE_PATH: &str = env!("SANDIA_DATABASE_PATH");
# let database = Database::from_path(DATABASE_PATH).unwrap();
# let mut mixture = Mixture::new();
# let c10 = database.nuclide(nuclide!(c - 9));
# assert!(mixture.add_nuclide(NuclideActivityPair {
#     nuclide: c10,
#     activity: 1e-6 * Ci
# }));
for EnergyCountPair { energy, count } in &mixture.decay_particles_in_interval(
    0.0,
    day,
    ProductType::PositronParticle,
    HowToOrder::OrderByEnergy,
    1000,
) {
    println!("{:7.3e} keV at {:.2e}", energy / keV, count);
}
# }
```
```text
8.178e2 keV at 6.35e-50
1.463e3 keV at 1.02e-48
3.313e3 keV at 3.74e-47
1.269e4 keV at 3.68e-47
1.313e4 keV at 1.93e-46
1.547e4 keV at 3.43e-46
```

### Reading equations
See [`decayed_to_nuclides_evolutions`](crate::wrapper::NuclideMixture::decayed_to_nuclides_evolutions)

Returned vector contains [`NuclideTimeEvolution`](crate::wrapper::NuclideTimeEvolution)s, each describing evolution of a [`Nuclide`](crate::wrapper::Nuclide). Evolution is described as a vector of additive terms, each being of form $\text{term\\_coeff} \cdot \exp(- \text{exponential\\_coeff} \cdot \text{t})$

`doc-example` contains a bunch of code to pretty-print terms for 1$\mu \text{Ci}$ of ${}^{24}\text{Ne}$:
```text
N(Ne24, t) = 1.083e7 * exp(+3.418e-3 * t) + 1.083e7 * exp(+3.418e-3 * t)
N(Na24, t) = -1.086e7 * exp(+3.418e-3 * t) + -1.086e7 * exp(+3.418e-3 * t) + 1.086e7 * exp(+1.284e-5 * t) + 1.077e3 * exp(+3.435e1 * t)
N(Na24m, t) = 1.077e3 * exp(+3.418e-3 * t) + 1.077e3 * exp(+3.418e-3 * t) + -1.077e3 * exp(+3.435e1 * t)
N(Mg24, t) = 3.539e4 * exp(+3.418e-3 * t) + 3.539e4 * exp(+3.418e-3 * t) + -1.086e7 * exp(+1.284e-5 * t) + 5.383e-1 * exp(+3.435e1 * t) + 1.083e7 * exp(+0.000e0 * t)
```

[SandiaDecay]: <https://github.com/sandialabs/SandiaDecay>
