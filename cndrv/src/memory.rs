use crate::{bindings::CNaddr, Blob, CurrentCtx, Queue};
use context_spore::{impl_spore, AsRaw};
use std::{
    alloc::Layout,
    ffi::c_void,
    marker::PhantomData,
    mem::size_of_val,
    ops::{Deref, DerefMut},
    ptr::null_mut,
    slice::{from_raw_parts, from_raw_parts_mut},
};

#[repr(transparent)]
pub struct DevByte(#[allow(unused)] u8);

#[inline]
pub fn memcpy_d2h<T: Copy>(dst: &mut [T], src: &[DevByte]) {
    let len = size_of_val(dst);
    let dst = dst.as_mut_ptr().cast();
    assert_eq!(len, size_of_val(src));
    cndrv!(cnMemcpyDtoH(dst, src.as_ptr() as _, len as _));
}

#[inline]
pub fn memcpy_h2d<T: Copy>(dst: &mut [DevByte], src: &[T]) {
    let len = size_of_val(src);
    let src = src.as_ptr().cast();
    assert_eq!(len, size_of_val(dst));
    cndrv!(cnMemcpyHtoD(dst.as_ptr() as _, src, len as _));
}

#[inline]
pub fn memcpy_d2d(dst: &mut [DevByte], src: &[DevByte]) {
    let len = size_of_val(src);
    assert_eq!(len, size_of_val(dst));
    cndrv!(cnMemcpyDtoD(dst.as_ptr() as _, src.as_ptr() as _, len as _));
}

impl Queue<'_> {
    #[inline]
    pub fn memcpy_h2d<T: Copy>(&self, dst: &mut [DevByte], src: &[T]) {
        let len = size_of_val(src);
        let src = src.as_ptr().cast();
        assert_eq!(len, size_of_val(dst));
        cndrv!(cnMemcpyHtoDAsync_V2(
            dst.as_ptr() as _,
            src,
            len as _,
            self.as_raw()
        ));
    }

    #[inline]
    pub fn memcpy_d2d(&self, dst: &mut [DevByte], src: &[DevByte]) {
        let len = size_of_val(src);
        assert_eq!(len, size_of_val(dst));
        cndrv!(cnMemcpyDtoDAsync(
            dst.as_ptr() as _,
            src.as_ptr() as _,
            len as _,
            self.as_raw()
        ));
    }
}

impl_spore!(DevMem and DevMemSpore by (CurrentCtx, Blob<CNaddr>));

impl CurrentCtx {
    pub fn malloc<T: Copy>(&self, len: usize) -> DevMem<'_> {
        let len = Layout::array::<T>(len).unwrap().size();
        let mut ptr = 0;
        cndrv!(cnMalloc(&mut ptr, len as _));
        DevMem(unsafe { self.wrap_raw(Blob { ptr, len }) }, PhantomData)
    }

    pub fn from_host<T: Copy>(&self, slice: &[T]) -> DevMem<'_> {
        let len = size_of_val(slice);
        let src = slice.as_ptr().cast();
        let mut ptr = 0;
        cndrv!(cnMalloc(&mut ptr, len as _));
        cndrv!(cnMemcpyHtoD(ptr, src, len as _));
        DevMem(unsafe { self.wrap_raw(Blob { ptr, len }) }, PhantomData)
    }
}

impl Drop for DevMem<'_> {
    #[inline]
    fn drop(&mut self) {
        cndrv!(cnFree(self.0.rss.ptr));
    }
}

impl Deref for DevMem<'_> {
    type Target = [DevByte];
    #[inline]
    fn deref(&self) -> &Self::Target {
        if self.0.rss.len == 0 {
            &[]
        } else {
            unsafe { from_raw_parts(self.0.rss.ptr as _, self.0.rss.len) }
        }
    }
}

impl DerefMut for DevMem<'_> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        if self.0.rss.len == 0 {
            &mut []
        } else {
            unsafe { from_raw_parts_mut(self.0.rss.ptr as _, self.0.rss.len) }
        }
    }
}

impl AsRaw for DevMemSpore {
    type Raw = CNaddr;
    #[inline]
    unsafe fn as_raw(&self) -> Self::Raw {
        self.0.rss.ptr
    }
}

impl DevMemSpore {
    #[inline]
    pub const fn len(&self) -> usize {
        self.0.rss.len
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.0.rss.len == 0
    }
}

impl_spore!(HostMem and HostMemSpore by (CurrentCtx, Blob<*mut c_void>));

impl CurrentCtx {
    pub fn malloc_host<T: Copy>(&self, len: usize) -> HostMem {
        let len = Layout::array::<T>(len).unwrap().size();
        let mut ptr = null_mut();
        cndrv!(cnMallocHost(&mut ptr, len as _));
        HostMem(unsafe { self.wrap_raw(Blob { ptr, len }) }, PhantomData)
    }
}

impl Drop for HostMem<'_> {
    #[inline]
    fn drop(&mut self) {
        cndrv!(cnFreeHost(self.0.rss.ptr));
    }
}

impl AsRaw for HostMem<'_> {
    type Raw = *mut c_void;
    #[inline]
    unsafe fn as_raw(&self) -> Self::Raw {
        self.0.rss.ptr
    }
}

impl Deref for HostMem<'_> {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { from_raw_parts(self.0.rss.ptr.cast(), self.0.rss.len) }
    }
}

impl DerefMut for HostMem<'_> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { from_raw_parts_mut(self.0.rss.ptr.cast(), self.0.rss.len) }
    }
}

impl Deref for HostMemSpore {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { from_raw_parts(self.0.rss.ptr.cast(), self.0.rss.len) }
    }
}

impl DerefMut for HostMemSpore {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { from_raw_parts_mut(self.0.rss.ptr.cast(), self.0.rss.len) }
    }
}

#[test]
fn test_behavior() {
    crate::init();
    let mut ptr = null_mut();
    crate::Device::new(0).context().apply(|_| {
        cndrv!(cnMallocHost(&mut ptr, 128));
        cndrv!(cnFreeHost(ptr));
    });
    ptr = null_mut();
    cndrv!(cnFreeHost(ptr));
}
