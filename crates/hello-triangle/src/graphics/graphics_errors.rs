use thiserror::Error;

#[derive(Error, Debug)]
pub enum GraphicsError {
    #[error("No valid GPU")]
    NoValidGPU,

    #[error("Invalid GPU")]
    InvalidGPU,
}