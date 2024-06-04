#![cfg(detected_neuware)]

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

#[inline(always)]
pub fn init() {
    cndrv!(cnInit(0));
}

#[test]
fn test() {
    init();
}
