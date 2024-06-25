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
            #[allow(unused_unsafe)]
            let err = unsafe { $f };
            assert_eq!(err, CNresult::CN_SUCCESS);
        }};
    }

    #[macro_export]
    macro_rules! cnrtc {
        ($f:expr) => {{
            #[allow(unused_imports)]
            use $crate::bindings::*;
            #[allow(unused_unsafe)]
            let err = unsafe { $f };
            assert_eq!(err, cnrtcStatus::CNRTC_SUCCESS);
        }};
    }
}

mod spore;

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

pub trait AsRaw {
    type Raw;
    /// # Safety
    ///
    /// The caller must ensure that the returned item is dropped before the original item.
    unsafe fn as_raw(&self) -> Self::Raw;
}

#[inline(always)]
pub fn init() {
    cndrv!(cnInit(0));
}

pub use cnrtc::CnrtcBinary;
pub use context::{Context, CurrentCtx};
pub use device::Device;
pub use memory::{memcpy_d2d, memcpy_d2h, memcpy_h2d, DevByte, DevMem, DevMemSpore};
pub use notifier::{Notifier, NotifierSpore};
pub use queue::{Queue, QueueSpore};
pub use spore::{ContextResource, ContextSpore, RawContainer};
pub use version::{driver_version, library_version, Version};

struct Blob<P> {
    ptr: P,
    len: usize,
}
