use crate::{bindings::CNkernel, AsRaw, Queue};
use std::{
    ffi::{c_void, CStr},
    ptr::null_mut,
};

use super::module::Module;

pub struct KernelFn<'m>(CNkernel, #[allow(unused)] &'m Module<'m>);

impl<'m> Module<'m> {
    pub fn get_kernel(&'m self, name: impl AsRef<CStr>) -> KernelFn<'m> {
        let name = name.as_ref();
        let mut kernel = null_mut();
        cndrv!(cnModuleGetKernel(self.as_raw(), name.as_ptr(), &mut kernel));
        KernelFn(kernel, self)
    }
}

impl KernelFn<'_> {
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
}
