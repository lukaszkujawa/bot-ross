mod base64;
mod rpc;
mod studio;

use bot_ross::window::Window;
use clap::Parser;
use std::error::Error;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::sync::mpsc::{self, RecvTimeoutError};
use std::thread;
use std::time::Duration;
use studio::Studio;

const FRAME: Duration = Duration::from_millis(30);

#[derive(Parser, Debug)]
#[command(name = "bot-ross-mcp", about = "An MCP server that lets an agent draw on a canvas with any brush preset")]
struct Cli {
    #[arg(long, default_value = "brushes")]
    brushes: PathBuf,
    #[arg(long, default_value_t = 1024)]
    size: usize,
    #[arg(long)]
    headless: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let mut studio = Studio::new(cli.brushes, cli.size, cli.size);
    let (sender, requests) = mpsc::channel();
    thread::spawn(move || {
        for line in io::stdin().lock().lines().map_while(Result::ok) {
            if !line.trim().is_empty() && sender.send(line).is_err() {
                break;
            }
        }
    });
    let mut window = if cli.headless { None } else { Some(Window::open("Bot Ross canvas", cli.size, cli.size)?) };
    let mut image = studio.image();
    let mut stdout = io::stdout().lock();
    loop {
        let message = match &mut window {
            Some(window) => {
                if !window.present(&image)? {
                    break;
                }
                match requests.recv_timeout(FRAME) {
                    Ok(message) => message,
                    Err(RecvTimeoutError::Timeout) => continue,
                    Err(RecvTimeoutError::Disconnected) => break,
                }
            }
            None => match requests.recv() {
                Ok(message) => message,
                Err(_) => break,
            },
        };
        if let Some(response) = rpc::handle(&message, &mut studio) {
            writeln!(stdout, "{response}")?;
            stdout.flush()?;
        }
        image = studio.image();
        if let Some(open) = &window {
            if (image.width, image.height) != open.size() {
                window = Some(Window::open("Bot Ross canvas", image.width, image.height)?);
            }
        }
    }
    Ok(())
}
