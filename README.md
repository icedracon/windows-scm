# windows-scm

[![Crates.io](https://img.shields.io/crates/v/windows-scm.svg)](https://crates.io/crates/windows-scm)
[![Docs.rs](https://docs.rs/windows-scm/badge.svg)](https://docs.rs/windows-scm)
[![CI](https://github.com/icedracon/windows-scm/actions/workflows/ci.yml/badge.svg)](https://github.com/icedracon/windows-scm/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Safe Rust wrapper around the **local** Windows Service Control Manager (SCM).
Owns its handles, closes them on drop, and turns `OpenSCManagerW` /
`CreateServiceW` / `OpenServiceW` / `StartServiceW` / `ControlService` /
`DeleteService` / `QueryServiceStatusEx` / `EnumServicesStatusExW` into typed
Rust APIs. Roughly **10× cheaper than svcctl-over-DCERPC** for local queries
because it skips the RPC marshalling and named-pipe hop.

## Status

**`0.2` tested companion crate.** Local enumeration, status, control,
creation, and deletion paths are implemented on top of `win32-min`; APIs may
still evolve before 1.0. See the central
[`win32-min` ecosystem map](https://github.com/icedracon/win32-min/blob/master/ECOSYSTEM.md)
for compatibility and maturity information.

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

## Research workflow

Produce a read-only local service inventory with state, process ID, service
name, and display name:

```powershell
cargo run --example list_services
```

The current API intentionally does not claim executable-path or service-ACL
analysis because full configuration queries are not implemented yet. See the
ecosystem's
[`RESEARCH-WORKFLOWS.md`](https://github.com/icedracon/win32-min/blob/master/RESEARCH-WORKFLOWS.md)
for the complete workflow set.

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

- [`win32-min`](https://github.com/icedracon/win32-min) — verified,
  dependency-free Win32 ABI foundation used by this crate.
- [`windows-token`](https://github.com/icedracon/windows-token) — RAII
  tokens + impersonation; useful when the SCM caller wants to hold
  `SeDebugPrivilege` or run under an alternate identity.
- [`windows-lsa`](https://github.com/icedracon/windows-lsa) — LSA ticket
  cache access, for tools that install a service and then act as the target
  principal.
- [`windows-sspi-shim`](https://github.com/icedracon/windows-sspi-shim) —
  SSPI Negotiate ergonomics, for the SMB / DCERPC side of the same tooling.

Together these cover identity, authentication, and local administration
workflows for Windows security research and defensive tooling.

## Dependencies

- `win32-min >= 0.1.2, < 0.2` with only `services` enabled.
- `thiserror` 2 for the public error taxonomy.
- No async runtime, serialization framework, or generated Windows bindings.

## License

MIT © 2026 [zevs](https://github.com/icedracon)
