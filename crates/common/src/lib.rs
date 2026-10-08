//! Shared NUC-compliant domain logic for Newgate ERP-LMS.
//!
//! Covers: FR-2.1 cohort binding, FR-2.2 carryover, FR-2.3 prerequisites,
//! FR-2.4 CGPA/spillover/NUC 1.5x cap, FR-3.3 matric numbers,
//! FR-1.1 JWT claims + additive RBAC, FR-4.1/4.3 money + webhook helpers.

pub mod academic;
pub mod auth;
pub mod config;
pub mod finance;
pub mod matric;

pub use academic::{
    carryovers_locked, check_prerequisites, compute_cgpa, evaluate_standing, AcademicStanding,
    Cohort, CourseEnrollment, Grade, Semester, Standing,
};
pub use auth::{Claims, Permission};
pub use finance::{verify_hmac_sha512, Money};
pub use matric::generate_matric_number;
