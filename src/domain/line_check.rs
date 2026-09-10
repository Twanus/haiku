use crate::domain::syllables;

/// Syllable count and per-word breakdown for one candidate haiku line.
pub struct LineCheck {
    pub count: u32,
    pub target: u32,
    pub breakdown: String,
}

impl LineCheck {
    pub fn is_ok(&self) -> bool {
        self.count == self.target
    }
}

pub fn check_line(line: &str, target: u32) -> LineCheck {
    let mut count = 0u32;
    let mut parts = Vec::new();
    for word in line.split_whitespace() {
        let n = syllables::count(word);
        count += n;
        parts.push(format!("{word}({n})"));
    }
    LineCheck {
        count,
        target,
        breakdown: parts.join(" "),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_line_reports_matching_count_and_breakdown() {
        let check = check_line("an old silent pond", 5);
        assert!(check.is_ok());
        assert_eq!(check.count, 5);
        assert_eq!(check.breakdown, "an(1) old(1) silent(2) pond(1)");
    }

    #[test]
    fn check_line_flags_too_few_syllables() {
        let check = check_line("an old silent", 5);
        assert!(!check.is_ok());
        assert_eq!(check.count, 4);
    }

    #[test]
    fn check_line_flags_too_many_syllables() {
        let check = check_line("a frog jumps into the pond and splashes", 7);
        assert!(!check.is_ok());
        assert!(check.count > 7);
    }

    #[test]
    fn check_line_empty_line_is_not_ok() {
        let check = check_line("", 5);
        assert!(!check.is_ok());
        assert_eq!(check.count, 0);
        assert_eq!(check.breakdown, "");
    }
}
