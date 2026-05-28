#![forbid(unsafe_code)]

mod args;
mod pager;

use crate::args::Args;
use anyhow::{Context, Result, ensure};
use clap::{CommandFactory, Parser};
use clap_complete::generate;
use directories_next::ProjectDirs;
use indicatif::{ProgressBar, ProgressStyle};
use log::info;
use owo_colors::OwoColorize;
use rdict_core::rdict::Rdict;
use rustyline::DefaultEditor;
use std::env;
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

#[derive(Debug)]
enum Format {
    /// Markdown with ANSI color escape sequences
    MarkdownColored,
    /// Plain Markdown
    Markdown,
    /// Formatted JSON
    Json,
}

impl App {
    /// Initializes the `rdict_cli` application
    async fn new(cli: Args) -> Result<Self> {
        let format = if cli.json {
            Format::Json
        } else if console::colors_enabled() {
            Format::MarkdownColored
        } else {
            Format::Markdown
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
        let query = &self.cli.input_text.join(" ");

        match &self.cli.input_text.len() {
            1.. => {
                info!("`input_text` provided through argument.");
                self.output_results(query).await?;
            }
            0 if stdin_is_piped => {
                info!("`input_text` provided through pipe.");
                let mut buffer = String::new();
                io::stdin().read_to_string(&mut buffer)?;
                let input_text = buffer.trim();
                ensure!(!input_text.is_empty(), "No input_text specified");
                self.output_results(input_text).await?;
            }
            0 => {
                info!("`input_text` not provided, entering interactive mode.");
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
            let readline = if cfg!(target_family = "windows") || !console::colors_enabled() {
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
        let spinner = supports_ansi().then(|| {
            let spinner = ProgressBar::new_spinner();
            spinner.set_message("Fetching data...");
            spinner.enable_steady_tick(Duration::from_millis(100));
            #[expect(clippy::literal_string_with_formatting_args)]
            spinner.set_style(
                ProgressStyle::default_spinner()
                    .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
                    .template("{spinner} {msg}")
                    .unwrap(),
            );

            spinner
        });

        let result = self
            .client
            .get_results(
                input_text,
                self.cli
                    .language
                    .unwrap_or(args::CliLanguage::English)
                    .into(),
            )
            .await?;

        if let Some(spinner) = spinner {
            spinner.finish_and_clear();
        }

        match self.format {
            Format::MarkdownColored | Format::Markdown => {
                let output = match self.format {
                    Format::MarkdownColored => result.render_colored(),
                    Format::Markdown => result.render_plain(),
                    _ => unreachable!(),
                };

                let mut indented_output = output
                    .lines()
                    .map(|line| format!("  {line}"))
                    .collect::<Vec<_>>()
                    .join("\n");

                if result.is_cached {
                    indented_output.push_str(&format!(
                        "\n\n  {}",
                        format!("[ {input_text} ] From cache").bright_black()
                    ));
                }

                let indented_output = format!("\n{indented_output}\n");

                // If window is too small, output the result in a pager
                let (_, height) =
                    crossterm::terminal::size().context("Failed to get terminal size")?;
                // NOTE: Removed 4 lines for shell prompt.
                if height - 4 < indented_output.lines().count() as u16 {
                    let mut terminal = ratatui::init();
                    (pager::Pager {
                        text: indented_output,
                        ..Default::default()
                    })
                    .run(&mut terminal)?;
                    ratatui::restore();
                } else {
                    println!("{indented_output}");
                }
            }
            Format::Json => println!("{}", serde_json::to_string_pretty(&result)?),
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let cli = Args::parse();

    // Generate shell completions
    if let Some(generator) = cli.completion {
        let mut cmd = Args::command();
        let name = cmd.get_name().to_string();
        generate(generator, &mut cmd, &name, &mut io::stdout());
        return Ok(());
    }

    let app = App::new(cli).await?;
    app.run().await
}

#[must_use]
pub fn supports_ansi() -> bool {
    if env::var("NO_COLOR").is_ok() {
        return false;
    }

    matches!(env::var("TERM"), Ok(term) if term != "dumb")
}
