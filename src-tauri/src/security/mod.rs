//! Security module providing input validation, sanitization, and protection mechanisms
//!
//! This module addresses critical security vulnerabilities by providing:
//! - Comprehensive input validation
//! - SQL injection prevention
//! - Path traversal protection
//! - Rate limiting
//! - Authentication middleware

pub mod auth_middleware;
pub mod input_validation;
pub mod rate_limiter;
pub mod secure_storage;

pub use auth_middleware::*;
pub use input_validation::*;
pub use rate_limiter::*;
pub use secure_storage::*;
