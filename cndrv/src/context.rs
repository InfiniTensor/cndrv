use crate::{bindings as cn, AsRaw, Device, RawContainer};
use std::{ffi::c_uint, marker::PhantomData, ptr::null_mut};

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct Context {
    ctx: cn::CNcontext,
    dev: cn::CNdev,
    shared: bool,
}

impl Device {
    #[inline]
    pub fn context(&self) -> Context {
        const FLAG: c_uint = cn::CNCtxSched::CN_CTX_SCHED_SYNC_AUTO as _;
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
    type Raw = cn::CNcontext;
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
    pub fn apply<T>(&self, f: impl FnOnce(&ContextGuard) -> T) -> T {
        f(&self.bound())
    }
}

#[repr(transparent)]
pub struct ContextGuard<'a>(cn::CNcontext, PhantomData<&'a ()>);

impl Context {
    #[inline]
    fn bound(&self) -> ContextGuard {
        cndrv!(cnCtxSetCurrent(self.ctx));
        ContextGuard(self.ctx, PhantomData)
    }
}

impl Drop for ContextGuard<'_> {
    #[inline]
    fn drop(&mut self) {
        debug_assert_eq!(
            {
                let mut current = null_mut();
                cndrv!(cnCtxGetCurrent(&mut current));
                current
            },
            self.0
        );
        cndrv!(cnCtxSetCurrent(null_mut()));
    }
}

impl AsRaw for ContextGuard<'_> {
    type Raw = cn::CNcontext;
    #[inline]
    unsafe fn as_raw(&self) -> Self::Raw {
        self.0
    }
}

impl ContextGuard<'_> {
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

    /// Wrap a raw object in a `RawContainer`.
    ///
    /// # Safety
    ///
    /// The raw object must be created in this [`Context`].
    #[inline]
    pub unsafe fn wrap_raw<T>(&self, raw: T) -> RawContainer<T> {
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
