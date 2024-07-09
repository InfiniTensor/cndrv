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

    const CODE: &str = r#"extern "C" __mlu_entry__ void kernel() { printf("Hello from MLU!\n"); }"#;

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

#[test]
fn test_add() {
    use crate::memcpy_d2h;
    use std::ffi::{c_void, CString};

    const N: usize = 64;
    let src = format!(
        r#"
extern "C" __mlu_entry__ void kernel(
    float      * ans_,
    float const* lhs_,
    float const* rhs_
) {{
    __nram__ float lhs[{N}];
    __nram__ float rhs[{N}];
    __nram__ float ans[{N}];
    __memcpy(lhs, lhs_, {N} * sizeof(float), GDRAM2NRAM);
    __memcpy(rhs, rhs_, {N} * sizeof(float), GDRAM2NRAM);
    __bang_add(ans, lhs, rhs, {N});
    __memcpy(ans_, ans, {N} * sizeof(float), NRAM2GDRAM);
}}"#
    );

    crate::init();
    let Some(dev) = crate::Device::fetch() else {
        return;
    };

    let (result, log) = CnrtcBinary::compile(src, dev.isa());
    if !log.is_empty() {
        eprintln!("{log}");
    }
    let bin = result.unwrap();

    let a = vec![1.0f32; N];
    let b = vec![2.0f32; N];
    let mut c = vec![0.0f32; N];

    if let Some(dev) = crate::Device::fetch() {
        dev.context().apply(|ctx| {
            let mut lhs = ctx.malloc::<f32>(N);
            let mut rhs = ctx.malloc::<f32>(N);
            let mut ans = ctx.malloc::<f32>(N);

            let queue = ctx.queue();
            queue.memcpy_h2d(&mut lhs, &a);
            queue.memcpy_h2d(&mut rhs, &b);

            let lhs_ptr = lhs.as_ptr();
            let rhs_ptr = rhs.as_ptr();
            let ans_ptr = ans.as_mut_ptr();
            let params: [*const c_void; 3] = [
                &ans_ptr as *const _ as _,
                &lhs_ptr as *const _ as _,
                &rhs_ptr as *const _ as _,
            ];

            ctx.load(&bin)
                .get_kernel(&CString::new("kernel").unwrap())
                .launch(1, 1, 1, params.as_ptr() as _, &queue);

            memcpy_d2h(&mut c, &ans);
        });
        assert_eq!(c, &[3.0f32; N]);
    };
}
