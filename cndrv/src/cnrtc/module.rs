use crate::{bindings::CNmodule, impl_spore, AsRaw, CnrtcBinary, ContextGuard};
use std::{marker::PhantomData, ptr::null_mut};

impl_spore!(Module and ModuleSpore by CNmodule);

impl ContextGuard<'_> {
    #[inline]
    pub fn load(&self, bin: CnrtcBinary) -> Module {
        let mut module = null_mut();
        cndrv!(cnModuleLoadFatBinary(bin.as_ptr(), &mut module));
        Module(unsafe { self.wrap_resource(module) }, PhantomData)
    }
}

impl Drop for Module<'_> {
    #[inline]
    fn drop(&mut self) {
        cndrv!(cnModuleUnload(self.0.res));
    }
}

impl AsRaw for Module<'_> {
    type Raw = CNmodule;
    #[inline]
    unsafe fn as_raw(&self) -> Self::Raw {
        self.0.res
    }
}
