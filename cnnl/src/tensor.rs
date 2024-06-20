use crate::bindings::cnnlTensorDescriptor_t;
use std::ptr::null_mut;

#[repr(transparent)]
pub struct Tensor(cnnlTensorDescriptor_t);

impl Tensor {
    #[inline]
    pub fn new() -> Self {
        let mut ptr = null_mut();
        cnnl!(cnnlCreateTensorDescriptor(&mut ptr));
        Tensor(ptr)
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
    let _tensor = Tensor::new();
    println!("test passed");
}
