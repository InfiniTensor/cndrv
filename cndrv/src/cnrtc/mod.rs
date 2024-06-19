#[test]
fn test_behavior() {
    use std::{
        ffi::CString,
        ptr::{null, null_mut},
    };

    let src = r#"\
#include "bang.h"
#define LEN 512

__mlu_func__ void f1(float* nx, float* ny) {
    for (int i = 0; i < LEN; ++i) {
        ny[i] = nx[i] * 100 + 1;
    }
}

extern "C" __mlu_global__ void devKernel(void *x, void *y) {
    __nram__ float nx[LEN];
    __nram__ float ny[LEN];
    __memcpy(nx, x, LEN * sizeof(float), GDRAM2NRAM);
    f1(nx, ny);
    __memcpy(y, ny, LEN * sizeof(float), NRAM2GDRAM);
}"#;

    let code = CString::new(src).unwrap();

    crate::init();
    let Some(dev) = crate::Device::fetch() else {
        return;
    };

    let mut isa = 0;
    cndrv!(cnDeviceGetAttribute(
        &mut isa,
        CNdevice_attribute::CN_DEVICE_ATTRIBUTE_MLU_ISA_VERSION,
        dev.0,
    ));

    let binary = {
        let mut program = null_mut();
        cnrtc!(cnrtcCreateCodeV2(
            &mut program,
            code.as_ptr().cast(),
            cnrtcCodeType::CNRTC_CODE_BANGC,
            null(),
            0,
            null_mut(),
            null_mut(),
        ));

        let options = [
            CString::new("--bang-fatbin-only").unwrap(),
            CString::new(format!("--bang-mlu-arch=mtp_{isa}")).unwrap(),
        ];
        let options = options.iter().map(|o| o.as_ptr()).collect::<Vec<_>>();
        cnrtc!(cnrtcCompileCode(
            program,
            options.len() as _,
            options.as_ptr().cast_mut()
        ));

        let mut bin_size = 0;
        cnrtc!(cnrtcGetFatBinarySize(program, &mut bin_size));

        let mut bin = vec![0u8; bin_size as _];
        cnrtc!(cnrtcGetFatBinary(program, bin.as_mut_ptr().cast()));
        cnrtc!(cnrtcDestroyCode(&mut program));
        bin
    };
    let name = CString::new("devKernel").unwrap();

    let mut module = null_mut();
    let mut kernel = null_mut();
    crate::init();
    if let Some(dev) = crate::Device::fetch() {
        dev.context().apply(|_| {
            cndrv!(cnModuleLoadFatBinary(binary.as_ptr().cast(), &mut module));
            cndrv!(cnModuleGetKernel(module, name.as_ptr(), &mut kernel));

            // driver!(cuLaunchKernel(
            //     f,
            //     1,
            //     1,
            //     1,
            //     1,
            //     1,
            //     1,
            //     0,
            //     null_mut(),
            //     null_mut(),
            //     null_mut()
            // ));
        });
    };
}
