use crate::fitness::Fitness;
use crate::image::Image;
use crate::output::{rgb_bytes, save_image};
use crate::painting::config::OutputConfig;
use crate::video::Recorder;
use crate::window::Window;
use std::error::Error;
use std::path::PathBuf;
use std::time::{Duration, Instant};

const FRAME_INTERVAL: Duration = Duration::from_millis(100);

pub struct Progress<'a> {
    window: Option<&'a mut Window>,
    report: usize,
    snapshots: Option<PathBuf>,
    last_frame: Instant,
    video: Option<(Recorder, usize)>,
    painted: usize,
    pub stopped: bool,
}

impl<'a> Progress<'a> {
    pub fn new(window: Option<&'a mut Window>, output: &OutputConfig) -> Self {
        Self { window, report: output.report.max(1), snapshots: output.snapshots.clone(), last_frame: Instant::now(), video: None, painted: 0, stopped: false }
    }

    pub fn recording(self, recorder: Recorder, step: usize) -> Self {
        Self { video: Some((recorder, step.max(1))), ..self }
    }

    pub fn finish(self, last: &Image) -> Result<Option<usize>, Box<dyn Error>> {
        self.video.map(|(recorder, _)| recorder.finish(&rgb_bytes(last))).transpose()
    }

    pub fn placed(&mut self, layer: &str, placed: usize, count: usize, fitness: &Fitness, started: Instant) -> Result<(), Box<dyn Error>> {
        let elapsed = started.elapsed().as_secs_f64();
        self.painted += 1;
        if let Some((recorder, step)) = &mut self.video
            && self.painted.is_multiple_of(*step)
        {
            recorder.push(&rgb_bytes(&fitness.image()))?;
        }
        if placed.is_multiple_of(self.report) || placed == count {
            println!("{layer:<16} {placed:>6}/{count}  error {:.3e}  {elapsed:>7.1}s  {:.1} strokes/s", fitness.total() as f64, placed as f64 / elapsed.max(1e-9));
            if let Some(directory) = &self.snapshots {
                std::fs::create_dir_all(directory)?;
                save_image(&fitness.image(), directory.join(format!("{layer}-{placed:06}.png")))?;
            }
        }
        if self.last_frame.elapsed() >= FRAME_INTERVAL {
            self.show(fitness)?;
        }
        Ok(())
    }

    pub fn show(&mut self, fitness: &Fitness) -> Result<(), Box<dyn Error>> {
        self.last_frame = Instant::now();
        if let Some(window) = self.window.as_deref_mut() {
            self.stopped = !window.present(&fitness.image())?;
        }
        Ok(())
    }
}
