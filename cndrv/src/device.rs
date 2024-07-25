use crate::{
    bindings::{
        CNdev,
        CNdevice_attribute::{self, *},
    },
    MemSize,
};
use context_spore::AsRaw;
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
        self.get_attribute(CN_DEVICE_ATTRIBUTE_MLU_ISA_VERSION)
    }

    #[inline]
    pub fn core_nram(&self) -> MemSize {
        self.get_attribute(CN_DEVICE_ATTRIBUTE_NRAM_SIZE_PER_CORE)
            .into()
    }

    #[inline]
    pub fn core_wram(&self) -> MemSize {
        self.get_attribute(CN_DEVICE_ATTRIBUTE_WEIGHT_RAM_SIZE_PER_CORE)
            .into()
    }

    #[inline]
    pub fn core_local(&self) -> MemSize {
        self.get_attribute(CN_DEVICE_ATTRIBUTE_LOCAL_MEMORY_SIZE_PER_CORE)
            .into()
    }

    #[inline]
    pub fn cluster_smem(&self) -> MemSize {
        self.get_attribute(CN_DEVICE_ATTRIBUTE_MAX_SHARED_RAM_SIZE_PER_CLUSTER)
            .into()
    }

    #[inline]
    pub fn const_mem(&self) -> MemSize {
        self.get_attribute(CN_DEVICE_ATTRIBUTE_TOTAL_CONST_MEMORY_SIZE)
            .into()
    }

    #[inline]
    pub fn info(&self) -> InfoFmt {
        InfoFmt(self)
    }

    #[inline]
    fn get_attribute(&self, attr: CNdevice_attribute) -> c_int {
        let mut val = 0;
        cndrv!(cnDeviceGetAttribute(&mut val, attr, self.0,));
        val
    }
}

pub struct InfoFmt<'a>(&'a Device);

impl fmt::Display for InfoFmt<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "\
MLU{} ({}: #{})
  isa = {}
  memory = {}
  const = {}
  core
    nram = {}
    wram = {}
    local = {}
  cluster
    smem = {}",
            self.0 .0,
            self.0.name(),
            self.0.uuid(),
            self.0.isa(),
            self.0.total_memory(),
            self.0.const_mem(),
            self.0.core_nram(),
            self.0.core_wram(),
            self.0.core_local(),
            self.0.cluster_smem(),
        )
    }
}

#[test]
fn test() {
    crate::init();
    for i in 0..Device::count() {
        println!("{}", Device::new(i as _).info());
    }
}
