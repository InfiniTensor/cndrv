use super::module::Module;
use crate::{
    bindings::{
        CNkernel,
        CNkernel_attribute::{self, *},
    },
    MemSize, Queue,
};
use context_spore::AsRaw;
use std::{
    ffi::{c_void, CStr},
    marker::PhantomData,
    ptr::null_mut,
};

#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct KernelFn<'m>(CNkernel, PhantomData<&'m ()>);

impl Module<'_> {
    #[inline]
    pub fn get_kernel(&self, name: impl AsRef<CStr>) -> KernelFn {
        let name = name.as_ref();
        let mut kernel = null_mut();
        cndrv!(cnModuleGetKernel(self.as_raw(), name.as_ptr(), &mut kernel));
        KernelFn(kernel, PhantomData)
    }
}

impl KernelFn<'_> {
    #[inline]
    pub fn launch(
        &self,
        dimz: u32,
        dimy: u32,
        dimx: u32,
        params: *const *const c_void,
        queue: &Queue,
    ) {
        cndrv!(cnInvokeKernel(
            self.0,
            dimx,
            dimy,
            dimz,
            cn_kernel_class::CN_KERNEL_CLASS_BLOCK,
            0,
            queue.as_raw(),
            params as _,
            null_mut(),
        ));
    }

    #[inline]
    pub fn nram_usage(&self) -> MemSize {
        self.get_attribute(CN_KERNEL_ATTRIBUTE_NRAM_SIZE_BYTES)
            .into()
    }

    #[inline]
    pub fn wram_usage(&self) -> MemSize {
        self.get_attribute(CN_KERNEL_ATTRIBUTE_WEIGHT_RAM_SIZE_BYTES)
            .into()
    }

    #[inline]
    pub fn smem_usage(&self) -> MemSize {
        self.get_attribute(CN_KERNEL_ATTRIBUTE_SHARED_SIZE_BYTES)
            .into()
    }

    #[inline]
    pub fn const_usage(&self) -> MemSize {
        self.get_attribute(CN_KERNEL_ATTRIBUTE_CONST_SIZE_BYTES)
            .into()
    }

    #[inline]
    pub fn binary_version(&self) -> i64 {
        self.get_attribute(CN_KERNEL_ATTRIBUTE_BINARY_VERSION)
    }

    #[inline]
    fn get_attribute(&self, attr: CNkernel_attribute) -> i64 {
        let mut value = 0;
        cndrv!(cnKernelGetAttribute(&mut value, attr, self.0));
        value
    }
}
