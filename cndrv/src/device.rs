﻿use crate::{bindings::CNdev, AsRaw};
use std::{ffi::c_int, fmt};

#[repr(transparent)]
pub struct Device(pub(crate) CNdev);

impl AsRaw for Device {
    type Raw = CNdev;
    #[inline]
    unsafe fn as_raw(&self) -> Self::Raw {
        self.0
    }
}

impl Device {
    #[inline]
    pub fn count() -> usize {
        let mut count = 0;
        cndrv!(cnDeviceGetCount(&mut count));
        count as _
    }

    #[inline]
    pub fn new(index: c_int) -> Self {
        let mut device = 0;
        cndrv!(cnDeviceGet(&mut device, index));
        Self(device)
    }

    #[inline]
    pub fn fetch() -> Option<Self> {
        if Self::count() > 0 {
            Some(Self::new(0))
        } else {
            None
        }
    }

    pub fn name(&self) -> String {
        let mut name = [0u8; 256];
        cndrv!(cnDeviceGetName(
            name.as_mut_ptr().cast(),
            name.len() as _,
            self.0
        ));
        String::from_utf8(name.iter().take_while(|&&c| c != 0).copied().collect()).unwrap()
    }

    pub fn uuid(&self) -> String {
        let mut uuid = [0u8; 36];
        cndrv!(cnDeviceGetUuidStr(
            uuid.as_mut_ptr().cast(),
            uuid.len() as _,
            self.0
        ));
        String::from_utf8(uuid.iter().take_while(|&&c| c != 0).copied().collect()).unwrap()
    }

    #[inline]
    pub fn total_memory(&self) -> MemSize {
        let mut bytes = 0;
        cndrv!(cnDeviceTotalMem(&mut bytes, self.0));
        MemSize(bytes as _)
    }

    #[inline]
    pub fn isa(&self) -> c_int {
        let mut isa = 0;
        cndrv!(cnDeviceGetAttribute(
            &mut isa,
            CNdevice_attribute::CN_DEVICE_ATTRIBUTE_MLU_ISA_VERSION,
            self.0,
        ));
        isa
    }

    #[inline]
    pub fn info(&self) -> InfoFmt {
        InfoFmt(self)
    }
}

pub struct InfoFmt<'a>(&'a Device);

impl fmt::Display for InfoFmt<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MLU{} ({}: #{}) isa={} | memory={}",
            self.0 .0,
            self.0.name(),
            self.0.uuid(),
            self.0.isa(),
            self.0.total_memory(),
        )
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(transparent)]
pub struct MemSize(pub usize);

impl fmt::Display for MemSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 == 0 {
            write!(f, "0")
        } else {
            let zeros = self.0.trailing_zeros();
            if zeros >= 40 {
                write!(f, "{}TiB", self.0 >> 40)
            } else if zeros >= 30 {
                write!(f, "{}GiB", self.0 >> 30)
            } else if zeros >= 20 {
                write!(f, "{}MiB", self.0 >> 20)
            } else if zeros >= 10 {
                write!(f, "{}KiB", self.0 >> 10)
            } else {
                write!(f, "{}B", self.0)
            }
        }
    }
}

impl From<c_int> for MemSize {
    #[inline]
    fn from(value: c_int) -> Self {
        Self(value as _)
    }
}

impl From<usize> for MemSize {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value)
    }
}

#[test]
fn test() {
    crate::init();
    for i in 0..Device::count() {
        println!("{}", Device::new(i as _).info());
    }
}
