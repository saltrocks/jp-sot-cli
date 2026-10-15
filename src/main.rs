use std::process::ExitCode;

use clap::Parser;
use jp_sot::{Dictionary, HttpFetcher};

/// Look up a Japanese word or kanji and print its breakdown as JSON.
#[derive(Parser)]
#[command(
    version,
    after_help = "Examples:\n  jp-sot 宣言    # look up a word\n  jp-sot 宣      # look up a single kanji"
)]
struct Cli {
    /// Word or single kanji to look up
    query: String,

    /// Print compact single-line JSON
    #[arg(long)]
    compact: bool,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(&cli).await {
        Ok(json) => {
            println!("{json}");
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
    let json = if cli.compact {
        serde_json::to_string(&entry)?
    } else {
        serde_json::to_string_pretty(&entry)?
    };
    Ok(json)
}
