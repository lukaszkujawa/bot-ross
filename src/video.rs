use std::error::Error;
use std::io::Write;
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};

pub const FPS: usize = 30;
pub const HOLD_SECONDS: usize = 2;

pub struct Recorder {
    ffmpeg: Child,
    stdin: ChildStdin,
    pub frames: usize,
}

impl Recorder {
    pub fn start(path: &Path, width: usize, height: usize) -> Result<Self, Box<dyn Error>> {
        let mut ffmpeg = Command::new("ffmpeg")
            .args(["-y", "-loglevel", "error", "-f", "rawvideo", "-pix_fmt", "rgb24", "-s", &format!("{width}x{height}")])
            .args(["-framerate", &FPS.to_string(), "-i", "-", "-vf", "pad=ceil(iw/2)*2:ceil(ih/2)*2", "-pix_fmt", "yuv420p", "-preset", "veryfast"])
            .arg(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .spawn()
            .map_err(|e| format!("cannot run ffmpeg: {e}"))?;
        let stdin = ffmpeg.stdin.take().expect("piped stdin");
        Ok(Self { ffmpeg, stdin, frames: 0 })
    }

    pub fn push(&mut self, rgb: &[u8]) -> Result<(), Box<dyn Error>> {
        self.stdin.write_all(rgb).map_err(|e| format!("ffmpeg stopped taking frames: {e}"))?;
        self.frames += 1;
        Ok(())
    }

    pub fn finish(mut self, last: &[u8]) -> Result<usize, Box<dyn Error>> {
        let painted = self.frames;
        for _ in 0..FPS * HOLD_SECONDS {
            self.push(last)?;
        }
        drop(self.stdin);
        let status = self.ffmpeg.wait()?;
        if !status.success() {
            return Err(format!("ffmpeg failed with {status}").into());
        }
        Ok(painted)
    }
}
