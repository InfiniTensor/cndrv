use crate::bindings::cnnlHandle_t;
use cndrv::{impl_spore, AsRaw, ContextGuard, Queue};
use std::{marker::PhantomData, ptr::null_mut};

impl_spore!(Cnnl and CnnlSpore by cnnlHandle_t);

impl Drop for Cnnl<'_> {
    #[inline]
    fn drop(&mut self) {
        cnnl!(cnnlDestroy(self.0.raw));
    }
}

impl AsRaw for Cnnl<'_> {
    type Raw = cnnlHandle_t;
    #[inline]
    unsafe fn as_raw(&self) -> Self::Raw {
        self.0.raw
    }
}

impl<'ctx> Cnnl<'ctx> {
    #[inline]
    pub fn new(ctx: &'ctx ContextGuard) -> Self {
        let mut handle = null_mut();
        cnnl!(cnnlCreate(&mut handle));
        Self(unsafe { ctx.wrap_raw(handle) }, PhantomData)
    }

    #[inline]
    pub fn set_queue(&mut self, queue: &Queue) {
        cnnl!(cnnlSetQueue(self.0.raw, queue.as_raw().cast()));
    }
}
