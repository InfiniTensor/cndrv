use crate::{
    bindings::{CNCtxSched, CNcontext, CNdev},
    AsRaw, Device, RawContainer,
};
use std::{ffi::c_uint, ptr::null_mut};

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct Context {
    ctx: CNcontext,
    dev: CNdev,
    shared: bool,
}

impl Device {
    #[inline]
    pub fn context(&self) -> Context {
        const FLAG: c_uint = CNCtxSched::CN_CTX_SCHED_SYNC_AUTO as _;
        let dev = unsafe { self.as_raw() };
        let mut ctx = null_mut();
        cndrv!(cnCtxCreate(&mut ctx, FLAG, dev));
        Context {
            ctx,
            dev,
            shared: false,
        }
    }

    #[inline]
    pub fn acquire_shared(&self) -> Context {
        let dev = unsafe { self.as_raw() };
        let mut ctx = null_mut();
        cndrv!(cnSharedContextAcquire(&mut ctx, dev));
        Context {
            ctx,
            dev,
            shared: true,
        }
    }
}

impl Drop for Context {
    #[inline]
    fn drop(&mut self) {
        if self.shared {
            cndrv!(cnSharedContextRelease(self.dev));
        } else {
            cndrv!(cnCtxDestroy(self.ctx));
        }
    }
}

unsafe impl Send for Context {}
unsafe impl Sync for Context {}

impl AsRaw for Context {
    type Raw = CNcontext;
    #[inline]
    unsafe fn as_raw(&self) -> Self::Raw {
        self.ctx
    }
}

impl Context {
    #[inline]
    pub fn api_version(&self) -> c_uint {
        let mut version = 0;
        cndrv!(cnCtxGetApiVersion(self.ctx, &mut version));
        version
    }

    #[inline]
    pub fn device(&self) -> Device {
        Device(self.dev)
    }

    #[inline]
    pub fn apply<T>(&self, f: impl FnOnce(&CurrentCtx) -> T) -> T {
        let mut current = null_mut();
        cndrv!(cnCtxGetCurrent(&mut current));
        cndrv!(cnCtxSetCurrent(self.ctx));
        let ans = f(&CurrentCtx(self.ctx));
        cndrv!(cnCtxSetCurrent(current));
        ans
    }
}

#[repr(transparent)]
pub struct CurrentCtx(CNcontext);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct NoCtxError;

impl AsRaw for CurrentCtx {
    type Raw = CNcontext;
    #[inline]
    unsafe fn as_raw(&self) -> Self::Raw {
        self.0
    }
}

impl CurrentCtx {
    #[inline]
    pub fn dev(&self) -> Device {
        let mut dev = 0;
        cndrv!(cnCtxGetDevice(&mut dev));
        Device(dev)
    }

    #[inline]
    pub fn synchronize(&self) {
        cndrv!(cnCtxSync());
    }

    /// Applies `f` to current context if it exists, otherwise returns `Err(NoCtxError)`.
    #[inline]
    pub fn apply_current<T>(f: impl FnOnce(&Self) -> T) -> Result<T, NoCtxError> {
        let mut raw = null_mut();
        cndrv!(cnCtxGetCurrent(&mut raw));
        if !raw.is_null() {
            Ok(f(&Self(raw)))
        } else {
            Err(NoCtxError)
        }
    }

    /// Designates `raw` as the current context without checking, then applies `f` to the context.
    ///
    /// # Safety
    ///
    /// The `raw` context must be the current pushed context on this thread.
    #[inline]
    pub unsafe fn apply_current_unchecked<T>(raw: CNcontext, f: impl FnOnce(&Self) -> T) -> T {
        f(&Self(raw))
    }

    /// Designates `raw` as the current context.
    ///
    /// # Safety
    ///
    /// The `raw` context must be the current pushed context on this thread.
    /// Generally, this method only used for [`RawContainer::ctx`] with limited lifetime.
    #[inline]
    pub unsafe fn from_raw(raw: &CNcontext) -> &Self {
        &*(raw as *const _ as *const _)
    }

    /// Wraps a raw object in a `RawContainer`.
    ///
    /// # Safety
    ///
    /// The raw object must be created in this [`Context`].
    #[inline]
    pub unsafe fn wrap_raw<T: Unpin>(&self, raw: T) -> RawContainer<T> {
        RawContainer { ctx: self.0, raw }
    }
}

#[test]
fn test() {
    use crate::Device;

    crate::init();
    for i in 0..Device::count() {
        let device = Device::new(i as _);

        let context = device.context();
        println!(
            "mlu{i}:        context api version={}",
            context.api_version()
        );

        let context = device.acquire_shared();
        println!(
            "mlu{i}: shared context api version={}",
            context.api_version()
        );
    }
}
