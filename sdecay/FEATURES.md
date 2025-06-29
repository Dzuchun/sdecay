Description of feature flags for this crate

# `default`

By default, `std` feature is enabled

# `git`

Relayed to [`sdecay-sys`] crate

# `alloc`

Enables types provided by `alloc` crate. Note, that this does not imply that crate can actually be built for no-alloc environment, since `SandiaDecay` obviously uses a lot of dynamic allocations

## Notable types
- [`BoxContainer`](crate::container::BoxContainer)
- [`ArcContainer`](crate::container::ArcContainer)

## Notable functions
- usually, functions outputting C++ types with _no suffix_ correspond to [`BoxContainer`](crate::container::BoxContainer)
- usually, functions outputting C++ types with `shared` suffix correspond to [`ArcContainer`](crate::container::ArcContainer)

# `std`

<section class="info">
Includes <code>alloc</code> feature
</section>

Allows database initialization by path from the environment

## Notable types
(none)

## Notable functions
- [`GenericUninitDatabase::init_env`]
- [`GenericDatabase::from_env`].

# `embed`/`embed-min`/`embed-nocoinc-min`

Embeds corresponding database that comes with repository directly into the binary. This is a _significant_ bloat, so consider using it only if you truly need a standalone binary, or employing quick prototyping.

## Notable types
(none)

## Notable functions
- [`GenericUninitDatabase::init_embedded`]
- [`GenericUninitDatabase::init_embedded_min`]
- [`GenericUninitDatabase::init_embedded_nocoinc_min`]
- [`GenericDatabase::embedded`]
- [`GenericDatabase::embedded_min`]
- [`GenericDatabase::embedded_nocoinc_min`]
