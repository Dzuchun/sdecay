use std::{
    env::var_os,
    path::{Path, PathBuf},
};

fn main() {
    println!("cargo:rerun-if-changed=lib.rs");
    println!("cargo:rerun-if-changed=wrapper.hpp");
    println!("cargo:rerun-if-changed=wrapper.cc");
    println!("cargo:rerun-if-changed=vendor");

    println!("cargo::rerun-if-env-changed=SANDIA_DECAY_INCLUDE_DIR");
    println!("cargo::rerun-if-env-changed=SANDIA_DECAY_LIB_DIR");
    println!("cargo::rerun-if-env-changed=SANDIA_DECAY_STATIC");

    if cfg!(feature = "git") || var_os("SANDIA_DECAY_GIT").is_some() {
        // build wrapper and library
        let vendor = Path::new("vendor");
        // build the wrapper
        cc::Build::new()
            .cpp(true)
            .file("wrapper.cc")
            .include(vendor)
            .compile("wrapper");
        // build the lib
        cc::Build::new()
            .cpp(true)
            .cargo_warnings(false)
            .include(vendor)
            .include(vendor.join("3rdparty"))
            .file(vendor.join("SandiaDecay.cpp"))
            .compile("SandiaDecay");
    } else {
        let ignore_checks = var_os("SANDIA_DECAY_IGNORE_CHECKS").is_some();
        // build the wrapper
        let mut wrapper = cc::Build::new();
        wrapper.cpp(true);
        if let Some(include_path) = var_os("SANDIA_DECAY_INCLUDE_DIR") {
            let include_path = PathBuf::from(include_path);
            assert!(
                ignore_checks || include_path.join("SandiaDecay.h").exists(),
                "`SANDIA_DECAY_INCLUDE_DIR` variable is set, but pointed location ({}) does not appear to contain SandiaDecay headers. Please, point to the correct location, or rename headers `SandiaDecay.h`",
                include_path.display(),
            );
            wrapper.include(include_path);
        }
        wrapper.file("wrapper.cc").compile("wrapper");

        if let Some(library_path) = var_os("SANDIA_DECAY_LIB_DIR") {
            // search for built library at specified location
            let library_path = PathBuf::from(library_path);
            assert!(
                ignore_checks
                    || library_path.join("SandiaDecay.lib").exists()
                    || library_path.join("libSandiaDecay.a").exists(),
                "`SANDIA_DECAY_LIB_DIR` variable is set, but pointed location ({}) does not appear to contain built SandiaDecay library. Please, point to the correct location, or rename the file to `SandiaDecay.lib` or `libSandiaDecay.a`",
                library_path.display(),
            );
            println!("cargo:rustc-link-search=native={}", library_path.display());
        } else {
            // search in default paths only
            println!(
                "cargo:warning=`sdecay-sys` is built in dynamic mode without specified library path, i.e. relying on default system path to discover the library"
            );
            println!(
                "cargo::warning=If your build fails, try enabling `sdecay-sys/static` feature, or setting `SANDIA_DECAY_STATIC` environment variable"
            );
            println!("cargo:rustc-link-lib=stdc++");
        }
        println!("cargo:rustc-link-lib=SandiaDecay");
    }
}
