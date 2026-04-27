use std::io::Read;
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

mod pattern_loader;
mod scorer;

use pattern_loader::Ruleset;
use scorer::Scorer;

#[derive(Parser, Debug)]
#[command(name = "ai-slop-cleaner")]
#[command(about = "Detect and score AI slop in text documents")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Path to the rules directory (contains banned-patterns.md and banned-words.md)
    #[arg(short, long, default_value = "rules")]
    rules_dir: PathBuf,

    /// Language filter: en, ko, auto, or all (default: auto).
    /// 'auto' detects based on text content. 'en' skips ko_* patterns.
    /// 'ko' skips English-only patterns. 'all' applies everything.
    #[arg(short, long, default_value = "auto")]
    lang: String,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Score a single file for AI slop
    Score {
        /// Path to the text file to analyze
        file: PathBuf,

        /// Output format: text, json, or markdown
        #[arg(short, long, default_value = "text")]
        format: String,
    },
    /// Score text from stdin
    Stdin {
        /// Output format: text, json, or markdown
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
        Commands::Score { file, format } => {
            let content = fs::read_to_string(&file)?;
            let result = scorer.score(&content, &cli.lang);
            print_result(&result, &format)?;
        }
        Commands::Stdin { format } => {
            let mut content = String::new();
            std::io::stdin().read_to_string(&mut content)?;
            let result = scorer.score(&content, &cli.lang);
            print_result(&result, &format)?;
        }
        Commands::Rules => {
            println!("Loaded {} banned patterns", scorer.ruleset().patterns.len());
            println!("Loaded {} banned words", scorer.ruleset().words.len());
            println!();
            println!("Banned Patterns:");
            for p in &scorer.ruleset().patterns {
                println!("  - {} (severity: {}, weight: {})", p.name, p.severity, p.weight);
            }
            println!();
            println!("Banned Words:");
            for w in &scorer.ruleset().words {
                println!(
                    "  - {} (replacement: {:?}, weight: {})",
                    w.word, w.replacement, w.weight
                );
            }
        }
    }

    Ok(())
}

fn print_result(
    result: &scorer::ScoreResult,
    format: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    match format {
        "json" => {
            let json = serde_json::to_string_pretty(result)?;
            println!("{}", json);
        }
        "markdown" => {
            println!("# AI Slop Score Report\n");
            println!("**Overall Score:** {:.2}/100\n", result.overall_score);
            println!("## Matches\n");
            for m in &result.matches {
                println!(
                    "- **{}** (severity: {}, weight: {:.1})\n  - {}\n  - Line: {}\n",
                    m.pattern_name, m.severity, m.weight, m.description, m.line_number
                );
            }
        }
        _ => {
            println!("AI Slop Score: {:.2}/100", result.overall_score);
            println!("Matches found: {} (patterns: {}, words: {})", 
                result.matches.len() + result.word_matches.len(),
                result.matches.len(), result.word_matches.len());
            for m in &result.matches {
                println!(
                    "  [{}] {} (weight: {:.1}) - Line {}: {}",
                    m.severity, m.pattern_name, m.weight, m.line_number, m.matched_text
                );
            }
            for w in &result.word_matches {
                let repl = match &w.replacement {
                    Some(r) => format!(" -> {}", r),
                    None => String::new(),
                };
                println!(
                    "  [word] \"{}\"{}  (weight: {:.1}) - Line {}: {}",
                    w.word, repl, w.weight, w.line_number, w.matched_text
                );
            }
        }
    }
    Ok(())
}
