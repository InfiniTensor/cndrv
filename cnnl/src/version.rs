use crate::bindings::cnnlGetLibVersion;
use std::{ffi::c_int, fmt};

pub struct Version {
    major: c_int,
    minor: c_int,
    patch: c_int,
}

impl Version {
    #[inline]
    pub fn get() -> Self {
        let mut ans = Self {
            major: 0,
            minor: 0,
            patch: 0,
        };
        unsafe { cnnlGetLibVersion(&mut ans.major, &mut ans.minor, &mut ans.patch) };
        ans
    }
}

impl fmt::Display for Version {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[test]
fn test() {
    println!("Version: {}", Version::get());
}
