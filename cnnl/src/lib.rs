#![doc = include_str!("../README.md")]
#![cfg(detected_neuware)]
#![deny(warnings)]

#[macro_use]
#[allow(unused, non_upper_case_globals, non_camel_case_types, non_snake_case)]
pub mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

    #[macro_export]
    macro_rules! cnnl {
        ($f:expr) => {{
            #[allow(unused_imports)]
            use $crate::bindings::*;
            #[allow(unused_unsafe)]
            let err = unsafe { $f };
            assert_eq!(err, cnnlStatus_t::CNNL_STATUS_SUCCESS);
        }};
    }
}

mod data_type;
mod handle;
mod tensor;
mod version;

pub use data_type::DataType;
pub use handle::{Cnnl, CnnlSpore};
pub use tensor::Tensor;
pub use version::Version;
