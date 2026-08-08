//! Query-only against a benign known service. We do NOT start/stop anything.

#![cfg(windows)]

use windows_scm::{ScmAccess, ScmHandle, ServiceAccess};

#[test]
fn query_spooler_status() {
    let scm = ScmHandle::open(None, ScmAccess::CONNECT).expect("OpenSCManagerW");
    // Try Spooler first; if it's absent (very stripped-down box) fall back to
    // any service the enumerator sees.
    let svc = match scm.open_service_typed("Spooler", ServiceAccess::QUERY_STATUS) {
        Ok(s) => s,
        Err(_) => {
            let scm2 =
                ScmHandle::open(None, ScmAccess::CONNECT | ScmAccess::ENUMERATE_SERVICE).unwrap();
            let all = scm2
                .enumerate(windows_scm::ServiceFilter::AllWin32)
                .expect("enumerate");
            let name = all.first().expect("at least one service").name.clone();
            scm.open_service_typed(&name, ServiceAccess::QUERY_STATUS)
                .expect("OpenServiceW fallback")
        }
    };
    let st = svc.query_status().expect("QueryServiceStatusEx");
    // service_type must be non-zero for any real service row.
    assert_ne!(st.service_type, 0, "service_type=0 is not a valid entry");
}
