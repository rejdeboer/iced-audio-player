use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug, Clone)]
pub enum Message {
    Tick(Instant),
    Play,
    Pause,
    LoadFile(PathBuf),
    SetPositionPreview(f32),
    SetPosition,
}
