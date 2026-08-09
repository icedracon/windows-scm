# windows-scm

[![Crates.io](https://img.shields.io/crates/v/windows-scm.svg)](https://crates.io/crates/windows-scm)
[![Docs.rs](https://docs.rs/windows-scm/badge.svg)](https://docs.rs/windows-scm)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Safe Rust wrapper around the **local** Windows Service Control Manager (SCM).
Owns its handles, closes them on drop, and turns `OpenSCManagerW` /
`CreateServiceW` / `OpenServiceW` / `StartServiceW` / `ControlService` /
`DeleteService` / `QueryServiceStatusEx` / `EnumServicesStatusExW` into typed
Rust APIs. Roughly **10× cheaper than svcctl-over-DCERPC** for local queries
because it skips the RPC marshalling and named-pipe hop.

## Status

**`0.1.0-dev`** — pre-alpha, expect breaking changes before `0.1.0`. Part of
the [icedracon](https://github.com/icedracon) Rust offensive-AD ecosystem.

## What it does

Local-only SCM handle + service handle wrappers with typed access masks,
service-type / start-type / state enums, an enumeration filter matching
`EnumServicesStatusExW`'s `SERVICE_STATE` bits, and a `CreateConfig<'a>`
builder for `CreateServiceW`. Non-Windows targets compile to an empty module
so downstream `cargo check --all-targets` stays cross-platform-clean. For
**remote** SCM use `dcerpc` + a `svcctl` client — this crate deliberately
covers only the local box.

## Usage

```rust,no_run
use windows_scm::{ScmHandle, ScmAccess, ServiceAccess, ServiceFilter};

fn main() -> windows_scm::Result<()> {
    let scm = ScmHandle::open(None, ScmAccess::CONNECT | ScmAccess::ENUMERATE_SERVICE)?;

    for s in scm.enumerate(ServiceFilter::ActiveWin32)?.iter().take(5) {
        println!("{:24}  {:?}", s.name, s.current_state);
    }

    let svc = scm.open_service_typed("Spooler", ServiceAccess::QUERY_STATUS)?;
    println!("Spooler: {:?}", svc.query_status()?.current_state);
    Ok(())
}
```

## What works / what does not (this version)

- Working:
  - `ScmHandle::open` / `open_service` / `open_service_typed` / `enumerate`
    with owned handles + `Drop` close.
  - Typed `ScmAccess` / `ServiceAccess` bitflags.
  - `ServiceType` / `StartType` / `ErrorControl` / `ServiceState` /
    `ControlsAccepted` enums matching the Win32 constants.
  - `Service::start(&[args])`, `stop(timeout_ms)`, `query_status`,
    `delete(self)`.
  - `ScmHandle::create_service(&CreateConfig)` — `CreateServiceW`.
  - `EnumServicesStatusExW` — active / inactive / all Win32 services.
- Stubbed / next milestone:
  - `ChangeServiceConfig[2]W` (edit start type, dependencies, description) —
    not yet exposed.
  - `QueryServiceConfigW` — status only, not full config, is returned today.
  - Some `Service` control paths return placeholder values with clear TODO
    comments; see `src/service.rs`.
  - Remote SCMR against `\\host` is out of scope for `0.1` — accepted by the
    underlying API but not the target of this crate.

## Related icedracon crates

- [`windows-token`](https://github.com/icedracon/windows-token) — RAII
  tokens + impersonation; useful when the SCM caller wants to hold
  `SeDebugPrivilege` or run under an alternate identity.
- [`windows-lsa`](https://github.com/icedracon/windows-lsa) — LSA ticket
  cache access, for tools that install a service and then act as the target
  principal.
- [`windows-sspi-shim`](https://github.com/icedracon/windows-sspi-shim) —
  SSPI Negotiate ergonomics, for the SMB / DCERPC side of the same tooling.

Together these enable "run adhammer as yourself" and impersonation-based
lateral-movement tooling without dragging in Impacket.

## License

MIT © 2026 [zevs](https://github.com/icedracon)
