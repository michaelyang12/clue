use clap::Parser;

/// Natural language to shell command translator
#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// What you want to do (or search term with --history)
    #[arg(default_value = "")]
    pub(crate) input: String,

    /// Show alternative commands and explanations
    #[arg(short, long)]
    pub(crate) verbose: bool,

    /// Execute the command after confirmation
    #[arg(short = 'x', long)]
    pub(crate) execute: bool,

    /// Show command history (optionally filter by search term)
    #[arg(long)]
    pub(crate) history: bool,
}
