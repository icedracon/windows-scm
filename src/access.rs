//! Access-mask bitflags for SCM and Service handles.
//!
//! Values mirror `winsvc.h`. Wrapping in newtypes lets callers write
//! `ScmAccess::CONNECT | ScmAccess::ENUMERATE_SERVICE` without pulling in a
//! bitflags dependency.

use core::ops::{BitOr, BitOrAssign};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScmAccess(pub u32);

impl ScmAccess {
    pub const CONNECT: Self = Self(0x0001);
    pub const CREATE_SERVICE: Self = Self(0x0002);
    pub const ENUMERATE_SERVICE: Self = Self(0x0004);
    pub const LOCK: Self = Self(0x0008);
    pub const QUERY_LOCK_STATUS: Self = Self(0x0010);
    pub const MODIFY_BOOT_CONFIG: Self = Self(0x0020);
    pub const ALL_ACCESS: Self = Self(0x000F_003F);

    pub const fn bits(self) -> u32 {
        self.0
    }
}

impl BitOr for ScmAccess {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}
impl BitOrAssign for ScmAccess {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServiceAccess(pub u32);

impl ServiceAccess {
    pub const QUERY_CONFIG: Self = Self(0x0001);
    pub const CHANGE_CONFIG: Self = Self(0x0002);
    pub const QUERY_STATUS: Self = Self(0x0004);
    pub const ENUMERATE_DEPENDENTS: Self = Self(0x0008);
    pub const START: Self = Self(0x0010);
    pub const STOP: Self = Self(0x0020);
    pub const PAUSE_CONTINUE: Self = Self(0x0040);
    pub const INTERROGATE: Self = Self(0x0080);
    pub const USER_DEFINED_CONTROL: Self = Self(0x0100);
    pub const DELETE: Self = Self(0x0001_0000);
    pub const ALL_ACCESS: Self = Self(0x000F_01FF);

    pub const fn bits(self) -> u32 {
        self.0
    }
}

impl BitOr for ServiceAccess {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}
impl BitOrAssign for ServiceAccess {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}
