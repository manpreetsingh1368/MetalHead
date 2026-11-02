#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case)]

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendError {
    DeviceNotFound,
    Other(String),
}

impl std::fmt::Display for BackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DeviceNotFound => write!(f, "GPU device not found"),
            Self::Other(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for BackendError {}

pub type Result<T> = std::result::Result<T, BackendError>;

pub mod device;

pub mod prelude {
    pub use crate::device::{GpuDevice, GpuProperties};
}