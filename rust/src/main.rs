use clap::{Parser, Subcommand};
use std::fs;
use std::io::Read;
use std::path::PathBuf;

mod pattern_loader;
mod scorer;

use pattern_loader::Ruleset;
use scorer::{MatchResult, Scorer};

#[derive(Parser, Debug)]
#[command(name = "ai-slop-cleaner")]
#[command(about = "Detect structural AI slop patterns in text documents")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Path to the rules directory (contains banned-patterns.md and banned-words.md)
    #[arg(short, long, default_value = "rules")]
    rules_dir: PathBuf,

    /// Language hint kept for CLI compatibility. The Rust matcher only applies
    /// universal structural patterns; language-specific judgment belongs to agents.
    #[arg(short, long, default_value = "auto")]
    lang: String,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Analyze a single file and output raw structural matches as JSON
    Analyze {
        /// Path to the text file to analyze
        file: PathBuf,
    },
    /// Deprecated: use `analyze`; no score is computed
    Score {
        /// Path to the text file to analyze
        file: PathBuf,

        /// Output format: text, json, or markdown. JSON is a raw match array.
        #[arg(short, long, default_value = "text")]
        format: String,
    },
    /// Analyze text from stdin
    Stdin {
        /// Output format: text, json, or markdown. JSON is a raw match array.
        #[arg(short, long, default_value = "text")]
        format: String,
    },
    /// List all loaded rules
    Rules,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let ruleset = Ruleset::load_from_dir(&cli.rules_dir)?;
    let scorer = Scorer::new(ruleset);

    match cli.command {
        Commands::Analyze { file } => {
            let content = fs::read_to_string(&file)?;
            let matches = scorer.analyze(&content, &cli.lang);
            print_matches_json(&matches)?;
        }
        Commands::Score { file, format } => {
            eprintln!(
                "Warning: 'score' is deprecated and no longer computes an overall score. Use 'analyze' for raw structural matches."
            );
            let content = fs::read_to_string(&file)?;
            let matches = scorer.analyze(&content, &cli.lang);
            print_matches(&matches, &format)?;
        }
        Commands::Stdin { format } => {
            let mut content = String::new();
            std::io::stdin().read_to_string(&mut content)?;
            let matches = scorer.analyze(&content, &cli.lang);
            print_matches(&matches, &format)?;
        }
        Commands::Rules => {
            println!("Loaded {} banned patterns", scorer.ruleset().patterns.len());
            println!("Loaded {} banned words", scorer.ruleset().words.len());
            let agent_patterns_path = cli.rules_dir.join("patterns-agent.md");
            let agent_patterns =
                Ruleset::load_agent_patterns_from_file(&agent_patterns_path).unwrap_or_default();
            println!("Loaded {} agent patterns", agent_patterns.len());
            println!();
            println!("Banned Patterns:");
            for p in &scorer.ruleset().patterns {
                println!(
                    "  - {} (scope: {}, severity: {}, weight: {}, description: {})",
                    p.name, p.lang_scope, p.severity, p.weight, p.description
                );
            }
            println!();
            println!("Banned Words:");
            for w in &scorer.ruleset().words {
                println!(
                    "  - {} (replacement: {:?}, weight: {})",
                    w.word, w.replacement, w.weight
                );
            }
            println!();
            println!("Agent Patterns:");
            for p in &agent_patterns {
                println!(
                    "  - {} (severity: {}, weight: {}, examples: {}, note: {}, description: {})",
                    p.name,
                    p.severity,
                    p.weight,
                    p.examples.len(),
                    p.multilingual_note.as_deref().unwrap_or("none"),
                    p.description
                );
            }
        }
    }

    Ok(())
}

fn print_matches_json(matches: &[MatchResult]) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(matches)?;
    println!("{}", json);
    Ok(())
}

fn print_matches(matches: &[MatchResult], format: &str) -> Result<(), Box<dyn std::error::Error>> {
    match format {
        "json" => {
            print_matches_json(matches)?;
        }
        "markdown" => {
            println!("# AI Slop Structural Match Report\n");
            println!("Matches found: {}\n", matches.len());
            for m in matches {
                println!(
                    "- **{}** (severity: {}, weight: {:.1})\n  - Line {}: {}\n",
                    m.pattern_name, m.severity, m.weight, m.line_number, m.matched_text
                );
            }
        }
        _ => {
            println!("Structural matches found: {}", matches.len());
            for m in matches {
                println!(
                    "  [{}] {} (weight: {:.1}) - Line {}: {}",
                    m.severity, m.pattern_name, m.weight, m.line_number, m.matched_text
                );
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn parses_analyze_subcommand() {
        let cli = Cli::try_parse_from(["ai-slop-cleaner", "analyze", "document.md"])
            .expect("analyze subcommand should parse");

        match cli.command {
            Commands::Analyze { file } => assert_eq!(file, PathBuf::from("document.md")),
            other => panic!("expected analyze command, got {other:?}"),
        }
    }
}
