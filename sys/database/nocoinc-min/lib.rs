#![doc = "README.md"]
#![allow(missing_docs)]

use pathsep::{join_path, path_separator};

#[cfg(not(docsrs))]
pub const FILE: &[u8] = include_bytes!(join_path!(env!("OUT_DIR"), "database.xml"));
#[cfg(docsrs)]
pub const FILE: &[u8] = &[];
