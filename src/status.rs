//! Service status types.

use windows::Win32::System::Services::SERVICE_STATUS_PROCESS;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ServiceState {
    Stopped = 0x1,
    StartPending = 0x2,
    StopPending = 0x3,
    Running = 0x4,
    ContinuePending = 0x5,
    PausePending = 0x6,
    Paused = 0x7,
    Unknown = 0xffff_ffff,
}

impl ServiceState {
    pub fn from_raw(v: u32) -> Self {
        match v {
            0x1 => Self::Stopped,
            0x2 => Self::StartPending,
            0x3 => Self::StopPending,
            0x4 => Self::Running,
            0x5 => Self::ContinuePending,
            0x6 => Self::PausePending,
            0x7 => Self::Paused,
            _ => Self::Unknown,
        }
    }
}

/// Bitmask of controls the service currently accepts (SERVICE_ACCEPT_*).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ControlsAccepted(pub u32);

impl ControlsAccepted {
    pub const STOP: u32 = 0x0000_0001;
    pub const PAUSE_CONTINUE: u32 = 0x0000_0002;
    pub const SHUTDOWN: u32 = 0x0000_0004;
    pub const PARAMCHANGE: u32 = 0x0000_0008;
    pub const NETBINDCHANGE: u32 = 0x0000_0010;
    pub const PRESHUTDOWN: u32 = 0x0000_0100;

    pub fn accepts_stop(self) -> bool {
        self.0 & Self::STOP != 0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ServiceStatus {
    pub service_type: u32,
    pub current_state: ServiceState,
    pub controls_accepted: ControlsAccepted,
    pub win32_exit_code: u32,
    pub service_specific_exit_code: u32,
    pub check_point: u32,
    pub wait_hint: u32,
    pub process_id: u32,
    pub service_flags: u32,
}

impl From<SERVICE_STATUS_PROCESS> for ServiceStatus {
    fn from(s: SERVICE_STATUS_PROCESS) -> Self {
        Self {
            service_type: s.dwServiceType.0,
            current_state: ServiceState::from_raw(s.dwCurrentState.0),
            controls_accepted: ControlsAccepted(s.dwControlsAccepted),
            win32_exit_code: s.dwWin32ExitCode,
            service_specific_exit_code: s.dwServiceSpecificExitCode,
            check_point: s.dwCheckPoint,
            wait_hint: s.dwWaitHint,
            process_id: s.dwProcessId,
            service_flags: s.dwServiceFlags.0,
        }
    }
}
