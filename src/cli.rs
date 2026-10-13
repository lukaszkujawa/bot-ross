use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "bot-ross", about = "Paints a reference image with evolved brush strokes")]
pub struct Cli {
    pub reference: PathBuf,
    #[arg(long)]
    pub config: PathBuf,
    #[arg(long)]
    pub size: u32,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(long)]
    pub seed: Option<u64>,
    #[arg(long)]
    pub headless: bool,
}
