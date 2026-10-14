use bot_ross::canvas::Canvas;
use bot_ross::cli::Cli;
use bot_ross::evolve::progress::Progress;
use bot_ross::output::save_image;
use bot_ross::painter::{background, paint};
use bot_ross::painting::plan::Plan;
use bot_ross::reference::Reference;
use bot_ross::video::Recorder;
use bot_ross::window::Window;
use clap::Parser;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let mut plan = Plan::load(&cli.config)?;
    plan.only(&cli.layers)?;
    let reference = Reference::load(&cli.reference, cli.size)?;
    let mut canvas = Canvas::new(reference.width, reference.height, background(&plan.background, &reference)?);
    let title = cli.reference.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
    let mut window = if cli.headless { None } else { Some(Window::open(&title, reference.width, reference.height)?) };
    println!("{}x{}, {} layers", reference.width, reference.height, plan.layers.len());
    {
        let mut progress = Progress::new(window.as_mut(), &plan.output);
        if let Some(path) = &cli.video {
            progress = progress.recording(Recorder::start(path, reference.width, reference.height)?, cli.video_step);
        }
        paint(&plan, &reference, &mut canvas, &mut progress, cli.seed.unwrap_or(plan.search.seed))?;
        if let (Some(frames), Some(path)) = (progress.finish(&canvas.render())?, &cli.video) {
            println!("{frames} frames in {}", path.display());
        }
    }
    let image = canvas.render();
    if let Some(out) = &cli.out {
        save_image(&image, out)?;
        println!("saved {}", out.display());
    }
    if let Some(window) = window.as_mut() {
        window.show_until_closed(&image)?;
    }
    Ok(())
}
