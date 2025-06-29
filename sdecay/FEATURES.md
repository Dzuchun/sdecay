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
