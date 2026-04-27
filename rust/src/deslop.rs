use crate::pattern_loader::Ruleset;
use crate::scorer::{MatchResult, Scorer};
use reqwest::blocking::Client;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use std::time::Duration;

const OPENAI_MODEL: &str = "gpt-4o-mini";
const ANTHROPIC_MODEL: &str = "claude-3-5-haiku-20241022";

pub struct Config {
    pub file: PathBuf,
    pub rules_dir: PathBuf,
    pub patterns_agent: PathBuf,
    pub max_rounds: usize,
    pub target_matches: usize,
    pub lang: String,
    pub assess: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct QualityReport {
    pub coherence: f64,
    pub completeness: f64,
    pub readability: f64,
    pub fidelity: f64,
    pub overall: f64,
    pub notes: String,
}

#[derive(Debug, Deserialize)]
struct QualityReportFields {
    coherence: f64,
    completeness: f64,
    readability: f64,
    fidelity: f64,
    #[serde(default)]
    notes: String,
}

impl<'de> Deserialize<'de> for QualityReport {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let fields = QualityReportFields::deserialize(deserializer)?;
        QualityReport::from_scores(
            fields.coherence,
            fields.completeness,
            fields.readability,
            fields.fidelity,
            fields.notes,
        )
        .map_err(serde::de::Error::custom)
    }
}

impl QualityReport {
    pub fn from_scores(
        coherence: f64,
        completeness: f64,
        readability: f64,
        fidelity: f64,
        notes: impl Into<String>,
    ) -> Result<Self, String> {
        validate_score("coherence", coherence)?;
        validate_score("completeness", completeness)?;
        validate_score("readability", readability)?;
        validate_score("fidelity", fidelity)?;

        let overall = (coherence + completeness + readability + fidelity) * 0.25;

        Ok(Self {
            coherence,
            completeness,
            readability,
            fidelity,
            overall,
            notes: notes.into(),
        })
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let ruleset = Ruleset::load_from_dir(&config.rules_dir)?;
    let scorer = Scorer::new(ruleset);
    let agent_patterns = fs::read_to_string(&config.patterns_agent)?;
    let original_text = fs::read_to_string(&config.file)?;
    let mut current_text = original_text.clone();
    let mut current_matches = scorer.analyze(&current_text, &config.lang);

    if current_matches.len() > config.target_matches {
        for round in 1..=config.max_rounds {
            let prompt = build_prompt(&current_text, &current_matches, &agent_patterns, round)?;
            let rewritten = request_rewrite(&prompt)?;

            if rewritten.trim().is_empty() {
                return Err("deslop LLM response was empty".into());
            }

            fs::write(&config.file, &rewritten)?;
            current_text = rewritten;
            current_matches = scorer.analyze(&current_text, &config.lang);

            eprintln!(
                "deslop round {round}/{}: {} structural matches remain",
                config.max_rounds,
                current_matches.len()
            );

            if current_matches.len() <= config.target_matches {
                break;
            }
        }
    }

    if config.assess {
        let quality = assess_quality(&original_text, &current_text)?;
        eprint_quality_report(&quality);
    }

    print!("{}", current_text);
    Ok(())
}

pub fn assess_quality(
    original: &str,
    rewritten: &str,
) -> Result<QualityReport, Box<dyn std::error::Error>> {
    let prompt = build_assessment_prompt(original, rewritten);
    let response = request_assessment(&prompt)?;
    parse_quality_report(&response)
}

pub fn eprint_low_quality_warning(report: &QualityReport) {
    if report.overall < 0.6 {
        eprintln!(
            "WARNING: quality assessment overall score {:.2} is below 0.60; the rewrite may have degraded quality.",
            report.overall
        );
    }
}

fn build_prompt(
    input_text: &str,
    structural_matches: &[MatchResult],
    agent_patterns: &str,
    round: usize,
) -> Result<String, Box<dyn std::error::Error>> {
    let structural_json = serde_json::to_string_pretty(structural_matches)?;

    Ok(format!(
        r#"You are a language-agnostic editor cleaning AI slop by holistic full-text inference.

Round: {round}

Critical instruction: REWRITE the entire text naturally. Do NOT regex-replace. Do NOT cut words from compound terms. Preserve meaning and grammar. Output ONLY the rewritten text.

Rules:
1. Read the full input before rewriting.
2. Use structural matches as evidence, not as replacement commands.
3. Use the universal pattern categories by intent, not as literal phrase lists.
4. Preserve facts, names, code, quotations, useful formatting, language, register, and meaning.
5. Return a complete replacement for the whole input text. Do not return a diff, patch, notes, markdown fence, or explanation.

## Structural matches from analyze() as JSON
```json
{structural_json}
```

## patterns-agent.md universal categories
```markdown
{agent_patterns}
```

## Full input text
```text
{input_text}
```

Final reminder: REWRITE the entire text naturally. Do NOT regex-replace. Do NOT cut words from compound terms. Preserve meaning and grammar. Output ONLY the rewritten text.
"#
    ))
}

fn build_assessment_prompt(original: &str, rewritten: &str) -> String {
    format!(
        r#"You are a quality evaluator for rewritten text.

This is EVALUATION ONLY, not rewriting. Do not output rewritten text.
Compare the ORIGINAL text with the REWRITTEN text and score the rewrite on four axes from 0.0 to 1.0:

1. Coherence: Logical flow between paragraphs, topic consistency, argument structure. Does the text read as a unified piece?
2. Completeness: Are all key facts/ideas from the original preserved? Nothing important was lost in rewriting?
3. Readability: Natural sentence structure, correct grammar, appropriate register. Does it sound like a human wrote it?
4. Fidelity: Is the meaning faithful to the original? No distortion, no hallucinated content, no shifted emphasis?

Return ONLY valid JSON with this exact shape:
{{
  "coherence": 0.85,
  "completeness": 0.90,
  "readability": 0.88,
  "fidelity": 0.92,
  "notes": "Briefly explain any low scores or quality concerns."
}}

Do not include markdown fences, explanations outside JSON, or rewritten text. The caller computes the overall score.

## ORIGINAL text
```text
{original}
```

## REWRITTEN text
```text
{rewritten}
```
"#
    )
}

fn request_rewrite(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    if let Ok(api_key) = env::var("OPENAI_API_KEY") {
        if !api_key.trim().is_empty() {
            return call_openai(&api_key, prompt);
        }
    }

    if let Ok(api_key) = env::var("ANTHROPIC_API_KEY") {
        if !api_key.trim().is_empty() {
            return call_anthropic(&api_key, prompt);
        }
    }

    interactive_rewrite(prompt)
}

fn request_assessment(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    if let Ok(api_key) = env::var("OPENAI_API_KEY") {
        if !api_key.trim().is_empty() {
            return call_openai(&api_key, prompt);
        }
    }

    if let Ok(api_key) = env::var("ANTHROPIC_API_KEY") {
        if !api_key.trim().is_empty() {
            return call_anthropic(&api_key, prompt);
        }
    }

    interactive_assessment(prompt)
}

fn call_openai(api_key: &str, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = http_client()?;
    let response: Value = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&json!({
            "model": OPENAI_MODEL,
            "messages": [
                {"role": "user", "content": prompt}
            ],
            "temperature": 0.3
        }))
        .send()?
        .error_for_status()?
        .json()?;

    response["choices"]
        .as_array()
        .and_then(|choices| choices.first())
        .and_then(|choice| choice["message"]["content"].as_str())
        .map(ToOwned::to_owned)
        .ok_or_else(|| "OpenAI response did not contain choices[0].message.content".into())
}

fn call_anthropic(api_key: &str, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = http_client()?;
    let response: Value = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&json!({
            "model": ANTHROPIC_MODEL,
            "max_tokens": 8192,
            "messages": [
                {"role": "user", "content": prompt}
            ]
        }))
        .send()?
        .error_for_status()?
        .json()?;

    response["content"]
        .as_array()
        .and_then(|content| {
            content
                .iter()
                .find_map(|block| block["text"].as_str())
                .map(ToOwned::to_owned)
        })
        .ok_or_else(|| "Anthropic response did not contain content text".into())
}

fn http_client() -> Result<Client, Box<dyn std::error::Error>> {
    Ok(Client::builder()
        .timeout(Duration::from_secs(120))
        .build()?)
}

fn interactive_rewrite(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    println!("{prompt}");
    eprintln!(
        "No OPENAI_API_KEY or ANTHROPIC_API_KEY set. Paste the complete rewritten text on stdin, then send EOF."
    );

    let mut rewritten = String::new();
    io::stdin().read_to_string(&mut rewritten)?;
    Ok(rewritten)
}

fn interactive_assessment(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    eprintln!("{prompt}");
    eprintln!(
        "No OPENAI_API_KEY or ANTHROPIC_API_KEY set. Paste the quality assessment JSON on stdin, then send EOF."
    );

    let mut assessment = String::new();
    io::stdin().read_to_string(&mut assessment)?;
    Ok(assessment)
}

fn parse_quality_report(response: &str) -> Result<QualityReport, Box<dyn std::error::Error>> {
    let trimmed = response.trim();
    match serde_json::from_str::<QualityReport>(trimmed) {
        Ok(report) => Ok(report),
        Err(first_error) => {
            if let Some(json_object) = extract_json_object(trimmed) {
                serde_json::from_str::<QualityReport>(json_object).map_err(|error| {
                    format!(
                        "quality assessment response was not valid JSON: {error}; initial parse error: {first_error}"
                    )
                    .into()
                })
            } else {
                Err(format!("quality assessment response was not valid JSON: {first_error}").into())
            }
        }
    }
}

fn extract_json_object(response: &str) -> Option<&str> {
    let start = response.find('{')?;
    let end = response.rfind('}')?;
    (end > start).then_some(&response[start..=end])
}

fn validate_score(name: &str, score: f64) -> Result<(), String> {
    if !score.is_finite() {
        return Err(format!("{name} score must be finite"));
    }
    if !(0.0..=1.0).contains(&score) {
        return Err(format!("{name} score must be between 0.0 and 1.0"));
    }
    Ok(())
}

fn eprint_quality_report(report: &QualityReport) {
    eprintln!("Quality assessment:");
    eprintln!("  coherence: {:.2}", report.coherence);
    eprintln!("  completeness: {:.2}", report.completeness);
    eprintln!("  readability: {:.2}", report.readability);
    eprintln!("  fidelity: {:.2}", report.fidelity);
    eprintln!("  overall: {:.2}", report.overall);
    if !report.notes.trim().is_empty() {
        eprintln!("  notes: {}", report.notes.trim());
    }
    eprint_low_quality_warning(report);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_contains_required_sections_and_instruction() {
        let matches = vec![MatchResult {
            pattern_name: "sample_pattern".to_string(),
            severity: "medium".to_string(),
            weight: 1.0,
            matched_text: "sample matched text".to_string(),
            line_number: 1,
        }];

        let prompt =
            build_prompt("Full input", &matches, "# Categories", 2).expect("prompt should build");

        assert!(prompt.contains("Full input"));
        assert!(prompt.contains("# Categories"));
        assert!(prompt.contains("\"pattern_name\": \"sample_pattern\""));
        assert!(prompt.contains("REWRITE the entire text naturally. Do NOT regex-replace. Do NOT cut words from compound terms. Preserve meaning and grammar. Output ONLY the rewritten text."));
    }

    #[test]
    fn assessment_prompt_requests_evaluation_json_without_rewriting() {
        let prompt = build_assessment_prompt("Original text", "Rewritten text");

        assert!(prompt.contains("ORIGINAL text"));
        assert!(prompt.contains("REWRITTEN text"));
        assert!(prompt.contains("This is EVALUATION ONLY, not rewriting"));
        assert!(prompt.contains("\"coherence\""));
        assert!(prompt.contains("\"completeness\""));
        assert!(prompt.contains("\"readability\""));
        assert!(prompt.contains("\"fidelity\""));
        assert!(prompt.contains("\"notes\""));
    }

    #[test]
    fn quality_report_deserializes_and_computes_overall() {
        let report: QualityReport = serde_json::from_str(
            r#"{
                "coherence": 0.8,
                "completeness": 0.6,
                "readability": 1.0,
                "fidelity": 0.9,
                "notes": "Completeness lost one minor detail."
            }"#,
        )
        .expect("quality report JSON should deserialize");

        assert_eq!(report.coherence, 0.8);
        assert_eq!(report.completeness, 0.6);
        assert_eq!(report.readability, 1.0);
        assert_eq!(report.fidelity, 0.9);
        assert!((report.overall - 0.825).abs() < f64::EPSILON);
        assert_eq!(report.notes, "Completeness lost one minor detail.");
    }

    #[test]
    fn quality_report_serializes_and_round_trips() {
        let report = QualityReport::from_scores(0.7, 0.8, 0.9, 1.0, "Looks good.")
            .expect("valid scores should build a report");

        let serialized =
            serde_json::to_string(&report).expect("quality report should serialize to JSON");
        assert!(serialized.contains("\"overall\""));

        let round_trip: QualityReport =
            serde_json::from_str(&serialized).expect("serialized report should deserialize");
        assert_eq!(round_trip, report);
    }

    #[test]
    fn quality_report_rejects_scores_outside_range() {
        let error = serde_json::from_str::<QualityReport>(
            r#"{
                "coherence": 1.2,
                "completeness": 0.6,
                "readability": 1.0,
                "fidelity": 0.9,
                "notes": "Bad score."
            }"#,
        )
        .expect_err("scores outside 0.0-1.0 should fail");

        assert!(error.to_string().contains("coherence"));
    }
}
