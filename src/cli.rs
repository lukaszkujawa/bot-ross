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
    pub out: Option<PathBuf>,
    #[arg(long)]
    pub seed: Option<u64>,
    #[arg(long, value_delimiter = ',')]
    pub layers: Vec<String>,
    #[arg(long)]
    pub headless: bool,
    #[arg(long)]
    pub video: Option<PathBuf>,
    #[arg(long, default_value_t = 1)]
    pub video_step: usize,
}
