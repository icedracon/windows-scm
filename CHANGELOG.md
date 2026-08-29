# Changelog

## Unreleased

- Documented the runnable read-only local service inventory workflow and its
  current configuration-query boundary.
- Added scheduled RustSec advisory auditing and weekly dependency monitoring.

## 0.2.1 - 2026-08-29

- Corrected stale pre-alpha and dependency documentation after the 0.2 FFI
  migration.
- Declared `win32-min` 0.1.2 as the minimum compatible ABI foundation.
- Declared Rust 1.85 as the MSRV and added CI, research-oriented package
  metadata, and an AI-readable index.

## 0.2.0 - 2026-08-27

- Migrated the Win32 FFI layer from generated Windows bindings to
  `win32-min`.
