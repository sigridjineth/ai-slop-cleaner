use regex::Regex;

use crate::pattern_loader::Ruleset;

/// A single match found in the text.
#[derive(Debug, Clone, serde::Serialize)]
pub struct MatchResult {
    pub pattern_name: String,
    pub severity: String,
    pub weight: f64,
    pub description: String,
    pub matched_text: String,
    pub line_number: usize,
    pub column_start: usize,
    pub column_end: usize,
}

/// The final scoring result.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ScoreResult {
    pub overall_score: f64,
    pub matches: Vec<MatchResult>,
    pub word_matches: Vec<WordMatchResult>,
    pub summary: ScoreSummary,
    pub detected_lang: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct WordMatchResult {
    pub word: String,
    pub replacement: Option<String>,
    pub weight: f64,
    pub line_number: usize,
    pub matched_text: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ScoreSummary {
    pub total_patterns_matched: usize,
    pub total_words_matched: usize,
    pub high_severity_count: usize,
    pub medium_severity_count: usize,
    pub low_severity_count: usize,
}

pub struct Scorer {
    ruleset: Ruleset,
}

/// Detect language from text content.
/// Returns "ko" if >= 5% of characters are Korean (Hangul),
/// "en" otherwise.
fn detect_language(text: &str) -> &'static str {
    let total_chars = text.chars().filter(|c| !c.is_whitespace()).count();
    if total_chars == 0 {
        return "en";
    }
    let korean_chars = text
        .chars()
        .filter(|c| {
            let cp = *c as u32;
            // Hangul Syllables (AC00-D7A3) + Jamo (1100-11FF, 3130-318F)
            (0xAC00..=0xD7A3).contains(&cp)
                || (0x1100..=0x11FF).contains(&cp)
                || (0x3130..=0x318F).contains(&cp)
        })
        .count();
    let ratio = korean_chars as f64 / total_chars as f64;
    if ratio >= 0.05 {
        "ko"
    } else {
        "en"
    }
}

/// Check if a pattern should be included given the resolved language.
/// Uses lang_scope field: universal = always, english = en only, korean = ko only.
fn should_include_pattern(lang_scope: &str, lang: &str) -> bool {
    match lang {
        "all" => true,
        _ => match lang_scope {
            "universal" => true,
            "english" => lang == "en",
            "korean" => lang == "ko",
            _ => true, // unknown scope = include by default
        },
    }
}

/// Check if a banned word should be included given the resolved language.
fn should_include_word(word: &str, lang: &str) -> bool {
    match lang {
        "en" => {
            // Skip Korean banned words (contain Hangul)
            !word.chars().any(|c| {
                let cp = c as u32;
                (0xAC00..=0xD7A3).contains(&cp)
            })
        }
        "ko" => true, // Korean text gets all word checks
        _ => true,
    }
}

impl Scorer {
    pub fn new(ruleset: Ruleset) -> Self {
        Scorer { ruleset }
    }

    pub fn ruleset(&self) -> &Ruleset {
        &self.ruleset
    }

    pub fn score(&self, text: &str, lang: &str) -> ScoreResult {
        let resolved_lang = match lang {
            "auto" => detect_language(text),
            "en" => "en",
            "ko" => "ko",
            _ => "all",
        };

        let mut matches = Vec::new();
        let mut word_matches = Vec::new();
        let lines: Vec<&str> = text.lines().collect();

        // Score structural patterns (filtered by language)
        for pattern in &self.ruleset.patterns {
            if !should_include_pattern(&pattern.lang_scope, resolved_lang) {
                continue;
            }
            for (line_idx, line) in lines.iter().enumerate() {
                for mat in pattern.regex.find_iter(line) {
                    matches.push(MatchResult {
                        pattern_name: pattern.name.clone(),
                        severity: pattern.severity.clone(),
                        weight: pattern.weight,
                        description: pattern.description.clone(),
                        matched_text: mat.as_str().to_string(),
                        line_number: line_idx + 1,
                        column_start: mat.start(),
                        column_end: mat.end(),
                    });
                }
            }
        }

        // Score banned words (filtered by language)
        for word_entry in &self.ruleset.words {
            if !should_include_word(&word_entry.word, resolved_lang) {
                continue;
            }
            let word_regex = Regex::new(&format!(r"(?i)\b{}\b", regex::escape(&word_entry.word)))
                .unwrap_or_else(|_| Regex::new(&regex::escape(&word_entry.word)).unwrap());

            for (line_idx, line) in lines.iter().enumerate() {
                for mat in word_regex.find_iter(line) {
                    word_matches.push(WordMatchResult {
                        word: word_entry.word.clone(),
                        replacement: word_entry.replacement.clone(),
                        weight: word_entry.weight,
                        line_number: line_idx + 1,
                        matched_text: mat.as_str().to_string(),
                    });
                }
            }
        }

        // Calculate overall score (0-100, higher = more slop)
        let pattern_score: f64 = matches.iter().map(|m| m.weight).sum();
        let word_score: f64 = word_matches.iter().map(|m| m.weight).sum();
        let total_score = (pattern_score + word_score).min(100.0);

        let high_count = matches.iter().filter(|m| m.severity == "high").count();
        let medium_count = matches.iter().filter(|m| m.severity == "medium").count();
        let low_count = matches.iter().filter(|m| m.severity == "low").count();

        ScoreResult {
            overall_score: total_score,
            matches,
            word_matches: word_matches.clone(),
            summary: ScoreSummary {
                total_patterns_matched: high_count + medium_count + low_count,
                total_words_matched: word_matches.len(),
                high_severity_count: high_count,
                medium_severity_count: medium_count,
                low_severity_count: low_count,
            },
            detected_lang: resolved_lang.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern_loader::Ruleset;

    fn test_ruleset() -> Ruleset {
        Ruleset {
            patterns: vec![crate::pattern_loader::BannedPattern {
                name: "test_pattern".to_string(),
                lang_scope: "universal".to_string(),
                severity: "high".to_string(),
                weight: 2.0,
                regex: Regex::new(r"(?i)\btest\b").unwrap(),
                description: "Test pattern".to_string(),
            }],
            words: vec![crate::pattern_loader::BannedWord {
                word: "badword".to_string(),
                replacement: Some("goodword".to_string()),
                weight: 1.5,
            }],
        }
    }

    #[test]
    fn test_basic_scoring() {
        let scorer = Scorer::new(test_ruleset());
        let result = scorer.score("This is a test with badword.", "all");
        assert!(result.overall_score > 0.0);
        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.word_matches.len(), 1);
    }

    #[test]
    fn test_no_matches() {
        let scorer = Scorer::new(test_ruleset());
        let result = scorer.score("This is clean text.", "all");
        assert_eq!(result.overall_score, 0.0);
        assert_eq!(result.matches.len(), 0);
        assert_eq!(result.word_matches.len(), 0);
    }

    #[test]
    fn test_korean_patterns() {
        let ruleset = Ruleset::load_from_dir("rules").unwrap_or_else(|_| test_ruleset());
        let scorer = Scorer::new(ruleset);

        let result = scorer.score("이것은 테스트가 아니라 예시입니다.", "all");
        let has_korean_redefinition = result
            .matches
            .iter()
            .any(|m| m.pattern_name.contains("a_not_b_korean"));
        assert!(
            has_korean_redefinition,
            "Should detect Korean redefinition pattern"
        );
    }

    #[test]
    fn test_english_redefinition() {
        let ruleset = Ruleset::load_from_dir("rules").unwrap_or_else(|_| test_ruleset());
        let scorer = Scorer::new(ruleset);

        let result = scorer.score("This is not just a test, it is a revolution.", "all");
        let has_redefinition = result
            .matches
            .iter()
            .any(|m| m.pattern_name.contains("redefinition") || m.pattern_name.contains("a_not_b"));
        assert!(
            has_redefinition,
            "Should detect English redefinition pattern"
        );
    }

    #[test]
    fn test_regex_compilation_all_patterns() {
        let ruleset = Ruleset::load_from_dir("rules").expect("Failed to load rules");
        assert!(!ruleset.patterns.is_empty(), "Should load patterns");

        for pattern in &ruleset.patterns {
            assert!(
                pattern.regex.as_str().len() > 0,
                "Pattern '{}' should have valid regex",
                pattern.name
            );
        }
    }

    #[test]
    fn test_unicode_korean_ranges() {
        let ruleset = Ruleset::load_from_dir("rules").expect("Failed to load rules");
        let scorer = Scorer::new(ruleset);

        let korean_text = "이 문제에 있어서 해결책을 찾아야 합니다. 이것을 통해 알 수 있습니다.";
        let result = scorer.score(korean_text, "all");

        let has_korean_match = result
            .matches
            .iter()
            .any(|m| m.pattern_name.starts_with("ko_"));
        assert!(has_korean_match, "Should detect Korean AI slop patterns");
    }

    #[test]
    fn test_lang_filter_en_skips_korean() {
        let ruleset = Ruleset::load_from_dir("rules").expect("Failed to load rules");
        let scorer = Scorer::new(ruleset);

        let result = scorer.score("This is a normal English README file.", "en");
        // With lang=en, korean-scoped patterns should be skipped,
        // but universal patterns (bold, em dash, bullet block, etc.) should still apply
        let has_korean_only = result
            .matches
            .iter()
            .any(|m| m.pattern_name.starts_with("ko_A") || m.pattern_name.starts_with("ko_D"));
        assert!(
            !has_korean_only,
            "lang=en should not produce Korean-only pattern matches"
        );
    }

    #[test]
    fn test_lang_auto_english() {
        let result_lang =
            super::detect_language("This is a purely English document with no Korean.");
        assert_eq!(result_lang, "en");
    }

    #[test]
    fn test_lang_auto_korean() {
        let result_lang = super::detect_language("이것은 한국어 문서입니다.");
        assert_eq!(result_lang, "ko");
    }
}
