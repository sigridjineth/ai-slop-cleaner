use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// A single banned pattern entry loaded from markdown tables.
#[derive(Debug, Clone)]
pub struct BannedPattern {
    pub name: String,
    pub severity: String,
    pub weight: f64,
    pub regex: Regex,
    pub description: String,
}

/// A single banned word entry loaded from markdown tables.
#[derive(Debug, Clone)]
pub struct BannedWord {
    pub word: String,
    pub replacement: Option<String>,
    pub weight: f64,
}

/// Container for all dynamically loaded rules.
#[derive(Debug, Clone)]
pub struct Ruleset {
    pub patterns: Vec<BannedPattern>,
    pub words: Vec<BannedWord>,
}

/// Agent-readable pattern definition for LLM-based multilingual detection.
#[derive(Debug, Clone)]
pub struct AgentPattern {
    pub name: String,
    pub severity: String,
    pub weight: f64,
    pub description: String,
    pub examples: Vec<String>,
    pub multilingual_note: Option<String>,
}

impl Ruleset {
    /// Load rules from the given directory (expects `banned-patterns.md` and `banned-words.md`).
    pub fn load_from_dir<P: AsRef<Path>>(dir: P) -> Result<Self, Box<dyn std::error::Error>> {
        let dir = dir.as_ref();
        let patterns = Self::load_patterns_from_file(dir.join("banned-patterns.md"))?;
        let words = Self::load_words_from_file(dir.join("banned-words.md"))?;
        Ok(Ruleset { patterns, words })
    }

    /// Load agent-readable pattern definitions from markdown.
    /// These are used for LLM-based multilingual detection instead of regex.
    pub fn load_agent_patterns_from_file<P: AsRef<Path>>(
        path: P,
    ) -> Result<Vec<AgentPattern>, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(&path)?;
        let mut patterns = Vec::new();
        let mut current: Option<AgentPattern> = None;
        let mut in_examples = false;
        let mut in_multilingual_note = false;

        for line in content.lines() {
            let trimmed = line.trim();

            // Pattern header: ### pattern_name
            if trimmed.starts_with("### ") {
                // Save previous pattern if exists
                if let Some(p) = current.take() {
                    patterns.push(p);
                }
                let name = trimmed[4..].trim().to_string();
                current = Some(AgentPattern {
                    name,
                    severity: String::new(),
                    weight: 1.0,
                    description: String::new(),
                    examples: Vec::new(),
                    multilingual_note: None,
                });
                in_examples = false;
                in_multilingual_note = false;
                continue;
            }

            if let Some(ref mut pat) = current {
                // Severity: - **Severity:** value
                if trimmed.starts_with("- **Severity:**") {
                    pat.severity = trimmed[15..].trim().to_string();
                    continue;
                }
                // Weight: - **Weight:** value
                if trimmed.starts_with("- **Weight:**") {
                    pat.weight = trimmed[13..].trim().parse::<f64>().unwrap_or(1.0);
                    continue;
                }
                // Description: - **Description:** value
                if trimmed.starts_with("- **Description:**") {
                    pat.description = trimmed[18..].trim().to_string();
                    continue;
                }
                // Multilingual note
                if trimmed.starts_with("- **Multilingual note:**") {
                    pat.multilingual_note = Some(trimmed[24..].trim().to_string());
                    in_multilingual_note = true;
                    in_examples = false;
                    continue;
                }
                // Examples section
                if trimmed == "- **Examples:**" || trimmed == "**Examples:**" {
                    in_examples = true;
                    in_multilingual_note = false;
                    continue;
                }
                // Example items
                if in_examples && (trimmed.starts_with("- ") || trimmed.starts_with("  - ")) {
                    let example = trimmed.trim_start_matches("- ").trim_start_matches("  - ").trim().to_string();
                    if !example.is_empty() {
                        pat.examples.push(example);
                    }
                    continue;
                }
                // Note items
                if in_multilingual_note && trimmed.starts_with("  ") {
                    let note = trimmed.trim().to_string();
                    if !note.is_empty() {
                        let existing = pat.multilingual_note.get_or_insert_with(String::new);
                        if !existing.is_empty() {
                            existing.push(' ');
                        }
                        existing.push_str(&note);
                    }
                    continue;
                }
                // If we hit a non-indented line that's not a field, reset state
                if !trimmed.is_empty() && !trimmed.starts_with('-') && !trimmed.starts_with("**") {
                    in_examples = false;
                    in_multilingual_note = false;
                }
            }
        }

        // Don't forget the last pattern
        if let Some(p) = current {
            patterns.push(p);
        }

        Ok(patterns)
    }

    /// Split a markdown table row into cells, respecting backtick-quoted content.
    /// This handles | characters inside regex patterns that are wrapped in backticks.
    fn split_table_row(row: &str) -> Vec<String> {
        let mut cells = Vec::new();
        let mut current = String::new();
        let mut in_backticks = false;
        let chars: Vec<char> = row.trim().chars().collect();
        let mut i = 0;
        
        while i < chars.len() {
            let c = chars[i];
            
            if c == '`' {
                in_backticks = !in_backticks;
                current.push(c);
            } else if c == '|' && !in_backticks {
                // Split point - save current cell and start new one
                let trimmed = current.trim().to_string();
                if !trimmed.is_empty() || !cells.is_empty() {
                    cells.push(trimmed);
                }
                current = String::new();
            } else {
                current.push(c);
            }
            
            i += 1;
        }
        
        // Don't forget the last cell
        let trimmed = current.trim().to_string();
        if !trimmed.is_empty() {
            cells.push(trimmed);
        }
        
        cells
    }

    fn load_patterns_from_file<P: AsRef<Path>>(
        path: P,
    ) -> Result<Vec<BannedPattern>, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(&path)?;
        let mut patterns = Vec::new();
        let mut in_table = false;
        let mut headers: Vec<String> = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("|") {
                let cells = Self::split_table_row(trimmed);

                if !in_table {
                    // First row: headers
                    headers = cells.iter().map(|s| s.to_lowercase()).collect();
                    in_table = true;
                    continue;
                }

                if cells.len() >= 2 && cells[0].starts_with("-") {
                    // Separator row, skip
                    continue;
                }

                if cells.len() >= headers.len() {
                    let mut map: HashMap<String, String> = HashMap::new();
                    for (i, cell) in cells.iter().enumerate() {
                        if let Some(h) = headers.get(i) {
                            map.insert(h.clone(), cell.to_string());
                        }
                    }

                    let name = map.get("name").cloned().unwrap_or_default();
                    let severity = map.get("severity").cloned().unwrap_or_default();
                    let weight = map
                        .get("weight")
                        .and_then(|w| w.parse::<f64>().ok())
                        .unwrap_or(1.0);
                    let regex_str = map.get("regex").cloned().unwrap_or_default();
                    let description = map.get("description").cloned().unwrap_or_default();

                    // Remove surrounding backticks from regex if present
                    let regex_str = regex_str.trim_matches('`');
                    
                    // Convert escaped pipes back to real pipes for regex alternation
                    // In markdown we write \| to avoid table conflicts, but regex needs |
                    let regex_str = regex_str.replace("\\|", "|");

                    if let Ok(regex) = Regex::new(&regex_str) {
                        patterns.push(BannedPattern {
                            name,
                            severity,
                            weight,
                            regex,
                            description,
                        });
                    } else {
                        eprintln!("Warning: Failed to compile regex for pattern '{}': {}", name, regex_str);
                    }
                }
            } else {
                in_table = false;
                headers.clear();
            }
        }

        Ok(patterns)
    }

    fn load_words_from_file<P: AsRef<Path>>(
        path: P,
    ) -> Result<Vec<BannedWord>, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(&path)?;
        let mut words = Vec::new();
        let mut in_table = false;
        let mut headers: Vec<String> = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("|") {
                let cells = Self::split_table_row(trimmed);

                if !in_table {
                    headers = cells.iter().map(|s| s.to_lowercase()).collect();
                    in_table = true;
                    continue;
                }

                if cells.len() >= 2 && cells[0].starts_with("-") {
                    continue;
                }

                if cells.len() >= headers.len() {
                    let mut map: HashMap<String, String> = HashMap::new();
                    for (i, cell) in cells.iter().enumerate() {
                        if let Some(h) = headers.get(i) {
                            map.insert(h.clone(), cell.to_string());
                        }
                    }

                    let word = map.get("word").cloned().unwrap_or_default();
                    let replacement = map.get("replacement").filter(|s| !s.is_empty()).cloned();
                    let weight = map
                        .get("weight")
                        .and_then(|w| w.parse::<f64>().ok())
                        .unwrap_or(1.0);

                    if !word.is_empty() {
                        words.push(BannedWord {
                            word,
                            replacement,
                            weight,
                        });
                    }
                }
            } else {
                in_table = false;
                headers.clear();
            }
        }

        Ok(words)
    }
}
