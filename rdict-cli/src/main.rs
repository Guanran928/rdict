mod args;

use crate::args::Args;
use anyhow::{Context, Result, anyhow, ensure};
use clap::Parser;
use directories_next::ProjectDirs;
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use rdict_core::parse::TranslationData;
use rdict_core::rdict::{Format, Rdict, output_chinese, output_english};
use rustyline::DefaultEditor;
use std::io::{self, IsTerminal, Read};
use std::path::PathBuf;
use std::time::Duration;

struct App {
    format: Format,
    client: Rdict,
    cli: Args,
}

impl App {
    async fn new(cli: Args) -> Result<Self> {
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

        let client = Rdict::new("https://m.youdao.com", db_path).await?;

        Ok(Self {
            format,
            client,
            cli,
        })
    }

    async fn run(&self) -> Result<()> {
        let stdin_is_piped = !io::stdin().is_terminal();

        match &self.cli.word {
            Some(word) => self.output_results(word).await?,
            None if stdin_is_piped => {
                let mut buffer = String::new();
                io::stdin().read_to_string(&mut buffer)?;
                let word = buffer.trim();
                ensure!(!word.is_empty(), "No word specified");
                self.output_results(word).await?;
            }
            _ => {
                self.interactive_mode()
                    .await
                    .context("Interactive mode failed")?;
            }
        }

        Ok(())
    }

    async fn interactive_mode(&self) -> rustyline::Result<()> {
        let mut rl = DefaultEditor::new()?;
        loop {
            let readline = if cfg!(target_family = "windows") {
                rl.readline("[rdict]# ")
            } else {
                rl.readline(format!("{}# ", "[rdict]".green()).as_str())
            };
            match readline {
                Ok(line) => {
                    if !line.is_empty() {
                        rl.add_history_entry(line.as_str())?;
                        let word = line.trim();
                        if let Err(err) = self.output_results(word).await {
                            println!("Error: {err:?}");
                        }
                    }
                }
                Err(err) => {
                    println!("Error: {err:?}");
                    break;
                }
            }
        }
        Ok(())
    }

    async fn output_results(&self, word: &str) -> Result<()> {
        let spinner = ProgressBar::new_spinner();
        spinner.set_message("Fetching data...");
        spinner.enable_steady_tick(Duration::from_millis(100));
        spinner.set_style(
            ProgressStyle::default_spinner()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
                .template("{spinner} {msg}")
                .unwrap(),
        );

        let result = self.client.get_results(word).await?;
        spinner.finish_and_clear();

        match self.format {
            Format::MarkdownColored => {
                let output = match &result.data {
                    TranslationData::ToChinese(tc) => output_chinese(tc)?,
                    TranslationData::ToEnglish(te) => output_english(te)?,
                };
                let indented: String = output
                    .lines()
                    .map(|line| format!("  {line}"))
                    .collect::<Vec<_>>()
                    .join("\n");

                println!("\n{indented}\n");

                if result.is_cached {
                    println!("  {}\n", format!("[ {word} ] From cache").bright_black());
                }
            }
            Format::Json => println!("{}", serde_json::to_string_pretty(&result.data)?),
            _ => todo!(),
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Args::parse();
    let app = App::new(cli).await?;
    app.run().await
}
