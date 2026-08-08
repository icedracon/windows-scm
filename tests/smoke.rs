//! Smoke test: enumerate should return >0 services on any Windows box.

#![cfg(windows)]

use windows_scm::{ScmAccess, ScmHandle, ServiceFilter};

#[test]
fn enumerate_returns_services() {
    let scm = ScmHandle::open(None, ScmAccess::CONNECT | ScmAccess::ENUMERATE_SERVICE)
        .expect("OpenSCManagerW(local) — is the SCM running?");
    let all = scm
        .enumerate(ServiceFilter::AllWin32)
        .expect("EnumServicesStatusExW failed");
    assert!(!all.is_empty(), "expected >0 services on this box");
    // Sanity: names & display strings should not all be empty.
    assert!(all.iter().any(|s| !s.name.is_empty()));
    assert!(all.iter().any(|s| !s.display.is_empty()));
}
