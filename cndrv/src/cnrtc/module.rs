﻿use crate::{bindings::CNmodule, CnrtcBinary, CurrentCtx};
use context_spore::{impl_spore, AsRaw};
use std::{marker::PhantomData, ptr::null_mut};

impl_spore!(Module and ModuleSpore by (CurrentCtx, CNmodule));

impl CurrentCtx {
    #[inline]
    pub fn load(&self, bin: &CnrtcBinary) -> Module {
        let mut module = null_mut();
        cndrv!(cnModuleLoadFatBinary(bin.as_ptr(), &mut module));
        Module(unsafe { self.wrap_raw(module) }, PhantomData)
    }
}

impl Drop for Module<'_> {
    #[inline]
    fn drop(&mut self) {
        cndrv!(cnModuleUnload(self.0.rss));
    }
}

impl AsRaw for Module<'_> {
    type Raw = CNmodule;
    #[inline]
    unsafe fn as_raw(&self) -> Self::Raw {
        self.0.rss
    }
}
