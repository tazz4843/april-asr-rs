use std::ffi::NulError;
use std::fmt::Formatter;
use std::str::Utf8Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    CString(NulError),
    /// Rust bindings got a nullptr from the C side
    NullPtr,
    /// Invalid UTF-8 was detected in a string from April
    InvalidUtf8(Utf8Error),
    /// Empty audio buffer was passed to feed_pcm16
    EmptyAudio,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::CString(e) => {
                write!(f, "failed to get CString: {}", e)
            }
            Error::NullPtr => f.write_str("got null ptr from april"),
            Error::InvalidUtf8(e) => {
                write!(f, "got invalid UTF-8 in a string from april: {}", e)
            }
            Error::EmptyAudio => f.write_str("attempting to feed an empty audio sample to april"),
        }
    }
}

impl std::error::Error for Error {}

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
