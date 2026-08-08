//! Local Service Control Manager wrapper.
//!
//! Pre-alpha (0.1.0-dev). See README.md for status and scope.
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
