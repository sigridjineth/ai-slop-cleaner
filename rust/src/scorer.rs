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

// The Rust binary only applies universal (language-agnostic) patterns.
// Language-specific pattern evaluation is delegated to an LLM agent
// via patterns-agent.md. The --lang flag is accepted for CLI compatibility
// but has no effect — all loaded patterns are universal scope.
impl Scorer {
    pub fn new(ruleset: Ruleset) -> Self {
        Scorer { ruleset }
    }

    pub fn ruleset(&self) -> &Ruleset {
        &self.ruleset
    }

    pub fn score(&self, text: &str, _lang: &str) -> ScoreResult {
        let mut matches = Vec::new();
        let mut word_matches = Vec::new();
        let lines: Vec<&str> = text.lines().collect();

        // Build byte-offset to line-number lookup for multi-line pattern matching
        let line_starts: Vec<usize> = {
            let mut starts = vec![0usize];
            for (i, b) in text.bytes().enumerate() {
                if b == b'\n' && i + 1 < text.len() {
                    starts.push(i + 1);
                }
            }
            starts
        };
        let byte_to_line = |byte_offset: usize| -> usize {
            match line_starts.binary_search(&byte_offset) {
                Ok(idx) => idx + 1,
                Err(idx) => idx,
            }
        };

        // Score structural patterns (all universal — no language filtering needed)
        for pattern in &self.ruleset.patterns {
            let is_multiline =
                pattern.regex.as_str().contains("(?m)") || pattern.regex.as_str().contains("\n");
            if is_multiline {
                for mat in pattern.regex.find_iter(text) {
                    let line_num = byte_to_line(mat.start());
                    let matched = mat.as_str();
                    let first_line = matched.lines().next().unwrap_or(matched);
                    matches.push(MatchResult {
                        pattern_name: pattern.name.clone(),
                        severity: pattern.severity.clone(),
                        weight: pattern.weight,
                        description: pattern.description.clone(),
                        matched_text: first_line.trim().to_string(),
                        line_number: line_num,
                        column_start: mat.start(),
                        column_end: mat.end(),
                    });
                }
            } else {
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
        }

        // Score banned words (all words checked — no language filtering)
        for word_entry in &self.ruleset.words {
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
            detected_lang: "universal".to_string(),
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
    fn test_universal_patterns_only() {
        // After removing language-specific regex patterns, only universal patterns remain.
        // Language-specific detection is delegated to LLM agents via patterns-agent.md.
        let ruleset = Ruleset::load_from_dir("rules").expect("Failed to load rules");
        for pattern in &ruleset.patterns {
            assert_eq!(
                pattern.lang_scope, "universal",
                "Pattern '{}' should be universal (got '{}'). Language-specific patterns belong in patterns-agent.md.",
                pattern.name, pattern.lang_scope
            );
        }
    }

    #[test]
    fn test_plus_conjunction() {
        let ruleset = Ruleset::load_from_dir("rules").expect("Failed to load rules");
        let scorer = Scorer::new(ruleset);

        let result = scorer.score("Supports English + Korean detection.", "all");
        let has_plus = result
            .matches
            .iter()
            .any(|m| m.pattern_name == "plus_conjunction");
        assert!(has_plus, "Should detect plus conjunction pattern");
    }

    #[test]
    fn test_plus_conjunction_match_is_tight() {
        let ruleset = Ruleset::load_from_dir("rules").expect("Failed to load rules");
        let scorer = Scorer::new(ruleset);

        let result = scorer.score(
            "The draft joins planning + review as if the plus sign were a conjunction.",
            "all",
        );
        let matched = result
            .matches
            .iter()
            .find(|m| m.pattern_name == "plus_conjunction")
            .map(|m| m.matched_text.as_str());

        assert_eq!(
            matched,
            Some("planning + review"),
            "Plus conjunction should report only the joined terms, not surrounding prose"
        );
    }

    #[test]
    fn test_plus_conjunction_supports_combining_marks() {
        let ruleset = Ruleset::load_from_dir("rules").expect("Failed to load rules");
        let scorer = Scorer::new(ruleset);

        let result = scorer.score("मसौदा लेखन + समीक्षा को एक ही बंधन की तरह रखता है।", "all");
        let matched = result
            .matches
            .iter()
            .find(|m| m.pattern_name == "plus_conjunction")
            .map(|m| m.matched_text.as_str());

        assert_eq!(
            matched,
            Some("लेखन + समीक्षा"),
            "Plus conjunction should include Devanagari combining marks in the matched terms"
        );
    }

    #[test]
    fn test_plus_conjunction_does_not_match_math_or_cplusplus() {
        let ruleset = Ruleset::load_from_dir("rules").expect("Failed to load rules");
        let scorer = Scorer::new(ruleset);

        let result = scorer.score(
            "The expression 3+2 is arithmetic, and C++ is a language.",
            "all",
        );
        let has_plus = result
            .matches
            .iter()
            .any(|m| m.pattern_name == "plus_conjunction");

        assert!(
            !has_plus,
            "Plus conjunction should not flag arithmetic expressions or C++"
        );
    }

    #[test]
    fn test_emoji_detection() {
        let ruleset = Ruleset::load_from_dir("rules").expect("Failed to load rules");
        let scorer = Scorer::new(ruleset);

        let result = scorer.score("This is great! 🚀 Let's ship it! ✨", "all");
        let has_emoji = result
            .matches
            .iter()
            .any(|m| m.pattern_name == "emoji_decoration");
        assert!(has_emoji, "Should detect emoji decoration");
    }

    #[test]
    fn test_em_dash_detection() {
        let ruleset = Ruleset::load_from_dir("rules").expect("Failed to load rules");
        let scorer = Scorer::new(ruleset);

        let result = scorer.score("The tool — which is fast — handles everything.", "all");
        let has_em_dash = result.matches.iter().any(|m| m.pattern_name == "em_dash");
        assert!(has_em_dash, "Should detect em dash decoration");
    }

    #[test]
    fn test_regex_compilation_all_patterns() {
        let ruleset = Ruleset::load_from_dir("rules").expect("Failed to load rules");
        assert!(!ruleset.patterns.is_empty(), "Should load patterns");

        for pattern in &ruleset.patterns {
            assert!(
                !pattern.regex.as_str().is_empty(),
                "Pattern '{}' should have valid regex",
                pattern.name
            );
        }
    }

    #[test]
    fn test_any_language_gets_universal_patterns() {
        // All patterns are universal — any text in any language should be scored
        // without language-specific filtering
        let ruleset = Ruleset::load_from_dir("rules").expect("Failed to load rules");
        let scorer = Scorer::new(ruleset);

        // Korean text with emoji
        let result = scorer.score("한국어 텍스트 🚀 좋습니다", "auto");
        assert!(result
            .matches
            .iter()
            .any(|m| m.pattern_name == "emoji_decoration"));

        // Japanese text with em dash
        let result = scorer.score("日本語のテスト — テストで���", "auto");
        assert!(result.matches.iter().any(|m| m.pattern_name == "em_dash"));

        // Chinese text with plus conjunction
        let result = scorer.score("支持中文 + 英文检���", "auto");
        assert!(result
            .matches
            .iter()
            .any(|m| m.pattern_name == "plus_conjunction"));
    }
}
