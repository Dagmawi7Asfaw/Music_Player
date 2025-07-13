use std::path::Path;

pub fn get_file_name_from_path(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string()
}

pub fn is_audio_file(path: &str) -> bool {
    if let Some(extension) = Path::new(path).extension() {
        let ext = extension.to_string_lossy().to_lowercase();
        matches!(ext.as_str(), "mp3" | "wav" | "flac" | "ogg" | "m4a" | "aac")
    } else {
        false
    }
}

pub fn format_duration(seconds: f64) -> String {
    let minutes = (seconds / 60.0) as u32;
    let seconds = (seconds % 60.0) as u32;
    format!("{:02}:{:02}", minutes, seconds)
} 