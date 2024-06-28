use crate::bindings::cnnlDataType_t;
use cndrv::AsRaw;
use digit_layout::DigitLayout;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(transparent)]
pub struct DataType(cnnlDataType_t);

impl AsRaw for DataType {
    type Raw = cnnlDataType_t;
    #[inline]
    unsafe fn as_raw(&self) -> Self::Raw {
        self.0
    }
}

impl From<DigitLayout> for DataType {
    fn from(dl: DigitLayout) -> Self {
        use cnnlDataType_t::*;
        use digit_layout::types::*;
        Self(match dl {
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
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(transparent)]
pub struct DataTypeNotMatch(pub cnnlDataType_t);

impl TryFrom<DataType> for DigitLayout {
    type Error = DataTypeNotMatch;
    fn try_from(dt: DataType) -> Result<Self, Self::Error> {
        use cnnlDataType_t::*;
        use digit_layout::types::*;
        match dt.0 {
            CNNL_DTYPE_HALF => Ok(F16),
            CNNL_DTYPE_BFLOAT16 => Ok(BF16),
            CNNL_DTYPE_FLOAT => Ok(F32),
            CNNL_DTYPE_DOUBLE => Ok(F64),
            CNNL_DTYPE_INT8 => Ok(I8),
            CNNL_DTYPE_INT16 => Ok(I16),
            CNNL_DTYPE_INT32 => Ok(I32),
            CNNL_DTYPE_INT64 => Ok(I64),
            CNNL_DTYPE_UINT8 => Ok(U8),
            CNNL_DTYPE_UINT16 => Ok(U16),
            CNNL_DTYPE_UINT32 => Ok(U32),
            CNNL_DTYPE_UINT64 => Ok(U64),
            CNNL_DTYPE_BOOL => Ok(BOOL),
            dt => Err(DataTypeNotMatch(dt)),
        }
    }
}
