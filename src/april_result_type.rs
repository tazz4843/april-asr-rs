use std::ffi::c_uint;
use std::fmt::Formatter;

#[repr(u32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AprilResultType {
    Unknown = 0,
    /// Specifies that the result is only partial, and a future call will
    /// contain much of the same text but updated
    RecognitionPartial = 1,
    /// Specifies that the result is final. Future calls will start from
    /// empty and will not contain any of the given text.
    RecognitionFinal = 2,
    /// If in non-synchronous mode, this may be called when the internal
    /// audio buffer is full and processing can't keep up.
    ///
    /// If this the value in the callback, the tokens vector will be empty.
    ErrorCantKeepUp = 3,
    /// Specifies that there has been some silence. Will not be called repeatedly.
    ///
    /// If this the value in the callback, the tokens vector will be empty.
    Silence = 4,
    Other(c_uint),
}

impl std::fmt::Display for AprilResultType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AprilResultType::Unknown => f.write_str("unknown"),
            AprilResultType::RecognitionPartial => f.write_str("partially completed"),
            AprilResultType::RecognitionFinal => f.write_str("final result"),
            AprilResultType::ErrorCantKeepUp => f.write_str("can't keep up"),
            AprilResultType::Silence => f.write_str("silence"),
            AprilResultType::Other(res) => {
                write!(f, "other result {}", res)
            }
        }
    }
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
