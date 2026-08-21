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

impl Error {
    /// Build an `Error::Win32` from the current thread's `GetLastError()` value.
    pub fn from_last_os_error() -> Self {
        let code = unsafe { win32_min::foundation::GetLastError() };
        let message = std::io::Error::from_raw_os_error(code as i32).to_string();
        Error::Win32 { code, message }
    }
}
