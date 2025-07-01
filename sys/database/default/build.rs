const URL: &str = "https://github.com/sandialabs/sandiadecay/raw/cd75314b56e12b80497531a1fbeed9c8e9968403/sandia.decay.xml";

fn main() {
    sandia_decay_database_common::download(URL);
}
