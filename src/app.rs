use crate::audio::AudioManager;
use crate::playlist::PlaylistManager;
use crate::ui::MusicPlayerUI;
use egui::Context;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct MusicPlayerApp {
    ui: MusicPlayerUI,
    audio_manager: Arc<Mutex<AudioManager>>,
    playlist_manager: Arc<Mutex<PlaylistManager>>,
}

impl MusicPlayerApp {
    pub fn new() -> Self {
        let audio_manager = Arc::new(Mutex::new(AudioManager::new()));
        let playlist_manager = Arc::new(Mutex::new(PlaylistManager::new()));
        
        Self {
            ui: MusicPlayerUI::new(),
            audio_manager,
            playlist_manager,
        }
    }
}

impl eframe::App for MusicPlayerApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Update the UI
        self.ui.update(
            ctx,
            self.audio_manager.clone(),
            self.playlist_manager.clone(),
        );
    }
} 