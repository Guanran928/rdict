mod args;

use crate::args::Args;
use anyhow::{Context, Result, ensure};
use clap::Parser;
use directories_next::ProjectDirs;
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use rdict_core::parse::TranslationData;
use rdict_core::rdict::{Format, Rdict, render_chinese_colored, render_english_colored};
use rustyline::DefaultEditor;
use std::io::{self, IsTerminal, Read};
use std::path::PathBuf;
use std::time::Duration;

struct App {
    /// Format used to output translation data
    format: Format,
    /// Rdict client
    client: Rdict,
    /// Command-line arguments handled by `clap`
    cli: Args,
}

impl App {
    /// Initializes the `rdict_cli` application
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
                .context("Could not determine project directory")?;
            Some(proj_dirs.cache_dir().join("cache.db"))
        };

        let client = Rdict::new("https://m.youdao.com", db_path).await?;

        Ok(Self {
            format,
            client,
            cli,
        })
    }

    /// Runs `rdict_cli`
    ///
    /// Enters interactive mode if `input_text` is not provided by command-line argument or piping.
    async fn run(&self) -> Result<()> {
        let stdin_is_piped = !io::stdin().is_terminal();

        match &self.cli.input_text {
            Some(input_text) => self.output_results(input_text).await?,
            None if stdin_is_piped => {
                let mut buffer = String::new();
                io::stdin().read_to_string(&mut buffer)?;
                let input_text = buffer.trim();
                ensure!(!input_text.is_empty(), "No input_text specified");
                self.output_results(input_text).await?;
            }
            _ => {
                self.interactive_mode()
                    .await
                    .context("Interactive mode failed")?;
            }
        }

        Ok(())
    }

    /// Runs `rdict_cli` in interactive mode
    async fn interactive_mode(&self) -> rustyline::Result<()> {
        let mut rl = DefaultEditor::new()?;
        loop {
            // HACK:
            // I don't have a Windows machine to fix https://github.com/kkawakam/rustyline/issues/562
            let readline = if cfg!(target_family = "windows") {
                rl.readline("[rdict]# ")
            } else {
                rl.readline(format!("{}# ", "[rdict]".green()).as_str())
            };
            match readline {
                Ok(line) => {
                    if !line.is_empty() {
                        rl.add_history_entry(line.as_str())?;
                        let input_text = line.trim();
                        if let Err(err) = self.output_results(input_text).await {
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

    /// Formats and outputs `input_text` in different format provided when initialized
    async fn output_results(&self, input_text: &str) -> Result<()> {
        let spinner = ProgressBar::new_spinner();
        spinner.set_message("Fetching data...");
        spinner.enable_steady_tick(Duration::from_millis(100));
        spinner.set_style(
            ProgressStyle::default_spinner()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
                .template("{spinner} {msg}")
                .unwrap(),
        );

        let result = self.client.get_results(input_text).await?;
        spinner.finish_and_clear();

        match self.format {
            Format::MarkdownColored => {
                let output = match &result.data {
                    TranslationData::ToChinese(tc) => render_chinese_colored(tc)?,
                    TranslationData::ToEnglish(te) => render_english_colored(te)?,
                };
                let indented: String = output
                    .lines()
                    .map(|line| format!("  {line}"))
                    .collect::<Vec<_>>()
                    .join("\n");

                println!("\n{indented}\n");

                if result.is_cached {
                    println!(
                        "  {}\n",
                        format!("[ {input_text} ] From cache").bright_black()
                    );
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
