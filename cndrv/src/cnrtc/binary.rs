use crate::bindings::{cnrtcCompileCode, cnrtcStatus};
use std::{
    ffi::{c_int, CString},
    os::raw::c_void,
    ptr::{null, null_mut},
};

pub struct CnrtcBinary(Vec<u8>);

impl CnrtcBinary {
    pub fn compile(src: impl AsRef<str>, isa: c_int) -> (Result<Self, cnrtcStatus>, String) {
        let src = CString::new(src.as_ref()).unwrap();
        let mut code = null_mut();
        cnrtc!(cnrtcCreateCodeV2(
            &mut code,
            src.as_ptr().cast(),
            cnrtcCodeType::CNRTC_CODE_BANGC,
            null(),
            0,
            null_mut(),
            null_mut(),
        ));

        let options = [
            CString::new("-O3").unwrap(),
            CString::new(format!("--bang-mlu-arch=mtp_{isa}")).unwrap(),
        ];
        let options = options.iter().map(|s| s.as_ptr()).collect::<Vec<_>>();

        let result =
            unsafe { cnrtcCompileCode(code, options.len() as _, options.as_ptr().cast_mut()) };
        let log = {
            let mut log_len = 0;
            cnrtc!(cnrtcGetCompilationLogSize(code, &mut log_len));
            if log_len > 1 {
                let mut log = vec![0u8; log_len as _];
                cnrtc!(cnrtcGetCompilationLog(code, log.as_mut_ptr().cast()));
                log.pop();
                std::str::from_utf8(&log).unwrap().trim().to_string()
            } else {
                String::new()
            }
        };
        let ans = if result == cnrtcStatus::CNRTC_SUCCESS {
            let mut bin_len = 0;
            cnrtc!(cnrtcGetFatBinarySize(code, &mut bin_len));
            let mut bin = vec![0u8; bin_len as _];
            cnrtc!(cnrtcGetFatBinary(code, bin.as_mut_ptr().cast()));
            cnrtc!(cnrtcDestroyCode(&mut code));
            Ok(Self(bin))
        } else {
            Err(result)
        };
        (ans, log)
    }
}

impl CnrtcBinary {
    #[inline]
    pub fn as_ptr(&self) -> *const c_void {
        self.0.as_ptr().cast()
    }
}
