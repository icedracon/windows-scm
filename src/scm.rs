//! `ScmHandle`: owning wrapper for an SC_HANDLE opened via `OpenSCManagerW`.

use crate::access::{ScmAccess, ServiceAccess};
use crate::error::Result;
use crate::service::Service;
use crate::status::ServiceState;
use crate::util::{opt_to_wide, to_wide, wide_ptr_to_string};

use windows::core::PCWSTR;
use windows::Win32::System::Services::{
    CloseServiceHandle, EnumServicesStatusExW, OpenSCManagerW, OpenServiceW, ENUM_SERVICE_STATE,
    ENUM_SERVICE_STATUS_PROCESSW, ENUM_SERVICE_TYPE, SC_ENUM_PROCESS_INFO, SC_HANDLE,
    SERVICE_ACTIVE, SERVICE_DRIVER, SERVICE_INACTIVE, SERVICE_STATE_ALL, SERVICE_WIN32,
};

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
            Self::AllIncludingDrivers => SERVICE_WIN32.0 | SERVICE_DRIVER.0,
            _ => SERVICE_WIN32.0,
        }
    }
    fn service_state(self) -> u32 {
        match self {
            Self::ActiveWin32 => SERVICE_ACTIVE.0,
            Self::InactiveWin32 => SERVICE_INACTIVE.0,
            Self::AllWin32 | Self::AllIncludingDrivers => SERVICE_STATE_ALL.0,
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
// long as we do not hand out references that outlive `self`. All the API
// surface takes `&self` and returns owned data or child handles that keep no
// reference into `self`.
unsafe impl Send for ScmHandle {}
unsafe impl Sync for ScmHandle {}

impl ScmHandle {
    /// Open the SCM on `machine` (None = local).
    pub fn open(machine: Option<&str>, access: ScmAccess) -> Result<Self> {
        let machine_w = opt_to_wide(machine);
        let machine_ptr = machine_w
            .as_ref()
            .map(|v| PCWSTR(v.as_ptr()))
            .unwrap_or(PCWSTR::null());
        // Second arg (database name) NULL => SERVICES_ACTIVE_DATABASE.
        let handle = unsafe { OpenSCManagerW(machine_ptr, PCWSTR::null(), access.bits())? };
        Ok(Self { handle })
    }

    /// Open an existing service by name.
    pub fn open_service(&self, name: &str, access: u32) -> Result<Service> {
        let name_w = to_wide(name);
        let handle = unsafe { OpenServiceW(self.handle, PCWSTR(name_w.as_ptr()), access)? };
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

        let sizing = unsafe {
            EnumServicesStatusExW(
                self.handle,
                SC_ENUM_PROCESS_INFO,
                ENUM_SERVICE_TYPE(stype),
                ENUM_SERVICE_STATE(sstate),
                None,
                &mut bytes_needed,
                &mut services_returned,
                Some(&mut resume_handle),
                PCWSTR::null(),
            )
        };

        // Expected: ERROR_MORE_DATA (0xEA / 234) when buffer too small.
        if sizing.is_ok() && bytes_needed == 0 {
            return Ok(Vec::new());
        }
        if let Err(e) = sizing {
            const ERROR_MORE_DATA: i32 = 0x800700EA_u32 as i32;
            if e.code().0 != ERROR_MORE_DATA {
                return Err(e.into());
            }
        }

        // Real pass. `ENUM_SERVICE_STATUS_PROCESSW` embeds pointers into this
        // buffer, so the buffer must live as long as we read those pointers.
        let mut buf: Vec<u8> = vec![0u8; bytes_needed as usize];
        resume_handle = 0;

        unsafe {
            EnumServicesStatusExW(
                self.handle,
                SC_ENUM_PROCESS_INFO,
                ENUM_SERVICE_TYPE(stype),
                ENUM_SERVICE_STATE(sstate),
                Some(&mut buf[..]),
                &mut bytes_needed,
                &mut services_returned,
                Some(&mut resume_handle),
                PCWSTR::null(),
            )?
        };

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
                service_type: e.ServiceStatusProcess.dwServiceType.0,
                current_state: ServiceState::from_raw(e.ServiceStatusProcess.dwCurrentState.0),
                process_id: e.ServiceStatusProcess.dwProcessId,
            });
        }
        Ok(out)
    }
}

impl Drop for ScmHandle {
    fn drop(&mut self) {
        if !self.handle.is_invalid() {
            // Ignore close errors — nothing sensible to do on drop.
            let _ = unsafe { CloseServiceHandle(self.handle) };
        }
    }
}
