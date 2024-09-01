use std::ffi::NulError;
use std::str::Utf8Error;

pub type Result<T> = std::result::Result<T, Error>;

pub enum Error {
    CString(NulError),
    /// Rust bindings got a nullptr from the C side
    NullPtr,
    /// Invalid UTF-8 was detected in a string from April
    InvalidUtf8(Utf8Error),
}

impl From<NulError> for Error {
    fn from(err: NulError) -> Self {
        Self::CString(err)
    }
}

impl From<Utf8Error> for Error {
    fn from(err: Utf8Error) -> Self {
        Self::InvalidUtf8(err)
    }
}
