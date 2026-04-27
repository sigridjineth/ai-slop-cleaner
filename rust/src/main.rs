use clap::{Parser, Subcommand};
use std::fs;
use std::io::Read;
use std::path::PathBuf;

mod deslop;
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
    /// Iteratively rewrite a file through a full-text LLM deslop loop
    Deslop {
        /// Path to the text file to rewrite
        file: PathBuf,

        /// Maximum rewrite/analyze rounds to run
        #[arg(long, default_value_t = 3)]
        max_rounds: usize,

        /// Stop once structural matches are at or below this count
        #[arg(long, default_value_t = 0)]
        target_matches: usize,

        /// Override the root rules directory for this deslop run
        #[arg(long)]
        rules_dir: Option<PathBuf>,

        /// Path to the agent-readable semantic pattern categories
        #[arg(long)]
        patterns_agent: Option<PathBuf>,

        /// Run at least one LLM rewrite even when analyze finds no structural matches
        #[arg(long)]
        force_rewrite: bool,

        /// Run a final LLM quality assessment after rewriting
        #[arg(long, default_value_t = true)]
        assess: bool,

        /// Skip the final LLM quality assessment
        #[arg(long = "no-assess", action = clap::ArgAction::SetTrue, conflicts_with = "assess")]
        no_assess: bool,
    },
    /// Compare original and rewritten files with an LLM quality assessment
    Assess {
        /// Path to the original text file
        original: PathBuf,

        /// Path to the rewritten text file
        rewritten: PathBuf,
    },
    /// List all loaded rules
    Rules,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Analyze { file } => {
            let scorer = load_scorer(&cli.rules_dir)?;
            let content = fs::read_to_string(&file)?;
            let matches = scorer.analyze(&content, &cli.lang);
            print_matches_json(&matches)?;
        }
        Commands::Score { file, format } => {
            eprintln!(
                "Warning: 'score' is deprecated and no longer computes an overall score. Use 'analyze' for raw structural matches."
            );
            let scorer = load_scorer(&cli.rules_dir)?;
            let content = fs::read_to_string(&file)?;
            let matches = scorer.analyze(&content, &cli.lang);
            print_matches(&matches, &format)?;
        }
        Commands::Stdin { format } => {
            let scorer = load_scorer(&cli.rules_dir)?;
            let mut content = String::new();
            std::io::stdin().read_to_string(&mut content)?;
            let matches = scorer.analyze(&content, &cli.lang);
            print_matches(&matches, &format)?;
        }
        Commands::Deslop {
            file,
            max_rounds,
            target_matches,
            rules_dir,
            patterns_agent,
            force_rewrite,
            assess,
            no_assess,
        } => {
            let rules_dir = rules_dir.unwrap_or(cli.rules_dir);
            let patterns_agent =
                patterns_agent.unwrap_or_else(|| rules_dir.join("patterns-agent.md"));
            deslop::run(deslop::Config {
                file,
                rules_dir,
                patterns_agent,
                max_rounds,
                target_matches,
                lang: cli.lang,
                assess: assess && !no_assess,
                force_rewrite,
            })?;
        }
        Commands::Assess {
            original,
            rewritten,
        } => {
            let original_text = fs::read_to_string(&original)?;
            let rewritten_text = fs::read_to_string(&rewritten)?;
            let report = deslop::assess_quality(&original_text, &rewritten_text)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            deslop::eprint_low_quality_warning(&report);
        }
        Commands::Rules => {
            let scorer = load_scorer(&cli.rules_dir)?;
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

fn load_scorer(rules_dir: &PathBuf) -> Result<Scorer, Box<dyn std::error::Error>> {
    let ruleset = Ruleset::load_from_dir(rules_dir)?;
    Ok(Scorer::new(ruleset))
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

    #[test]
    fn parses_deslop_subcommand() {
        let cli = Cli::try_parse_from([
            "ai-slop-cleaner",
            "deslop",
            "document.md",
            "--max-rounds",
            "5",
            "--target-matches",
            "1",
            "--rules-dir",
            "custom-rules",
            "--patterns-agent",
            "custom-patterns.md",
            "--force-rewrite",
        ])
        .expect("deslop subcommand should parse");

        match cli.command {
            Commands::Deslop {
                file,
                max_rounds,
                target_matches,
                rules_dir,
                patterns_agent,
                force_rewrite,
                assess,
                no_assess,
            } => {
                assert_eq!(file, PathBuf::from("document.md"));
                assert_eq!(max_rounds, 5);
                assert_eq!(target_matches, 1);
                assert_eq!(rules_dir, Some(PathBuf::from("custom-rules")));
                assert_eq!(patterns_agent, Some(PathBuf::from("custom-patterns.md")));
                assert!(force_rewrite);
                assert!(assess);
                assert!(!no_assess);
            }
            other => panic!("expected deslop command, got {other:?}"),
        }
    }

    #[test]
    fn parses_deslop_no_assess_flag() {
        let cli = Cli::try_parse_from(["ai-slop-cleaner", "deslop", "document.md", "--no-assess"])
            .expect("deslop --no-assess should parse");

        match cli.command {
            Commands::Deslop {
                assess, no_assess, ..
            } => {
                assert!(assess);
                assert!(no_assess);
            }
            other => panic!("expected deslop command, got {other:?}"),
        }
    }

    #[test]
    fn parses_assess_subcommand() {
        let cli = Cli::try_parse_from(["ai-slop-cleaner", "assess", "original.md", "rewritten.md"])
            .expect("assess subcommand should parse");

        match cli.command {
            Commands::Assess {
                original,
                rewritten,
            } => {
                assert_eq!(original, PathBuf::from("original.md"));
                assert_eq!(rewritten, PathBuf::from("rewritten.md"));
            }
            other => panic!("expected assess command, got {other:?}"),
        }
    }
}
