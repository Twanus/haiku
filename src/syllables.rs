/// Count syllables in a single English word.
///
/// Heuristic: count vowel groups, then apply three common English spelling
/// corrections on top:
/// - a silent trailing `e` ("like" -> 1, not 2) ...
/// - ... unless it's a syllabic `-le` after a consonant ("table", "little"),
///   which restores the syllable a plain silent-`e` rule would drop
///   (this also correctly leaves "whole" alone, since the letter before
///   its `-le` is the vowel `o`, not a consonant).
/// - a silent `-ed` after a consonant other than `t`/`d` ("walked" -> 1,
///   but "wanted"/"landed" keep their extra syllable).
///
/// Still not perfect (no dictionary, no stress rules), but noticeably
/// better than raw vowel-group counting for a haiku checker.
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
    let chars: Vec<char> = letters.chars().collect();

    let mut n = 0i32;
    let mut prev_vowel = false;
    for &c in &chars {
        let vowel = is_vowel(c);
        if vowel && !prev_vowel {
            n += 1;
        }
        prev_vowel = vowel;
    }

    if letters.ends_with('e') {
        n -= 1;
    }

    if chars.len() > 2 && letters.ends_with("le") && !is_vowel(chars[chars.len() - 3]) {
        n += 1;
    }

    if chars.len() > 2 && letters.ends_with("ed") {
        let before = chars[chars.len() - 3];
        if !is_vowel(before) && before != 't' && before != 'd' {
            n -= 1;
        }
    }

    n.max(1) as u32
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

    #[test]
    fn silent_trailing_e() {
        for word in ["like", "hope", "fire", "the", "be"] {
            assert_eq!(count(word), 1, "{word}");
        }
    }

    #[test]
    fn syllabic_le_after_consonant() {
        for word in ["table", "little", "apple", "able"] {
            assert_eq!(count(word), 2, "{word}");
        }
    }

    #[test]
    fn le_after_vowel_is_not_syllabic() {
        // The "e" before "-le" here is a vowel, not a consonant, so this
        // is an ordinary silent trailing `e`, not a syllabic `-le`.
        assert_eq!(count("whole"), 1);
    }

    #[test]
    fn silent_ed_after_consonant() {
        for word in ["walked", "jumped", "loved", "played"] {
            assert_eq!(count(word), 1, "{word}");
        }
    }

    #[test]
    fn voiced_ed_after_t_or_d() {
        for word in ["wanted", "landed", "needed", "trusted"] {
            assert_eq!(count(word), 2, "{word}");
        }
    }

    #[test]
    fn multi_syllable_words() {
        assert_eq!(count("beautiful"), 3);
        assert_eq!(count("haiku"), 2);
        assert_eq!(count("silent"), 2);
    }

    #[test]
    fn count_line_sums_words() {
        assert_eq!(count_line("an old silent pond"), 5);
    }
}
