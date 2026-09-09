use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::syllables;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Haiku {
    pub lines: [String; 3],
}

#[derive(Debug, Error)]
pub enum HaikuError {
    #[error("a haiku must have exactly 3 non-empty lines")]
    WrongLineCount,
    #[error("expected 5-7-5 syllables, got {0}-{1}-{2}")]
    BadSyllables(u32, u32, u32),
}

impl Haiku {
    pub fn new(
        line1: impl Into<String>,
        line2: impl Into<String>,
        line3: impl Into<String>,
    ) -> Result<Self, HaikuError> {
        let haiku = Self {
            lines: [
                line1.into().trim().to_string(),
                line2.into().trim().to_string(),
                line3.into().trim().to_string(),
            ],
        };
        haiku.validate()?;
        Ok(haiku)
    }

    /// Parse three non-empty lines from free text.
    pub fn parse(text: &str) -> Result<Self, HaikuError> {
        let lines: Vec<String> = text
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::to_string)
            .collect();

        if lines.len() != 3 {
            return Err(HaikuError::WrongLineCount);
        }

        Self::new(lines[0].clone(), lines[1].clone(), lines[2].clone())
    }

    pub fn syllable_counts(&self) -> [u32; 3] {
        [
            syllables::count_line(&self.lines[0]),
            syllables::count_line(&self.lines[1]),
            syllables::count_line(&self.lines[2]),
        ]
    }

    pub fn validate(&self) -> Result<(), HaikuError> {
        let [a, b, c] = self.syllable_counts();
        if a == 5 && b == 7 && c == 5 {
            Ok(())
        } else {
            Err(HaikuError::BadSyllables(a, b, c))
        }
    }
}

impl std::fmt::Display for Haiku {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n{}\n{}", self.lines[0], self.lines[1], self.lines[2])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_requires_three_lines() {
        assert!(matches!(
            Haiku::parse("only one"),
            Err(HaikuError::WrongLineCount)
        ));
    }
}
