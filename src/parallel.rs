// Backward compatibility layer for parallel module
//
// This module re-exports all functionality from the new modular structure
// to maintain backward compatibility with existing code.

// Re-export generic parallel utilities (currently unused but available for future use)
// pub use crate::utils::parallel::*;

// Re-export AWS-specific adapters
pub use crate::aws::parallel_aws::*;
