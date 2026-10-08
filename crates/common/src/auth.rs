//! FR-1.1: stateless JWT + additive RBAC permission vectors.
use serde::{Deserialize, Serialize};

/// Additive permission bitflags (stored as u64 vector element).
pub mod perm {
    pub const STUDENT_READ: u64 = 1 << 0;
    pub const COURSE_REGISTER: u64 = 1 << 1;
    pub const EXAM_TAKE: u64 = 1 << 2;
    pub const STAFF_WRITE: u64 = 1 << 3;
    pub const HOD_APPROVE: u64 = 1 << 4;
    pub const BURSARY_LEDGER: u64 = 1 << 5;
    pub const ADMISSIONS_SCREEN: u64 = 1 << 6;
    pub const CLINIC_READ: u64 = 1 << 7;
    pub const CLINIC_WRITE: u64 = 1 << 8;
    pub const ADMIN_ALL: u64 = 1 << 63;
}

pub type Permission = u64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,            // user id
    pub cohort: Option<String>, // e.g. 2026-Semester2
    pub perms: Vec<Permission>, // additive RBAC vectors
    pub exp: usize,
    pub iss: String,
}

impl Claims {
    pub fn has(&self, perm: Permission) -> bool {
        self.perms
            .iter()
            .any(|p| p & perm == perm || *p == perm::ADMIN_ALL)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn additive_rbac() {
        let c = Claims {
            sub: "u1".into(),
            cohort: Some("2026-Semester2".into()),
            perms: vec![perm::STUDENT_READ | perm::COURSE_REGISTER],
            exp: 0,
            iss: "newgate-erp".into(),
        };
        assert!(c.has(perm::STUDENT_READ));
        assert!(!c.has(perm::BURSARY_LEDGER));
    }
}
