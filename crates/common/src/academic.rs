//! FR-2.x Academic Engine — pure, DB-free, fully unit-tested.
//! Encodes NUC rules: dynamic cohorts, carryover lock, prerequisites,
//! CGPA standing, spillover, 1.5x max-duration Senate cap.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Semester {
    Semester1,
    Semester2,
}

impl Semester {
    pub fn as_suffix(&self) -> &'static str {
        match self {
            Semester::Semester1 => "Semester1",
            Semester::Semester2 => "Semester2",
        }
    }
}

/// Permanently bound intake cohort, e.g. `2026-Semester2` (FR-2.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Cohort {
    pub year: u16,
    pub semester: Semester,
    /// Normal program length in years (e.g. 4). NUC cap = ceil(1.5x).
    pub program_years: u8,
}

impl Cohort {
    pub fn parse(s: &str, program_years: u8) -> Result<Self, String> {
        let (y, sem) = s.split_once('-').ok_or("cohort must be YYYY-SemesterN")?;
        let year: u16 = y.parse().map_err(|_| "bad cohort year")?;
        let semester = match sem {
            "Semester1" => Semester::Semester1,
            "Semester2" => Semester::Semester2,
            _ => return Err("cohort semester must be Semester1|Semester2".into()),
        };
        if !(1..=6).contains(&program_years) {
            return Err("program_years must be 1..=6".into());
        }
        Ok(Self {
            year,
            semester,
            program_years,
        })
    }

    pub fn label(&self) -> String {
        format!("{}-{}", self.year, self.semester.as_suffix())
    }

    /// Academic sessions elapsed since intake (0 = intake session).
    /// Semester2 intake joins mid-session: session 0 is still their first.
    pub fn sessions_elapsed(&self, current_year: u16) -> i32 {
        current_year as i32 - self.year as i32
    }

    /// NUC maximum allowed sessions before mandatory Senate review.
    pub fn max_sessions(&self) -> u32 {
        ((self.program_years as f32) * 1.5).ceil() as u32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Grade {
    A,
    B,
    C,
    D,
    E,
    F,
}

impl Grade {
    pub fn points(&self) -> f64 {
        match self {
            Grade::A => 5.0,
            Grade::B => 4.0,
            Grade::C => 3.0,
            Grade::D => 2.0,
            Grade::E => 1.0,
            Grade::F => 0.0,
        }
    }
    pub fn failed(&self) -> bool {
        matches!(self, Grade::F)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseEnrollment {
    pub course_code: String,
    pub credit_units: u32,
    pub grade: Grade,
    /// Lower levels must clear first; used for cart ordering.
    pub level: u16, // e.g. 100, 200, 300, 400
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Standing {
    Good,
    /// CGPA < 1.00 — pause cohort progression, repeat core courses.
    ProbationRepeat,
    Spillover(u8), // SPILLOVER_1, _2, ...
    /// Exceeded 1.5x normal duration.
    SenateReview,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcademicStanding {
    pub cgpa: f64,
    pub total_units: u32,
    pub standing: Standing,
}

/// FR-2.2: every failed (F) course not later cleared must be
/// pre-populated and locked before higher-level/elective registration.
pub fn carryovers_locked(history: &[CourseEnrollment]) -> Vec<String> {
    // A course is cleared if ANY later/passing record exists for same code.
    let cleared: HashSet<String> = history
        .iter()
        .filter(|e| !e.grade.failed())
        .map(|e| e.course_code.clone())
        .collect();
    let mut locked: Vec<String> = history
        .iter()
        .filter(|e| e.grade.failed() && !cleared.contains(&e.course_code))
        .map(|e| e.course_code.clone())
        .collect();
    locked.sort();
    locked.dedup();
    locked
}

/// FR-2.3: block advanced registration unless prerequisites cleared (non-F).
pub fn check_prerequisites(
    passed_codes: &HashSet<String>,
    course_code: &str,
    prerequisites: &[String],
) -> Result<(), String> {
    let missing: Vec<_> = prerequisites
        .iter()
        .filter(|p| !passed_codes.contains(*p))
        .collect();
    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "cannot register {}: missing prerequisites {:?}",
            course_code, missing
        ))
    }
}

/// GPA helper: weighted by credit units.
pub fn compute_cgpa(enrollments: &[CourseEnrollment]) -> (f64, u32) {
    let units: u32 = enrollments.iter().map(|e| e.credit_units).sum();
    if units == 0 {
        return (0.0, 0);
    }
    let pts: f64 = enrollments
        .iter()
        .map(|e| e.grade.points() * e.credit_units as f64)
        .sum();
    (pts / units as f64, units)
}

/// FR-2.4: session-end standing decision.
pub fn evaluate_standing(
    cohort: Cohort,
    current_year: u16,
    enrollments: &[CourseEnrollment],
    all_required_cleared: bool,
) -> AcademicStanding {
    let (cgpa, total_units) = compute_cgpa(enrollments);
    let elapsed = cohort.sessions_elapsed(current_year).max(0) as u32;
    let normal = cohort.program_years as u32;
    let max = cohort.max_sessions();

    let standing = if elapsed >= max && !all_required_cleared {
        Standing::SenateReview
    } else if elapsed >= normal && !all_required_cleared {
        Standing::Spillover((elapsed - normal + 1).min(9) as u8)
    } else if cgpa < 1.0 && total_units > 0 {
        Standing::ProbationRepeat
    } else {
        Standing::Good
    };
    AcademicStanding {
        cgpa,
        total_units,
        standing,
    }
}

/// Validate a registration cart: carryovers must come first (FR-2.2).
pub fn validate_cart(
    history: &[CourseEnrollment],
    cart: &[String],
    prereq_map: &HashMap<String, Vec<String>>,
) -> Result<(), String> {
    let locked = carryovers_locked(history);
    for c in &locked {
        if !cart.contains(c) {
            return Err(format!("carryover {} must be in cart and locked first", c));
        }
    }
    // Locked courses must precede all others in cart order.
    if !locked.is_empty() {
        let first_non_locked = cart.iter().position(|c| !locked.contains(c));
        let last_locked = cart.iter().rposition(|c| locked.contains(c));
        if let (Some(f), Some(l)) = (first_non_locked, last_locked) {
            if l > f {
                return Err("carryovers must be pre-populated before higher-level courses".into());
            }
        }
    }
    let passed: HashSet<String> = history
        .iter()
        .filter(|e| !e.grade.failed())
        .map(|e| e.course_code.clone())
        .collect();
    for code in cart {
        if let Some(pre) = prereq_map.get(code) {
            check_prerequisites(&passed, code, pre)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn enr(code: &str, units: u32, grade: Grade, level: u16) -> CourseEnrollment {
        CourseEnrollment {
            course_code: code.into(),
            credit_units: units,
            grade,
            level,
        }
    }

    #[test]
    fn cohort_roundtrip_and_sem2() {
        let c = Cohort::parse("2026-Semester2", 4).unwrap();
        assert_eq!(c.label(), "2026-Semester2");
        assert_eq!(c.max_sessions(), 6); // 1.5x4
        assert_eq!(
            Cohort::parse("2026-Semester1", 5).unwrap().max_sessions(),
            8
        ); // ceil(7.5)
    }

    #[test]
    fn carryover_lock_and_clear() {
        let h = vec![
            enr("CSC101", 3, Grade::F, 100),
            enr("MTH101", 3, Grade::C, 100),
        ];
        assert_eq!(carryovers_locked(&h), vec!["CSC101"]);
        // retake passed → cleared
        let h2 = vec![
            enr("CSC101", 3, Grade::F, 100),
            enr("CSC101", 3, Grade::D, 100),
        ];
        assert!(carryovers_locked(&h2).is_empty());
    }

    #[test]
    fn cart_rejects_missing_carryover_and_bad_order() {
        let h = vec![enr("CSC101", 3, Grade::F, 100)];
        assert!(validate_cart(&h, &["CSC201".into()], &HashMap::new()).is_err());
        // wrong order: higher-level first
        assert!(validate_cart(&h, &["CSC201".into(), "CSC101".into()], &HashMap::new()).is_err());
        assert!(validate_cart(&h, &["CSC101".into(), "CSC201".into()], &HashMap::new()).is_ok());
    }

    #[test]
    fn probation_spillover_senate() {
        let cohort = Cohort::parse("2020-Semester1", 4).unwrap();
        let bad = vec![
            enr("CSC101", 3, Grade::F, 100),
            enr("MTH101", 3, Grade::F, 100),
        ];
        // same session, CGPA 0 → probation
        let s = evaluate_standing(cohort, 2020, &bad, false);
        assert_eq!(s.standing, Standing::ProbationRepeat);
        // past normal duration → spillover
        let s = evaluate_standing(cohort, 2024, &bad, false);
        assert_eq!(s.standing, Standing::Spillover(1));
        // past 1.5x cap (6 sessions) → senate
        let s = evaluate_standing(cohort, 2026, &bad, false);
        assert_eq!(s.standing, Standing::SenateReview);
        // cleared → good even late
        let good = vec![enr("CSC101", 3, Grade::B, 100)];
        let s = evaluate_standing(cohort, 2026, &good, true);
        assert_eq!(s.standing, Standing::Good);
    }
}
