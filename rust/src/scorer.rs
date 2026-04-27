use crate::pattern_loader::Ruleset;

/// A single structural pattern match found in the text.
#[derive(Debug, Clone, serde::Serialize)]
pub struct MatchResult {
    pub pattern_name: String,
    pub severity: String,
    pub weight: f64,
    pub matched_text: String,
    pub line_number: usize,
}

pub struct Scorer {
    ruleset: Ruleset,
}

// The Rust binary only applies universal (language-agnostic) structural
// patterns. It intentionally does not score, classify, or make holistic
// judgments; that work is delegated to an LLM agent that can read the full
// text plus these raw structural matches. The --lang flag is accepted for CLI
// compatibility but has no effect — all loaded patterns are universal scope.
impl Scorer {
    pub fn new(ruleset: Ruleset) -> Self {
        Scorer { ruleset }
    }

    pub fn ruleset(&self) -> &Ruleset {
        &self.ruleset
    }

    pub fn analyze(&self, text: &str, _lang: &str) -> Vec<MatchResult> {
        let mut matches = Vec::new();
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

        // Match structural patterns (all universal — no language filtering needed)
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
                        matched_text: first_line.trim().to_string(),
                        line_number: line_num,
                    });
                }
            } else {
                for (line_idx, line) in lines.iter().enumerate() {
                    for mat in pattern.regex.find_iter(line) {
                        matches.push(MatchResult {
                            pattern_name: pattern.name.clone(),
                            severity: pattern.severity.clone(),
                            weight: pattern.weight,
                            matched_text: mat.as_str().to_string(),
                            line_number: line_idx + 1,
                        });
                    }
                }
            }
        }

        matches
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern_loader::Ruleset;
    use regex::Regex;

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
    fn test_basic_analysis_matches_structural_patterns_only() {
        let scorer = Scorer::new(test_ruleset());
        let matches = scorer.analyze("This is a test with badword.", "all");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].pattern_name, "test_pattern");
    }

    #[test]
    fn test_banned_words_do_not_produce_matches() {
        let scorer = Scorer::new(test_ruleset());
        let matches = scorer.analyze("This text only contains badword.", "all");
        assert_eq!(
            matches.len(),
            0,
            "The Rust matcher should only emit structural pattern matches"
        );
    }

    #[test]
    fn test_no_matches() {
        let scorer = Scorer::new(test_ruleset());
        let matches = scorer.analyze("This is clean text.", "all");
        assert_eq!(matches.len(), 0);
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

        let matches = scorer.analyze("Supports English + Korean detection.", "all");
        let has_plus = matches.iter().any(|m| m.pattern_name == "plus_conjunction");
        assert!(has_plus, "Should detect plus conjunction pattern");
    }

    #[test]
    fn test_plus_conjunction_match_is_tight() {
        let ruleset = Ruleset::load_from_dir("rules").expect("Failed to load rules");
        let scorer = Scorer::new(ruleset);

        let matches = scorer.analyze(
            "The draft joins planning + review as if the plus sign were a conjunction.",
            "all",
        );
        let matched = matches
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

        let matches = scorer.analyze("मसौदा लेखन + समीक्षा को एक ही बंधन की तरह रखता है।", "all");
        let matched = matches
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

        let matches = scorer.analyze(
            "The expression 3+2 is arithmetic, and C++ is a language.",
            "all",
        );
        let has_plus = matches.iter().any(|m| m.pattern_name == "plus_conjunction");

        assert!(
            !has_plus,
            "Plus conjunction should not flag arithmetic expressions or C++"
        );
    }

    #[test]
    fn test_emoji_detection() {
        let ruleset = Ruleset::load_from_dir("rules").expect("Failed to load rules");
        let scorer = Scorer::new(ruleset);

        let matches = scorer.analyze("This is great! 🚀 Let's ship it! ✨", "all");
        let has_emoji = matches.iter().any(|m| m.pattern_name == "emoji_decoration");
        assert!(has_emoji, "Should detect emoji decoration");
    }

    #[test]
    fn test_em_dash_detection() {
        let ruleset = Ruleset::load_from_dir("rules").expect("Failed to load rules");
        let scorer = Scorer::new(ruleset);

        let matches = scorer.analyze("The tool — which is fast — handles everything.", "all");
        let has_em_dash = matches.iter().any(|m| m.pattern_name == "em_dash");
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
        // All patterns are universal — any text in any language should be matched
        // without language-specific filtering
        let ruleset = Ruleset::load_from_dir("rules").expect("Failed to load rules");
        let scorer = Scorer::new(ruleset);

        // Korean text with emoji
        let matches = scorer.analyze("한국어 텍스트 🚀 좋습니다", "auto");
        assert!(matches.iter().any(|m| m.pattern_name == "emoji_decoration"));

        // Japanese text with em dash
        let matches = scorer.analyze("日本語のテスト — テストで���", "auto");
        assert!(matches.iter().any(|m| m.pattern_name == "em_dash"));

        // Chinese text with plus conjunction
        let matches = scorer.analyze("支持中文 + 英文检���", "auto");
        assert!(matches.iter().any(|m| m.pattern_name == "plus_conjunction"));
    }
}
