use move_core_types::vm_status::StatusCode;

/// Result of the execution.
#[derive(Debug)]
pub struct VmResult {
    /// Execution status code read from the MoveVM
    pub status_code: StatusCode,
    /// Optional error message.
    pub error_message: Option<String>,
    /// Gas used.
    pub gas_used: u64,
}

impl VmResult {
    /// Create a new VmResult.
    pub fn new(status_code: StatusCode, error_message: Option<String>, gas_used: u64) -> Self {
        Self {
            status_code,
            error_message,
            gas_used,
        }
    }

    /// Check if the execution was successful.
    #[inline]
    pub fn is_ok(&self) -> bool {
        self.status_code == StatusCode::EXECUTED
    }

    /// Check if the execution failed.
    #[inline]
    pub fn is_err(&self) -> bool {
        !self.is_ok()
    }
}
