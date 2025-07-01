#![doc = "README.md"]

use pathsep::{join_path, path_separator};

#[allow(missing_docs)]
pub const FILE: &[u8] = include_bytes!(join_path!(env!("OUT_DIR"), "database.xml"));
