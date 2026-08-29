//! `ScmHandle`: owning wrapper for an SC_HANDLE opened via `OpenSCManagerW`.

use crate::access::{ScmAccess, ServiceAccess};
use crate::error::{Error, Result};
use crate::service::Service;
use crate::status::ServiceState;
use crate::util::{opt_to_wide, to_wide, wide_ptr_to_string};

use core::ffi::c_void;
use win32_min::foundation::PCWSTR;
use win32_min::services::{
    CloseServiceHandle, EnumServicesStatusExW, OpenSCManagerW, OpenServiceW,
    ENUM_SERVICE_STATUS_PROCESSW, SC_ENUM_TYPE, SC_HANDLE, SERVICE_ACTIVE, SERVICE_DRIVER,
    SERVICE_INACTIVE, SERVICE_STATE_ALL, SERVICE_WIN32_OWN_PROCESS, SERVICE_WIN32_SHARE_PROCESS,
};

const SERVICE_WIN32: u32 = SERVICE_WIN32_OWN_PROCESS | SERVICE_WIN32_SHARE_PROCESS;

/// Which services to include in an enumeration call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceFilter {
    /// Active Win32 (user-mode) services only.
    ActiveWin32,
    /// Inactive Win32 services only.
    InactiveWin32,
    /// All Win32 services regardless of state.
    AllWin32,
    /// All services and drivers, active + inactive.
    AllIncludingDrivers,
}

impl ServiceFilter {
    fn service_type(self) -> u32 {
        match self {
            Self::AllIncludingDrivers => SERVICE_WIN32 | SERVICE_DRIVER,
            _ => SERVICE_WIN32,
        }
    }
    fn service_state(self) -> u32 {
        match self {
            Self::ActiveWin32 => SERVICE_ACTIVE,
            Self::InactiveWin32 => SERVICE_INACTIVE,
            Self::AllWin32 | Self::AllIncludingDrivers => SERVICE_STATE_ALL,
        }
    }
}

/// One row from `EnumServicesStatusExW`.
#[derive(Debug, Clone)]
pub struct ServiceInfo {
    pub name: String,
    pub display: String,
    pub service_type: u32,
    pub current_state: ServiceState,
    pub process_id: u32,
}

/// Owning wrapper around an SC_HANDLE opened by `OpenSCManagerW`.
///
/// Handle is closed via `CloseServiceHandle` on `Drop`.
pub struct ScmHandle {
    pub(crate) handle: SC_HANDLE,
}

// SC_HANDLE is a raw kernel handle; owning it here is `Send + Sync`-safe as
// long as we do not hand out references that outlive `self`.
unsafe impl Send for ScmHandle {}
unsafe impl Sync for ScmHandle {}

impl ScmHandle {
    /// Open the SCM on `machine` (None = local).
    pub fn open(machine: Option<&str>, access: ScmAccess) -> Result<Self> {
        let machine_w = opt_to_wide(machine);
        let machine_ptr = machine_w
            .as_ref()
            .map(|v| PCWSTR(v.as_ptr()))
            .unwrap_or(PCWSTR::NULL);
        let handle = unsafe { OpenSCManagerW(machine_ptr, PCWSTR::NULL, access.bits()) };
        if handle.is_null() {
            return Err(Error::from_last_os_error());
        }
        Ok(Self { handle })
    }

    /// Open an existing service by name.
    pub fn open_service(&self, name: &str, access: u32) -> Result<Service> {
        let name_w = to_wide(name);
        let handle = unsafe { OpenServiceW(self.handle, PCWSTR(name_w.as_ptr()), access) };
        if handle.is_null() {
            return Err(Error::from_last_os_error());
        }
        Ok(Service::from_raw(handle))
    }

    /// Convenience wrapper with a typed access mask.
    pub fn open_service_typed(&self, name: &str, access: ServiceAccess) -> Result<Service> {
        self.open_service(name, access.bits())
    }

    /// Enumerate services matching `filter`.
    ///
    /// This calls `EnumServicesStatusExW` twice — first with a zero buffer to
    /// learn the required byte count, then with an allocation of that size.
    pub fn enumerate(&self, filter: ServiceFilter) -> Result<Vec<ServiceInfo>> {
        let stype = filter.service_type();
        let sstate = filter.service_state();

        // Sizing pass.
        let mut bytes_needed: u32 = 0;
        let mut services_returned: u32 = 0;
        let mut resume_handle: u32 = 0;

        let ok = unsafe {
            EnumServicesStatusExW(
                self.handle,
                SC_ENUM_TYPE::SC_ENUM_PROCESS_INFO,
                stype,
                sstate,
                core::ptr::null_mut(),
                0,
                &mut bytes_needed,
                &mut services_returned,
                &mut resume_handle,
                PCWSTR::NULL,
            )
        };

        // Expected: ERROR_MORE_DATA (234) when buffer too small.
        const ERROR_MORE_DATA: u32 = 234;
        if ok != 0 {
            // No error, and if bytes_needed is 0 the service list is empty.
            if bytes_needed == 0 {
                return Ok(Vec::new());
            }
        } else {
            let last = unsafe { win32_min::foundation::GetLastError() };
            if last != ERROR_MORE_DATA {
                return Err(Error::from_last_os_error());
            }
        }

        // Real pass.
        let mut buf: Vec<u8> = vec![0u8; bytes_needed as usize];
        resume_handle = 0;

        let ok = unsafe {
            EnumServicesStatusExW(
                self.handle,
                SC_ENUM_TYPE::SC_ENUM_PROCESS_INFO,
                stype,
                sstate,
                buf.as_mut_ptr() as *mut c_void,
                buf.len() as u32,
                &mut bytes_needed,
                &mut services_returned,
                &mut resume_handle,
                PCWSTR::NULL,
            )
        };
        if ok == 0 {
            return Err(Error::from_last_os_error());
        }

        let count = services_returned as usize;
        let entries_ptr = buf.as_ptr() as *const ENUM_SERVICE_STATUS_PROCESSW;
        let mut out = Vec::with_capacity(count);
        for i in 0..count {
            // SAFETY: EnumServicesStatusExW wrote `count` entries into `buf`.
            let e = unsafe { &*entries_ptr.add(i) };
            let name = unsafe { wide_ptr_to_string(e.lpServiceName.0)? };
            let display = unsafe { wide_ptr_to_string(e.lpDisplayName.0)? };
            out.push(ServiceInfo {
                name,
                display,
                service_type: e.ServiceStatusProcess.dwServiceType,
                current_state: ServiceState::from_raw(e.ServiceStatusProcess.dwCurrentState),
                process_id: e.ServiceStatusProcess.dwProcessId,
            });
        }
        Ok(out)
    }
}

impl Drop for ScmHandle {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            // Ignore close errors — nothing sensible to do on drop.
            let _ = unsafe { CloseServiceHandle(self.handle) };
        }
    }
}
