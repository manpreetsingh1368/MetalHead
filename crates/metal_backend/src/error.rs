
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendError {
    DeviceNotFound,
    DeviceInitialization(String),
    UnsupportedBackend(String),
    AllocationFailed(String),
    KernelCompilation(String),
    KernelExecution(String),
    SynchronizationFailed(String),
    Other(String),
}

impl std::fmt::Display for BackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DeviceNotFound =>
                write!(f, "GPU device not found"),
            Self::DeviceInitialization(s) =>
                write!(f, "GPU initialization failed: {s}"),
            Self::UnsupportedBackend(s) =>
                write!(f, "Unsupported GPU backend: {s}"),
            Self::AllocationFailed(s) =>
                write!(f, "GPU allocation failed: {s}"),
            Self::KernelCompilation(s) =>
                write!(f, "Kernel compilation failed: {s}"),
            Self::KernelExecution(s) =>
                write!(f, "Kernel execution failed: {s}"),
            Self::SynchronizationFailed(s) =>
                write!(f, "GPU synchronization failed: {s}"),
            Self::Other(s) => write!(f, "{s}"),
        }
    }
}

impl std::error::Error for BackendError {}

pub type Result<T> = std::result::Result<T, BackendError>;
