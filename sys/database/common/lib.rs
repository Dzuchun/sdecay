#![doc = include_str!("README.md")]
#![allow(
    missing_docs,
    clippy::missing_panics_doc,
    clippy::items_after_statements
)]

use std::{
    env::var_os,
    fs::File,
    io::{BufWriter, Read, Write},
    path::PathBuf,
};

pub fn download(url: &str) {
    let out_dir = PathBuf::from(var_os("OUT_DIR").expect("should have a cargo output dir"));
    let database_path = out_dir.join("database.xml");
    {
        let mut response = minreq::get(url)
            .send_lazy()
            .expect("should be able to download a database file");

        let database_file = File::options()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&database_path)
            .expect("should be able to open database file");
        let mut writer = BufWriter::new(database_file);

        const BUFSIZ: usize = 2 << 20;
        let mut buf = vec![0; BUFSIZ];
        loop {
            let len = response
                .read(buf.as_mut_slice())
                .expect("should be able to read a response");
            if len == 0 {
                // we are done
                break;
            }
            writer
                .write_all(&buf[..len])
                .expect("should be able to write into a file");
        }
    }
}
