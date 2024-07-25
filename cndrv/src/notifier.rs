use crate::{bindings::CNnotifier, CurrentCtx, Queue};
use context_spore::{impl_spore, AsRaw};
use std::{marker::PhantomData, ptr::null_mut, time::Duration};

impl_spore!(Notifier and NotifierSpore by (CurrentCtx, CNnotifier));

impl<'ctx> Queue<'ctx> {
    pub fn record(&self) -> Notifier<'ctx> {
        let mut event = null_mut();
        cndrv!(cnCreateNotifier(
            &mut event,
            CNNotifierFlags::CN_NOTIFIER_DEFAULT as _
        ));
        cndrv!(cnPlaceNotifier(event, self.as_raw()));
        Notifier(unsafe { self.ctx().wrap_raw(event) }, PhantomData)
    }
}

impl Drop for Notifier<'_> {
    #[inline]
    fn drop(&mut self) {
        cndrv!(cnDestroyNotifier(self.0.rss));
    }
}

impl AsRaw for Notifier<'_> {
    type Raw = CNnotifier;
    #[inline]
    unsafe fn as_raw(&self) -> Self::Raw {
        self.0.rss
    }
}

impl Queue<'_> {
    pub fn bench(&self, f: impl Fn(usize, &Self), times: usize, warm_up: usize) -> Duration {
        for i in 0..warm_up {
            f(i, self);
        }
        let start = self.record();
        for i in 0..times {
            f(i, self);
        }
        let end = self.record();
        end.synchronize();
        end.elapse_from(&start).div_f32(times as _)
    }
}

impl Notifier<'_> {
    #[inline]
    pub fn synchronize(&self) {
        cndrv!(cnWaitNotifier(self.0.rss));
    }

    #[inline]
    pub fn elapse_from(&self, start: &Self) -> Duration {
        let mut ms = 0.0;
        cndrv!(cnNotifierElapsedTime(&mut ms, start.0.rss, self.0.rss));
        Duration::from_secs_f32(ms * 1e-3)
    }
}
