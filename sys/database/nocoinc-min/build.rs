const URL: &str =
    "https://github.com/sandialabs/SandiaDecay/raw/refs/heads/master/sandia.decay.nocoinc.min.xml";

fn main() {
    sandia_decay_database_common::download(URL);
}
