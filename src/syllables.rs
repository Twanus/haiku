/// Count syllables in a single English word.
///
/// Heuristic: vowel groups, with a silent trailing `e`. Not perfect;
/// good enough for a haiku checker.
pub fn count(word: &str) -> u32 {
    let letters: String = word
        .chars()
        .filter(|c| c.is_ascii_alphabetic())
        .map(|c| c.to_ascii_lowercase())
        .collect();

    if letters.is_empty() {
        return 0;
    }

    let is_vowel = |c: char| matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'y');
    let mut n = 0u32;
    let mut prev_vowel = false;
    for c in letters.chars() {
        let vowel = is_vowel(c);
        if vowel && !prev_vowel {
            n += 1;
        }
        prev_vowel = vowel;
    }

    if letters.ends_with('e') && n > 1 {
        n -= 1;
    }

    n.max(1)
}

/// Count syllables in a line of words.
pub fn count_line(line: &str) -> u32 {
    line.split_whitespace().map(count).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_is_zero() {
        assert_eq!(count(""), 0);
        assert_eq!(count("..."), 0);
        assert_eq!(count_line(""), 0);
    }

    #[test]
    fn one_syllable_words() {
        for word in ["old", "pond", "frog", "in", "sound"] {
            assert_eq!(count(word), 1, "{word}");
        }
    }
}
