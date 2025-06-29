Project consists of 3 crates:
- `nolt` (no-lifetime) - a tiny helper proc macro
- `sdecay-sys`, exposing C++ functions and structs directly (inherently unsafe)
- `sdecay` exposing _safe representations_ of C++ structures

# No cxx?

[cxx] is an awesome crate, aiming to provide safe Rust-C++ intertop. While being an amazing project, I felt really limited by it's proposed workflow. Sure, it is perfectly justified - C++ semantics is quite complex, but I didn't feel like completely opaque structures with no member access are justified because of that.

This crate does not use [cxx], but it introduces similar concepts aiming to solve same issues.

# No bindgen??

At the time, bindings are generated manually on my system. This has it's pros and cons (notably, users don't need to compile and use `bindgen`), but I still plan to add support for build-time generation gated by a feature flag.

# Container

Some C++ types can have "move constructors", executing code when struct is moved around in the memory. Obviously, Rust has no such concept, thus requiring some sort of representation for this semantics.

[`Container`](crate::container::Container) is a trait defining "something, allowing exclusive access to `T`, but not moving `T` with itself". Notably, `&mut T` and [`Box`] are such structures. [`Rc`](alloc::rc::Rc) and [`Arc`](std::sync::Arc) can be considered such, if exclusive access were to be optional[^2].

[^2]: both [`Rc`](alloc::rc::Rc) and [`Arc`](std::sync::Arc) can provide exclusive access in case the ownership is not shared (see [`Rc::get_mut`](alloc::rc::Rc::get_mut) and [`Arc::get_mut`](std::sync::Arc::get_mut))

[`Container`](crate::container::Container) implementations always provide `&T`, but never provide `&mut T` -- instead, they only ever expose [`Pin<&mut T>`](core::pin::Pin). In fact, contained structures are not allowed to more at all, unless they implement [`Moveable`](crate::container::Moveable) trait essentially defining a move constructor. In case of C++ types, these implementations are hooked to a small helper functions on C++ side, performing a move in C++ code (guaranteeing it's performed according to C++ semantics). For [`Moveable`](crate::container::Moveable) types, [`Container`](crate::container::Container)s provide additional methods, allowing ownership transfer between containers.

`&mut T` container allows storing structs locally, in case heap allocation would bring too much of overhead.

# Wrappers

One of the goals was to impose as little overhead as possible, which requires type representations shared by C++ and Rust. C++ has no idea about Rust lifetimes, so in Rust code we essentially keep two copies for each type:
- `bindgen`-generated C ABI compatible type containing raw pointers and (sometimes) blobs of bytes, representing `std::string` and `std::vector<..>`
- Rust wrapper containing exactly the same fields, but translated into safe Rust types

Wrapper types are validated in two steps:
- `sdecay-sys` tests, asserting correct size and alignment **against sizes and alignments provided by C++ side**. This prevents some errors caused by incorrect type translation by `bindgen` as well as compilation with incorrect type size assumptions (for example, in case of significantly different target)
- `sdecay` static tests, asserting correct Rust wrapper representation of `bindgen`-generated one. These assertions include total size and align assertions as well as offsets of all public fields

No amount of design is able to prevent bad config, so if you were to provide C++ side compiled with structure fields shuffled around, wrapper usage would still cause a UB. This should never happen in case of build with submodule repo (i.e. with `git` feature enabled).

# Unsafe blocks

`sdecay` crate has forbid level on `clippy::undocumented_unsafe_blocks` and `clippy::missing_safety_doc` lints. Additionally, `unsafe_code` lint is forbidden in all of the modules, except for `container` and `wrapper` (including it's submodules). That's still quite a lot of code to audit, but it's mostly forwarding Rust types into ffi calls.

**ALL** of the `unsafe` blocks are commented with a reasonable message explaining why certain operation is safe to do. There's no lint enforcing that, but generally each `unsafe` block container a single `unsafe` action, forcing explanations to be provided for each of them.

Most of the `unsafe` blocks are hidden behind macro expansion, which comes with it's pros and cons, but that's how it is.

# C++ intertop

Uncaught C++ exception is a UB, so all the functions I deemed reasonable are wrapped in exception handler on C++ side. Notably, empty `NuclideMixture` throws an exception if asked to calculate things like line intensities (whole evolution is fine, but spectral **LINES**?? unacceptable), so this wrapper returns an empty vector instead.

All C++ standard library types are completely opaque and only interacted with by a bunch of short C++ functions perforing elementary actions, like moving them around, obtaining data pointer (for vector), etc.

# Lifetimes

Thankfully, `SandiaDecay` has fairly simple lifetime model:
- everything is owned by `Database`
- please don't drop anything, database will do that

It's fairly easy to implement in Rust, so that's what I did.

# Who is nolt?

`nolt` is a helper macro setting all the lifetimes in the type to `'static`. If this rings a bell - it's for a not-yet-good-reason.

This macro is used in various situations, where I need to specify lifetime parameters, but don't have any, such as at wrapper layout test. Workaround would be to make a separate macro for that, or devise an even-more-complex syntax to define wrappers. Instead, I opted to write a macro to set all the lifetimes to `'static`.

It's not a concert _yet_, since at the time Rust has not lifetime specialization.

In case you are wondering, why `nolt` should be a _procedural_ macro, I encourage you to try doing that with declarative macro. I feel like it should be possible, and would be happy adopt your solution.

# no Miri tests?

[Miri] is an amazing tool for detecting UB in Rust code. However, as the name implies it[^3] interprets Rust mid-layer representation, most likely not including code on C++ side. That defeats the whole purpose, since most of the `unsafe` code in the crate is related to FFI calls.

[^3]: If you have any info regarding proper pronouns to use when referring to Miri, please advise me :/

[cxx]: https://cxx.rs/
[Miri]: https://github.com/rust-lang/miri
