//! FR-3.3 collision-free matric numbers: DEPT/YYYY/NNNN (sequential per dept+year).
use std::collections::HashMap;
use std::sync::Mutex;

/// In-memory allocator; production backs this with a Postgres SEQUENCE per (dept, year).
pub struct MatricAllocator {
    counters: Mutex<HashMap<String, u32>>,
}

impl MatricAllocator {
    pub fn new() -> Self {
        Self {
            counters: Mutex::new(HashMap::new()),
        }
    }
    pub fn next(&self, dept_code: &str, intake_year: u16) -> String {
        generate_matric_number(dept_code, intake_year, self.claim(dept_code, intake_year))
    }
    fn claim(&self, dept: &str, year: u16) -> u32 {
        let key = format!("{}-{}", dept.to_uppercase(), year);
        let mut m = self.counters.lock().unwrap();
        let c = m.entry(key).or_insert(0);
        *c += 1;
        *c
    }
}

impl Default for MatricAllocator {
    fn default() -> Self {
        Self::new()
    }
}

pub fn generate_matric_number(dept_code: &str, intake_year: u16, seq: u32) -> String {
    format!("{}/{}/{:04}", dept_code.to_uppercase(), intake_year, seq)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sequential_no_collision() {
        let a = MatricAllocator::new();
        assert_eq!(a.next("CSS", 2026), "CSS/2026/0001");
        assert_eq!(a.next("CSS", 2026), "CSS/2026/0002");
        assert_eq!(a.next("CSC", 2026), "CSC/2026/0001"); // per-dept isolation
    }
}
