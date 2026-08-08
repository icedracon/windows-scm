# windows-scm

**STATUS: pre-alpha (0.1.0-dev)** — API surface being defined; some paths return placeholder values with clear TODO comments. Not for production.

Thin, safe wrapper around the local Windows Service Control Manager (SCM) — `OpenSCManagerW`, `CreateServiceW`, `OpenServiceW`, `StartServiceW`, `ControlService`, `DeleteService`, `QueryServiceStatusEx`, `EnumServicesStatusExW`.

## Why

Roughly **~10x cheaper than svcctl-over-DCERPC** for local queries because it skips the RPC marshalling and named-pipe hop. Use `dcerpc`/`svcctl` for remote hosts; use this crate on the local box.

## Scope

- Local-only (v0.1). `machine: None` opens the local SCM. `Some("\\\\host")` is accepted by the underlying API but not the target of this crate — remote SCMR is a separate concern.
- `cfg(windows)` — the crate is empty on non-Windows targets so downstream `cargo check` still works cross-platform.
- Handles are owned; `Drop` closes them.

## Minimal usage

```rust
use windows_scm::{ScmHandle, ScmAccess, ServiceFilter};

let scm = ScmHandle::open(None, ScmAccess::CONNECT | ScmAccess::ENUMERATE_SERVICE)?;
let services = scm.enumerate(ServiceFilter::ActiveWin32)?;
for s in services.iter().take(5) {
    println!("{}  {}  state={:?}", s.name, s.display, s.current_state);
}

let svc = scm.open_service("Spooler", 0x0004 /* SERVICE_QUERY_STATUS */)?;
let st = svc.query_status()?;
println!("Spooler: {:?}", st.current_state);
```

## License

MIT
