#![doc = include_str!("../README.md")]
#![deny(warnings, unsafe_code, missing_docs)]

use std::{env::var_os, path::PathBuf};

/// Returns the path to the Neuware home directory, if it is set.
#[inline]
pub fn find_neuware_home() -> Option<PathBuf> {
    var_os("NEUWARE_HOME").map(PathBuf::from)
}
