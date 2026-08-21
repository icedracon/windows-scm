//! `Service`: owning wrapper for a service SC_HANDLE.

use crate::access::ServiceAccess;
use crate::error::{Error, Result};
use crate::scm::ScmHandle;
use crate::status::{ServiceState, ServiceStatus};
use crate::util::{opt_to_wide, to_wide};

use core::ffi::c_void;
use core::mem::size_of;
use win32_min::foundation::PCWSTR;
use win32_min::services::{
    CloseServiceHandle, ControlService, CreateServiceW, DeleteService, QueryServiceStatusEx,
    StartServiceW, SC_HANDLE, SC_STATUS_TYPE, SERVICE_CONTROL_STOP, SERVICE_STATUS,
    SERVICE_STATUS_PROCESS,
};

/// `dwServiceType` values for `CreateServiceW`.
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum ServiceType {
    Win32OwnProcess = 0x0000_0010,
    Win32ShareProcess = 0x0000_0020,
    KernelDriver = 0x0000_0001,
    FileSystemDriver = 0x0000_0002,
}

/// `dwStartType` values for `CreateServiceW`.
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum StartType {
    Boot = 0,
    System = 1,
    Auto = 2,
    Manual = 3,
    Disabled = 4,
}

/// `dwErrorControl` values for `CreateServiceW`.
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum ErrorControl {
    Ignore = 0,
    Normal = 1,
    Severe = 2,
    Critical = 3,
}

/// Parameters for `create_service` / `ScmHandle::create_service`.
#[derive(Debug, Clone)]
pub struct CreateConfig<'a> {
    pub name: &'a str,
    pub display: &'a str,
    pub bin_path: &'a str,
    pub service_type: ServiceType,
    pub start_type: StartType,
    pub error_control: ErrorControl,
    /// `Some((account, password))` — account like `"NT AUTHORITY\\LocalService"` or `".\\user"`.
    /// `None` runs as LocalSystem.
    pub account: Option<(&'a str, &'a str)>,
}

/// Owning wrapper around a service SC_HANDLE.
pub struct Service {
    handle: SC_HANDLE,
}

unsafe impl Send for Service {}
unsafe impl Sync for Service {}

impl Service {
    pub(crate) fn from_raw(handle: SC_HANDLE) -> Self {
        Self { handle }
    }

    /// Start the service with optional argv (Unicode strings).
    ///
    /// Passing an empty slice is equivalent to `net start` with no arguments.
    pub fn start(&self, args: &[&str]) -> Result<()> {
        if args.is_empty() {
            let ok = unsafe { StartServiceW(self.handle, 0, core::ptr::null()) };
            if ok == 0 {
                return Err(Error::from_last_os_error());
            }
            return Ok(());
        }
        // Own the wide buffers, then build a parallel Vec<PCWSTR> pointing into them.
        let wides: Vec<Vec<u16>> = args.iter().map(|s| to_wide(s)).collect();
        let ptrs: Vec<PCWSTR> = wides.iter().map(|w| PCWSTR(w.as_ptr())).collect();
        let ok = unsafe { StartServiceW(self.handle, ptrs.len() as u32, ptrs.as_ptr()) };
        if ok == 0 {
            return Err(Error::from_last_os_error());
        }
        Ok(())
    }

    /// Send SERVICE_CONTROL_STOP and (best-effort) poll until Stopped or timeout.
    ///
    /// `timeout_ms` == 0 sends the control and returns immediately without polling.
    pub fn stop(&self, timeout_ms: u32) -> Result<ServiceStatus> {
        let mut st: SERVICE_STATUS = unsafe { core::mem::zeroed() };
        let ok = unsafe { ControlService(self.handle, SERVICE_CONTROL_STOP, &mut st) };
        if ok == 0 {
            return Err(Error::from_last_os_error());
        }

        if timeout_ms == 0 {
            return self.query_status();
        }

        // Polling loop — 100ms cadence, capped at `timeout_ms`.
        let start = std::time::Instant::now();
        loop {
            let cur = self.query_status()?;
            if cur.current_state == ServiceState::Stopped {
                return Ok(cur);
            }
            if start.elapsed().as_millis() as u32 >= timeout_ms {
                return Err(Error::Timeout { timeout_ms });
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    /// Mark the service for deletion. Actual deletion occurs when the last
    /// handle closes and no processes reference it.
    pub fn delete(self) -> Result<()> {
        let ok = unsafe { DeleteService(self.handle) };
        if ok == 0 {
            return Err(Error::from_last_os_error());
        }
        Ok(())
    }

    /// Query current status (`QueryServiceStatusEx` with `SC_STATUS_PROCESS_INFO`).
    pub fn query_status(&self) -> Result<ServiceStatus> {
        let mut buf: SERVICE_STATUS_PROCESS = unsafe { core::mem::zeroed() };
        let mut needed: u32 = 0;
        let ok = unsafe {
            QueryServiceStatusEx(
                self.handle,
                SC_STATUS_TYPE::SC_STATUS_PROCESS_INFO,
                &mut buf as *mut SERVICE_STATUS_PROCESS as *mut c_void,
                size_of::<SERVICE_STATUS_PROCESS>() as u32,
                &mut needed,
            )
        };
        if ok == 0 {
            return Err(Error::from_last_os_error());
        }
        Ok(buf.into())
    }
}

impl Drop for Service {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            let _ = unsafe { CloseServiceHandle(self.handle) };
        }
    }
}

impl ScmHandle {
    /// Create a new service. Requires `ScmAccess::CREATE_SERVICE` on the SCM handle.
    ///
    /// Returns a `Service` handle opened with `SERVICE_ALL_ACCESS`.
    pub fn create_service(&self, cfg: &CreateConfig<'_>) -> Result<Service> {
        let name_w = to_wide(cfg.name);
        let display_w = to_wide(cfg.display);
        let bin_w = to_wide(cfg.bin_path);
        let account_w = opt_to_wide(cfg.account.map(|(u, _)| u));
        let password_w = opt_to_wide(cfg.account.map(|(_, p)| p));

        let account_ptr = account_w
            .as_ref()
            .map(|v| PCWSTR(v.as_ptr()))
            .unwrap_or(PCWSTR::NULL);
        let password_ptr = password_w
            .as_ref()
            .map(|v| PCWSTR(v.as_ptr()))
            .unwrap_or(PCWSTR::NULL);

        let h = unsafe {
            CreateServiceW(
                self.handle,
                PCWSTR(name_w.as_ptr()),
                PCWSTR(display_w.as_ptr()),
                ServiceAccess::ALL_ACCESS.bits(),
                cfg.service_type as u32,
                cfg.start_type as u32,
                cfg.error_control as u32,
                PCWSTR(bin_w.as_ptr()),
                PCWSTR::NULL, // load-order group
                core::ptr::null_mut(), // tag id out-param
                PCWSTR::NULL, // dependencies (double-null-terminated)
                account_ptr,
                password_ptr,
            )
        };
        if h.is_null() {
            return Err(Error::from_last_os_error());
        }
        Ok(Service::from_raw(h))
    }
}
