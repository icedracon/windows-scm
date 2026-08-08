use thiserror::Error;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Win32 error {code}: {message}")]
    Win32 { code: u32, message: String },

    #[error("invalid UTF-16 in service field")]
    InvalidUtf16,

    #[error("service did not reach requested state within {timeout_ms} ms")]
    Timeout { timeout_ms: u32 },

    #[error("unsupported: {0}")]
    Unsupported(&'static str),
}

impl From<windows_core::Error> for Error {
    fn from(e: windows_core::Error) -> Self {
        Error::Win32 {
            code: e.code().0 as u32,
            message: e.message().to_string(),
        }
    }
}
