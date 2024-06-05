use std::{cmp::Ordering, ffi::c_int, fmt};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Version {
    major: c_int,
    minor: c_int,
    patch: c_int,
}

impl fmt::Display for Version {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl PartialOrd for Version {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Equal => match self.minor.cmp(&other.minor) {
                Ordering::Equal => self.patch.cmp(&other.patch),
                ord => ord,
            },
            ord => ord,
        }
    }
}

#[inline]
pub fn driver_version() -> Version {
    let mut version = Version {
        major: 0,
        minor: 0,
        patch: 0,
    };
    cndrv!(cnGetDriverVersion(
        &mut version.major,
        &mut version.minor,
        &mut version.patch
    ));
    version
}

#[inline]
pub fn library_version() -> Version {
    let mut version = Version {
        major: 0,
        minor: 0,
        patch: 0,
    };
    cndrv!(cnGetLibVersion(
        &mut version.major,
        &mut version.minor,
        &mut version.patch
    ));
    version
}

#[test]
fn test() {
    crate::init();
    println!("Driver version: {}", driver_version());
    println!("Library version: {}", library_version());
}
