use rust_music_player::MusicPlayerApp;

fn main() -> Result<(), eframe::Error> {
    tracing_subscriber::fmt::init();
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Rust Music Player",
        options,
        Box::new(|_cc| Box::new(MusicPlayerApp::new())),
    )
} 