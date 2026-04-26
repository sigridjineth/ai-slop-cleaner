use std::path::PathBuf;

use ai_slop_cleaner_rs::{
    analyze_code_paths, analyze_text, run_mcp_stdio, run_ralph, score_from_components,
};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "ai-slop-cleaner-rs",
    version,
    about = "Single-binary AI slop scorer, code-smell detector, Ralph cleaner, and MCP server"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Print the integer AI Slop Score for a file.
    Score {
        file: PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// Print full AI-slop analysis JSON for a file.
    Analyze {
        file: PathBuf,
        #[arg(long)]
        compact: bool,
    },
    /// Iteratively clean prose until the score reaches a threshold.
    Ralph {
        file: PathBuf,
        #[arg(long, default_value_t = 25)]
        threshold: u32,
        #[arg(long, default_value_t = 10)]
        max_iterations: usize,
        #[arg(long)]
        output: Option<PathBuf>,
        #[arg(long)]
        overwrite: bool,
        #[arg(long)]
        json: bool,
    },
    /// Analyze Python/JS/Rust files for code cleanup smells.
    CodeSmells {
        paths: Vec<PathBuf>,
        #[arg(long)]
        tests: Option<PathBuf>,
        #[arg(long)]
        compact: bool,
    },
    /// MCP server commands.
    Mcp {
        #[command(subcommand)]
        command: McpCommand,
    },
}

#[derive(Subcommand)]
enum McpCommand {
    /// Serve MCP JSON-RPC over stdio.
    Serve,
}

#[tokio::main]
async fn main() -> anyhow_free::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Score { file, json } => {
            let text = std::fs::read_to_string(&file)?;
            let analysis = analyze_text(&text, &file.to_string_lossy());
            if json {
                println!("{}", serde_json::to_string_pretty(&analysis)?);
            } else {
                println!("{}", score_from_components(&analysis.components));
            }
        }
        Command::Analyze { file, compact } => {
            let text = std::fs::read_to_string(&file)?;
            let analysis = analyze_text(&text, &file.to_string_lossy());
            if compact {
                println!("{}", serde_json::to_string(&analysis)?);
            } else {
                println!("{}", serde_json::to_string_pretty(&analysis)?);
            }
        }
        Command::Ralph {
            file,
            threshold,
            max_iterations,
            output,
            overwrite,
            json,
        } => {
            let result = run_ralph(
                &file,
                threshold,
                max_iterations,
                output.as_deref(),
                overwrite || output.is_none(),
            )?;
            if json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                for iteration in &result.iterations {
                    println!(
                        "iteration {}: score={} changed={}",
                        iteration.iteration, iteration.score, iteration.changed
                    );
                    for finding in iteration.top_findings.iter().take(3) {
                        let line = finding
                            .line
                            .map(|line| format!(":{line}"))
                            .unwrap_or_default();
                        println!("  - {}{}: {}", finding.category, line, finding.text);
                    }
                }
                let output = result.output.as_deref().unwrap_or("");
                println!(
                    "{}: final_score={} threshold={} {}",
                    result.status.to_uppercase(),
                    result.final_score,
                    result.threshold,
                    output
                );
            }
            if result.status != "success" {
                std::process::exit(1);
            }
        }
        Command::CodeSmells {
            paths,
            tests,
            compact,
        } => {
            let report = analyze_code_paths(&paths, tests.as_deref())?;
            if compact {
                println!("{}", serde_json::to_string(&report)?);
            } else {
                println!("{}", serde_json::to_string_pretty(&report)?);
            }
        }
        Command::Mcp {
            command: McpCommand::Serve,
        } => {
            run_mcp_stdio().await?;
        }
    }
    Ok(())
}

mod anyhow_free {
    pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
}
