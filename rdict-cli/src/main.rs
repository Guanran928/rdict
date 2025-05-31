mod args;

use crate::args::Args;
use anyhow::{Context, anyhow, ensure};
use clap::Parser;
use directories_next::ProjectDirs;
use rdict_core::rdict::{Format, Rdict};
use std::io::{self, IsTerminal, Read};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Args::parse();
    let stdin_is_piped = !io::stdin().is_terminal();

    let format = if cli.json {
        Format::Json
    } else {
        Format::MarkdownColored
    };

    let db_path: Option<PathBuf> = if cli.no_cache {
        None
    } else {
        let proj_dirs = ProjectDirs::from("dev", "ny4", "rdict")
            .ok_or_else(|| anyhow!("Could not determine project directory"))?;
        Some(proj_dirs.cache_dir().join("cache.db"))
    };

    let rdict = Rdict::new("https://m.youdao.com", format, db_path).await?;

    match cli.word {
        Some(word) => rdict.output_results(&word).await?,
        None if stdin_is_piped => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer).unwrap();
            let word = buffer.trim();
            ensure!(!word.is_empty(), "No word specified");
            rdict.output_results(word).await?;
        }
        _ => {
            rdict
                .interactive_mode()
                .await
                .context("Interactive mode failed")?;
        }
    }

    Ok(())
}
