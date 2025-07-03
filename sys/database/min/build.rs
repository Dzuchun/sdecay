use std::env::var_os;

const URL: &str =
    "https://github.com/sandialabs/SandiaDecay/raw/refs/heads/master/sandia.decay.min.xml";

fn main() {
    println!("cargo::rustc-check-cfg=cfg(docsrs)");
    if var_os("DOCS_RS").is_some() {
        println!("cargo::rustc-cfg=docsrs");
    } else {
        sandia_decay_database_common::download(URL);
    }
}
