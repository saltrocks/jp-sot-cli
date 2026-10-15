use std::process::ExitCode;

use clap::{Parser, ValueEnum};
use jp_sot::{Dictionary, HttpFetcher};

/// Look up a Japanese word or kanji and print its breakdown.
#[derive(Parser)]
#[command(
    version,
    after_help = "Examples:\n  jp-sot 宣言    # look up a word\n  jp-sot 宣      # look up a single kanji"
)]
struct Cli {
    /// Word or single kanji to look up
    query: String,

    /// Output format
    #[arg(long, value_enum, default_value_t)]
    format: Format,

    /// Print JSON on a single line (with --format json)
    #[arg(long)]
    compact: bool,
}

#[derive(Clone, Copy, Default, ValueEnum)]
enum Format {
    /// Indented outline with the kanji decomposition drawn as a tree
    #[default]
    Text,
    /// JSON, for scripts and other programs
    Json,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(&cli).await {
        Ok(output) => {
            print!("{output}");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("Error: {err:#}");
            ExitCode::FAILURE
        }
    }
}

async fn run(cli: &Cli) -> anyhow::Result<String> {
    let dictionary = Dictionary::new(HttpFetcher::new(HttpFetcher::DEFAULT_DELAY)?);
    let entry = dictionary.lookup(cli.query.trim()).await?;
    let output = match cli.format {
        Format::Text => jp_sot::render::text(&entry),
        Format::Json if cli.compact => serde_json::to_string(&entry)? + "\n",
        Format::Json => serde_json::to_string_pretty(&entry)? + "\n",
    };
    Ok(output)
}
