use bot_ross::video::{FPS, HOLD_SECONDS, Recorder};
use std::path::Path;
use std::process::Command;

fn ffmpeg_missing() -> bool {
    Command::new("ffmpeg").arg("-version").output().is_err()
}

#[test]
fn encodes_pushed_frames_and_holds_the_last() {
    if ffmpeg_missing() {
        return;
    }
    let path = std::env::temp_dir().join(format!("bot-ross-video-{}.mp4", std::process::id()));
    let mut recorder = Recorder::start(&path, 5, 3).unwrap();
    for shade in [0u8, 128] {
        recorder.push(&[shade; 45]).unwrap();
    }
    assert_eq!(recorder.finish(&[255; 45]).unwrap(), 2);
    let decoded = Command::new("ffmpeg").arg("-i").arg(&path).args(["-map", "0:v:0", "-f", "null", "-"]).output().unwrap();
    let log = String::from_utf8_lossy(&decoded.stderr);
    let frames = log.rsplit("frame=").next().and_then(|rest| rest.split_whitespace().next()).unwrap_or_default();
    assert!(log.contains(", 6x4") && frames == (2 + FPS * HOLD_SECONDS).to_string(), "{log}");
    std::fs::remove_file(path).unwrap();
}

#[test]
fn unwritable_output_is_an_error() {
    if ffmpeg_missing() {
        return;
    }
    let mut recorder = Recorder::start(Path::new("/nonexistent/dir/out.mp4"), 2, 2).unwrap();
    let _ = recorder.push(&[0; 12]);
    assert!(recorder.finish(&[0; 12]).is_err());
}
