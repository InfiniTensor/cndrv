#![doc = include_str!("../README.md")]
#![cfg(detected_neuware)]
#![deny(warnings)]

#[macro_use]
#[allow(unused, non_upper_case_globals, non_camel_case_types, non_snake_case)]
pub mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

    #[macro_export]
    macro_rules! cndrv {
        ($f:expr) => {{
            #[allow(unused_imports)]
            use $crate::bindings::*;
            #[allow(unused_unsafe, clippy::macro_metavars_in_unsafe)]
            let err = unsafe { $f };
            assert_eq!(err, CNresult::CN_SUCCESS);
        }};
    }

    #[macro_export]
    macro_rules! cnrtc {
        ($f:expr) => {{
            #[allow(unused_imports)]
            use $crate::bindings::*;
            #[allow(unused_unsafe, clippy::macro_metavars_in_unsafe)]
            let err = unsafe { $f };
            assert_eq!(err, cnrtcStatus::CNRTC_SUCCESS);
        }};
    }
}

/// §4.3 Version Management
mod version;

/// §4.4 Device Management
mod device;

/// §4.5 Context Management
mod context;

/// §4.6 Memory Management
mod memory;

/// §4.7 Queue Management
mod queue;

/// §4.8 Notifier Management
mod notifier;

/// §4.9 Atomic Operation Management
// mod atomic;

/// §4.10 Module Management
// mod module;

/// §4.11 Execution Control Management
// mod execution;

/// §4.12 Virtual Memory Management
// mod vmem;

/// §4.13 Task Topo Management
// mod task_topo;

/// CNRTC
mod cnrtc;

#[inline(always)]
pub fn init() {
    cndrv!(cnInit(0));
}

pub use cnrtc::CnrtcBinary;
pub use context::{Context, CurrentCtx, NoCtxError};
pub use context_spore::{impl_spore, AsRaw, ContextResource, ContextSpore, RawContainer};
pub use device::Device;
pub use memory::{
    memcpy_d2d, memcpy_d2h, memcpy_h2d, DevByte, DevMem, DevMemSpore, HostMem, HostMemSpore,
};
pub use notifier::{Notifier, NotifierSpore};
pub use queue::{Queue, QueueSpore};
pub use version::{driver_version, library_version, Version};

struct Blob<P> {
    ptr: P,
    len: usize,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(transparent)]
pub struct MemSize(pub usize);

use std::{ffi::c_int, fmt};

impl fmt::Display for MemSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 == 0 {
            write!(f, "0")
        } else {
            let zeros = self.0.trailing_zeros();
            if zeros >= 40 {
                write!(f, "{}TiB", self.0 >> 40)
            } else if zeros >= 30 {
                write!(f, "{}GiB", self.0 >> 30)
            } else if zeros >= 20 {
                write!(f, "{}MiB", self.0 >> 20)
            } else if zeros >= 10 {
                write!(f, "{}KiB", self.0 >> 10)
            } else {
                write!(f, "{}B", self.0)
            }
        }
    }
}

impl From<c_int> for MemSize {
    #[inline]
    fn from(value: c_int) -> Self {
        Self(value as _)
    }
}

impl From<i64> for MemSize {
    #[inline]
    fn from(value: i64) -> Self {
        Self(value as _)
    }
}
