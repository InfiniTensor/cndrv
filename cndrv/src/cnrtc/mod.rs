mod binary;
mod kernel_fn;
mod module;

pub use binary::CnrtcBinary;

#[test]
fn test_behavior() {
    use std::{
        ffi::CString,
        ptr::{null, null_mut},
    };

    const CODE: &str =
        r#"extern "C" __mlu_global__ void kernel() { printf("Hello from MLU!\n"); }"#;

    crate::init();
    let Some(dev) = crate::Device::fetch() else {
        return;
    };

    let binary = {
        let src = CString::new(CODE).unwrap();
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
            CString::new("--bang-fatbin-only").unwrap(),
            CString::new(format!("--bang-mlu-arch=mtp_{}", dev.isa())).unwrap(),
        ];
        let options = options.iter().map(|o| o.as_ptr()).collect::<Vec<_>>();
        cnrtc!(cnrtcCompileCode(
            code,
            options.len() as _,
            options.as_ptr().cast_mut()
        ));

        let mut bin_size = 0;
        cnrtc!(cnrtcGetFatBinarySize(code, &mut bin_size));

        let mut bin = vec![0u8; bin_size as _];
        cnrtc!(cnrtcGetFatBinary(code, bin.as_mut_ptr().cast()));
        cnrtc!(cnrtcDestroyCode(&mut code));
        bin
    };

    crate::init();
    if let Some(dev) = crate::Device::fetch() {
        dev.context().apply(|ctx| {
            use crate::AsRaw;

            let name = CString::new("kernel").unwrap();
            let mut module = null_mut();
            let mut kernel = null_mut();
            cndrv!(cnModuleLoadFatBinary(binary.as_ptr().cast(), &mut module));
            cndrv!(cnModuleGetKernel(module, name.as_ptr(), &mut kernel));

            {
                let queue = ctx.queue();
                cndrv!(cnInvokeKernel(
                    kernel,
                    1,
                    1,
                    1,
                    cn_kernel_class::CN_KERNEL_CLASS_BLOCK,
                    0,
                    queue.as_raw(),
                    null_mut(),
                    null_mut()
                ));
            }

            cndrv!(cnModuleUnload(module));
        });
    };
}
