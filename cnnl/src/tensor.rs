use crate::{bindings::cnnlTensorDescriptor_t, DataType};
use cndrv::AsRaw;
use std::ptr::null_mut;

#[repr(transparent)]
pub struct Tensor(cnnlTensorDescriptor_t);

impl AsRaw for Tensor {
    type Raw = cnnlTensorDescriptor_t;
    #[inline]
    unsafe fn as_raw(&self) -> Self::Raw {
        self.0
    }
}

impl Tensor {
    #[inline]
    pub fn new(dt: DataType, shape: &[i64], strides: &[i64]) -> Self {
        assert_eq!(shape.len(), strides.len());
        let mut desc = null_mut();
        cnnl!(cnnlCreateTensorDescriptor(&mut desc));
        cnnl!(cnnlSetTensorDescriptorEx_v2(
            desc,
            cnnlTensorLayout_t::CNNL_LAYOUT_ARRAY,
            dt.as_raw(),
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

#[test]
fn test() {
    use digit_layout::types::F16;
    let _tensor = Tensor::new(F16.into(), &[2, 3, 4], &[12, 4, 1]);
    println!("test passed");
}
