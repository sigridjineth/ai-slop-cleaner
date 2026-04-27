use crate::pattern_loader::Ruleset;
use crate::scorer::{MatchResult, Scorer};
use reqwest::blocking::Client;
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
}

pub fn run(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let ruleset = Ruleset::load_from_dir(&config.rules_dir)?;
    let scorer = Scorer::new(ruleset);
    let agent_patterns = fs::read_to_string(&config.patterns_agent)?;
    let mut current_text = fs::read_to_string(&config.file)?;
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

    print!("{}", current_text);
    Ok(())
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
}
