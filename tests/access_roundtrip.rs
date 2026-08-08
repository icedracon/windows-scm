//! Structural test: ScmAccess/ServiceAccess bit-or round-trips, and
//! ServiceState raw conversion is total.

use windows_scm::{ScmAccess, ServiceAccess};

#[cfg(windows)]
use windows_scm::ServiceState;

#[test]
fn scm_access_bitor() {
    let a = ScmAccess::CONNECT | ScmAccess::ENUMERATE_SERVICE;
    assert_eq!(a.bits(), 0x0001 | 0x0004);
    assert_eq!(ScmAccess::ALL_ACCESS.bits(), 0x000F_003F);
}

#[test]
fn service_access_bitor() {
    let a = ServiceAccess::START | ServiceAccess::STOP | ServiceAccess::QUERY_STATUS;
    assert_eq!(a.bits(), 0x0010 | 0x0020 | 0x0004);
    assert_eq!(ServiceAccess::ALL_ACCESS.bits(), 0x000F_01FF);
}

#[cfg(windows)]
#[test]
fn service_state_roundtrip() {
    for raw in [0x1u32, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7] {
        let s = ServiceState::from_raw(raw);
        assert_eq!(s as u32, raw, "state {raw:#x} did not round-trip");
    }
    assert_eq!(ServiceState::from_raw(0xdead), ServiceState::Unknown);
}
