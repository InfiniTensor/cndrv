use std::{env::var_os, path::PathBuf};

#[inline]
pub fn find_neuware_home() -> Option<PathBuf> {
    var_os("NEUWARE_HOME").map(PathBuf::from)
}
