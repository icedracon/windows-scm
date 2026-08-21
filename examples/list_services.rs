//! Live probe: enumerate all Win32 services on the local machine via the
//! win32-min-backed FFI shim. Proves the SCM → EnumServicesStatusExW wire is
//! byte-correct (struct layouts, string pointer walk, paging cursor).
//!
//! Usage:
//!
//!   cargo run --example list_services
//!
//! Prints one line per service: `<state> <pid>  <name>  (<display>)`.

use windows_scm::{ScmAccess, ScmHandle, ServiceFilter, ServiceState};

fn state_glyph(s: ServiceState) -> &'static str {
    match s {
        ServiceState::Running => "RUN ",
        ServiceState::Stopped => "STOP",
        ServiceState::StartPending => "STA?",
        ServiceState::StopPending => "STO?",
        ServiceState::Paused => "PAUS",
        _ => "??  ",
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let scm = ScmHandle::open(None, ScmAccess::CONNECT | ScmAccess::ENUMERATE_SERVICE)?;
    let services = scm.enumerate(ServiceFilter::AllWin32)?;

    println!("=== {} services enumerated ===", services.len());
    for s in &services {
        println!(
            "  {} {:>6}  {:<40}  ({})",
            state_glyph(s.current_state),
            s.process_id,
            s.name,
            s.display,
        );
    }

    Ok(())
}
