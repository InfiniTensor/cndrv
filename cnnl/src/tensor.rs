use crate::bindings::{cnnlDataType_t, cnnlTensorDescriptor_t};
use digit_layout::DigitLayout;
use std::ptr::null_mut;

#[repr(transparent)]
pub struct Tensor(cnnlTensorDescriptor_t);

impl Tensor {
    #[inline]
    pub fn new(dl: DigitLayout, shape: &[i64], strides: &[i64]) -> Self {
        assert_eq!(shape.len(), strides.len());
        let mut desc = null_mut();
        cnnl!(cnnlCreateTensorDescriptor(&mut desc));
        cnnl!(cnnlSetTensorDescriptorEx_v2(
            desc,
            cnnlTensorLayout_t::CNNL_LAYOUT_ARRAY,
            convert_dt(dl),
            shape.len() as _,
            shape.as_ptr(),
            strides.as_ptr(),
        ));
        Tensor(desc)
    }
}

impl Drop for Tensor {
    #[inline]
    fn drop(&mut self) {
        cnnl!(cnnlDestroyTensorDescriptor(self.0));
    }
}

fn convert_dt(dl: DigitLayout) -> cnnlDataType_t {
    use cnnlDataType_t::*;
    use digit_layout::types::*;
    match dl {
        F16 => CNNL_DTYPE_HALF,
        BF16 => CNNL_DTYPE_BFLOAT16,
        F32 => CNNL_DTYPE_FLOAT,
        F64 => CNNL_DTYPE_DOUBLE,
        I8 => CNNL_DTYPE_INT8,
        I16 => CNNL_DTYPE_INT16,
        I32 => CNNL_DTYPE_INT32,
        I64 => CNNL_DTYPE_INT64,
        U8 => CNNL_DTYPE_UINT8,
        U16 => CNNL_DTYPE_UINT16,
        U32 => CNNL_DTYPE_UINT32,
        U64 => CNNL_DTYPE_UINT64,
        BOOL => CNNL_DTYPE_BOOL,
        _ => CNNL_DTYPE_INVALID,
    }
}

#[test]
fn test() {
    use digit_layout::types::F16;
    let _tensor = Tensor::new(F16, &[2, 3, 4], &[12, 4, 1]);
    println!("test passed");
}
