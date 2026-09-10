//! Import haikus from a JSON array of raw haiku text, as used by e.g.
//! <https://github.com/remy/haiku/blob/master/static/db.json>: each array
//! element is a single string with 3 newline-separated lines.
//!
//! Entries that don't parse as a valid 5-7-5 `Haiku` are skipped rather
//! than treated as an error, since real-world haiku collections mix in
//! looser or mis-punctuated verse.

use thiserror::Error;

use crate::domain::haiku::Haiku;

#[derive(Debug, Error)]
pub enum ImportError {
    #[error("could not parse import file as a JSON array of haiku strings: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct ImportReport {
    pub imported: Vec<Haiku>,
    pub skipped: Vec<String>,
}

pub fn parse(json: &str) -> Result<ImportReport, ImportError> {
    let entries: Vec<String> = serde_json::from_str(json)?;
    let mut report = ImportReport::default();
    for entry in entries {
        match Haiku::parse(&entry) {
            Ok(haiku) => report.imported.push(haiku),
            Err(_) => report.skipped.push(entry),
        }
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_array_imports_nothing() {
        let report = parse("[]").unwrap();
        assert_eq!(report.imported, Vec::new());
        assert_eq!(report.skipped, Vec::<String>::new());
    }

    #[test]
    fn valid_haiku_is_imported() {
        let json = r#"["An old silent pond\nA frog jumps into the pond\nSplash! Silence again"]"#;
        let report = parse(json).unwrap();
        assert_eq!(report.imported.len(), 1);
        assert_eq!(report.imported[0].lines[0], "An old silent pond");
        assert!(report.skipped.is_empty());
    }

    #[test]
    fn invalid_syllable_count_is_skipped_not_erred() {
        let json = r#"["too short\nstill not seventeen\nway way off"]"#;
        let report = parse(json).unwrap();
        assert!(report.imported.is_empty());
        assert_eq!(
            report.skipped,
            vec!["too short\nstill not seventeen\nway way off".to_string()]
        );
    }

    #[test]
    fn mixed_entries_are_classified_independently() {
        let json = r#"[
            "An old silent pond\nA frog jumps into the pond\nSplash! Silence again",
            "nope",
            "I walked to the store\nWanted a little table\nWhole apple loved much"
        ]"#;
        let report = parse(json).unwrap();
        assert_eq!(report.imported.len(), 2);
        assert_eq!(report.skipped, vec!["nope".to_string()]);
    }

    #[test]
    fn entries_missing_three_lines_are_skipped() {
        let json = r#"["only one line", ""]"#;
        let report = parse(json).unwrap();
        assert!(report.imported.is_empty());
        assert_eq!(report.skipped.len(), 2);
    }

    #[test]
    fn non_array_json_is_a_json_error() {
        assert!(matches!(parse("{}"), Err(ImportError::Json(_))));
    }

    #[test]
    fn malformed_json_is_a_json_error() {
        assert!(matches!(parse("not json"), Err(ImportError::Json(_))));
    }
}
