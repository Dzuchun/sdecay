const URL: &str =
    "https://github.com/sandialabs/SandiaDecay/raw/refs/heads/master/sandia.decay.min.xml";

fn main() {
    sandia_decay_database_common::download(URL);
}
