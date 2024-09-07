use std::ffi::c_uint;

#[repr(u32)]
pub enum AprilResultType {
    Unknown = 0,
    RecognitionPartial = 1,
    RecognitionFinal = 2,
    ErrorCantKeepUp = 3,
    Silence = 4,
    Other(c_uint),
}

impl From<c_uint> for AprilResultType {
    fn from(value: c_uint) -> Self {
        match value {
            0 => Self::Unknown,
            1 => Self::RecognitionPartial,
            2 => Self::RecognitionFinal,
            3 => Self::ErrorCantKeepUp,
            4 => Self::Silence,
            r => Self::Other(r),
        }
    }
}
