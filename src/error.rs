use thiserror::Error;

#[derive(Debug, Error)]
pub enum SystemTrayError {
    #[error("Failed to send event")]
    SendError,
    #[error("Failed to create C-style string: {0}")]
    Ffi(#[from] std::ffi::NulError),
    #[error("Failed to poll event: {0}")]
    PollEventError(String),
}