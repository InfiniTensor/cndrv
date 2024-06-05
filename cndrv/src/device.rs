use crate::{bindings as cn, AsRaw};
use std::ffi::c_int;

#[repr(transparent)]
pub struct Device(cn::CNdev);

impl AsRaw for Device {
    type Raw = cn::CNdev;
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

    #[inline]
    pub fn total_memory(&self) -> usize {
        let mut bytes = 0;
        cndrv!(cnDeviceTotalMem(&mut bytes, self.0));
        bytes as _
    }
}

#[test]
fn test() {
    crate::init();
    for i in 0..Device::count() {
        let dev = Device::new(i as _);
        println!("mlu{i}: mem={}", dev.total_memory(),);
    }
}
