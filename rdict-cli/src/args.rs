use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    pub input_text: Option<String>,

    /// Cache translations
    #[arg(long)]
    pub no_cache: bool,

    /// Output using JSON
    #[arg(long)]
    pub json: bool,
}
