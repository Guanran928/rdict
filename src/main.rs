mod args;
mod parse;
mod rdict;

use crate::args::Args;
use crate::rdict::{Format, Rdict};
use anyhow::{Context, anyhow, ensure};
use clap::Parser;
use directories_next::ProjectDirs;
use rusqlite::Connection;
use std::fs;
use std::io::{self, IsTerminal, Read};

fn main() -> anyhow::Result<()> {
    let cli = Args::parse();
    let stdin_is_piped = !io::stdin().is_terminal();

    let conn: Option<Connection> = if cli.no_cache {
        None
    } else {
        let proj_dirs = ProjectDirs::from("dev", "ny4", "rdict")
            .ok_or_else(|| anyhow!("Could not determine project directory"))?;
        let db_path = proj_dirs.cache_dir().join("cache.db");
        let should_init_db = !db_path.exists();

        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create cache directory at {:?}", parent.display())
            })?;
        }

        let conn = Connection::open(&db_path)
            .with_context(|| format!("Failed to open database at {}", db_path.display()))?;

        if should_init_db {
            let init_statements = [
                "CREATE TABLE to_english_results (
                    word TEXT PRIMARY KEY,
                    data TEXT NOT NULL
                );",
                "CREATE TABLE to_chinese_results (
                    word TEXT PRIMARY KEY,
                    data TEXT NOT NULL
                );",
            ];

            for statement in init_statements {
                conn.execute(statement, ()).with_context(|| {
                    format!(
                        "Failed to initialize SQLite cache database: {}",
                        db_path.display()
                    )
                })?;
            }
        }

        Some(conn)
    };

    let format = if cli.json {
        Format::Json
    } else {
        Format::Pretty
    };

    let rdict = Rdict::new("https://m.youdao.com", conn, format);

    match cli.word {
        Some(word) => rdict.output_results(&word)?,
        None if stdin_is_piped => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer).unwrap();
            let word = buffer.trim();
            ensure!(!word.is_empty(), "No word specified");
            rdict.output_results(word)?;
        }
        _ => {
            rdict
                .interactive_mode()
                .context("Interactive mode failed")?;
        }
    }

    Ok(())
}
