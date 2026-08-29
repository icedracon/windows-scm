//! Local Service Control Manager wrapper.
//!
//! The 0.2 series provides tested local SCM enumeration and lifecycle
//! workflows. See README.md for current boundaries and unsupported APIs.
//!
//! This crate is Windows-only. On non-Windows targets it compiles to an empty
//! module so downstream `cargo check --all-targets` stays clean.

#![deny(unsafe_op_in_unsafe_fn)]

#[cfg(windows)]
mod access;
#[cfg(windows)]
mod error;
#[cfg(windows)]
mod scm;
#[cfg(windows)]
mod service;
#[cfg(windows)]
mod status;
#[cfg(windows)]
mod util;

#[cfg(windows)]
pub use access::{ScmAccess, ServiceAccess};
#[cfg(windows)]
pub use error::{Error, Result};
#[cfg(windows)]
pub use scm::{ScmHandle, ServiceFilter, ServiceInfo};
#[cfg(windows)]
pub use service::{CreateConfig, ErrorControl, Service, ServiceType, StartType};
#[cfg(windows)]
pub use status::{ControlsAccepted, ServiceState, ServiceStatus};
