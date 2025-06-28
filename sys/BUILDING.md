`sdecay-sys` has multiple build modes, defined by
- `SANDIA_DECAY_INCLUDE_DIR` environment variable
- `SANDIA_DECAY_LIB_DIR` environment variable
- `SANDIA_DECAY_GIT` environment variable
- `SANDIA_DECAY_IGNORE_CHECKS` environment variable
- `git` feature of `sdecay-sys` crate [^1]

[^1]: note, that `sdecay` has a `git` feature for the purpose of relaying it to `sdecay-sys`. If you are using `sdecay`, you can enable it's `git` feature to trigger the behavior

# Git submodule
If `git` feature is active, or `SANDIA_DECAY_GIT` environment variable is _set_ (has any value, possibly `""`), `SandiaDecay` library will be built with the crate and statically linked to it.

Most likely, this is the mode you want to use, as it required no manual clone and `cmake` build - in fact, you don't even need `cmake` installed on your system!

Example:
```toml
# Cargo.toml
[dependencies]
sdecay = { version = ..., features = [ "git" ] }
```
or
```toml
# .cargo/config.toml
SANDIA_DECAY_GIT="1"
```

# Paths
If `git` feature is disabled and `SANDIA_DECAY_GIT` environment variable is not set, crate checks for `SANDIA_DECAY_INCLUDE_DIR` and `SANDIA_DECAY_LIB_DIR` environment variables, corresponding to location of `SandiaDecay`'s header and compiled binary.

<section class="info">
Note, that compilation will likely fail, if either of these locations do not contain expected files, so crate will try to gracefully fail if this is the case. In case you want to continue the compilation anyway, please set <code>SANDIA_DECAY_IGNORE_CHECKS</code> environment variable.
</section>

This is a reasonable option, if `SandiaDecay` is used by other programs, but not installed system-wide.

Example:
```toml
# Cargo.toml
[dependencies]
sdecay = ...
```
```toml
# .cargo/config.toml
SANDIA_DECAY_INCLUDE_DIR = "/home/me/repos/SandiaDecay/"
SANDIA_DECAY_LIB_DIR = "/home/me/repos/SandiaDecay/build"
```
(assuming `SandiaDecay` repo was cloned to `/home/me/repos/SandiaDecay/` and built it the `build` subdirectory)

# From default
In case none of the mentioned elements are present, crate will still try to compile hoping for `SandiaDecay` headers to be in "default include paths", and `SandiaDecay` binary be in "default library paths".

Example:
```toml
# Cargo.toml
[dependencies]
sdecay = ...
```
(assuming `SandiaDecay.h` is somewhere like `/usr/include`, and `libSandiaDecay.a` is somewhere like `/usr/lib`)
