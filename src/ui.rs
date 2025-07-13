use crate::audio::AudioManager;
use crate::playlist::{PlaylistManager, Song};
use egui::{Context, ScrollArea, Ui};
use std::sync::Arc;
use tokio::sync::Mutex;
use rfd::FileDialog;
use walkdir::WalkDir;

pub struct MusicPlayerUI {
    volume: f32,
    selected_song_index: Option<usize>,
    // Remove unused fields
    is_playing: bool,
    is_paused: bool,
    demo_songs: Vec<Song>,
    selected_songs: Vec<usize>, // Multiple selection support
}

impl MusicPlayerUI {
    pub fn new() -> Self {
        Self {
            volume: 0.5,
            selected_song_index: None,
            is_playing: false,
            is_paused: false,
            demo_songs: Vec::new(), // Start with empty playlist
            selected_songs: Vec::new(),
        }
    }

    pub fn update(
        &mut self,
        ctx: &Context,
        audio_manager: Arc<Mutex<AudioManager>>,
        _playlist_manager: Arc<Mutex<PlaylistManager>>,
    ) {
        self.update_playback_state(&audio_manager);
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🎵 Rust Music Player");
            ui.separator();
            ui.columns(2, |columns| {
                self.render_playlist_panel(&mut columns[0]);
                self.render_controls_panel(&mut columns[1], audio_manager.clone());
            });
        });
    }

    fn update_playback_state(&mut self, audio_manager: &Arc<Mutex<AudioManager>>) {
        if let Ok(manager) = audio_manager.try_lock() {
            self.is_playing = manager.is_playing();
            self.is_paused = manager.is_paused();
            
            // Check if current song has finished and auto-advance
            if self.is_playing && !self.is_paused && manager.is_finished() {
                self.auto_advance_to_next_song(audio_manager.clone());
            }
        }
    }

    fn render_playlist_panel(&mut self, ui: &mut Ui) {
        ui.heading("Playlist");
        ui.separator();
        
        // Show selection info
        if !self.selected_songs.is_empty() {
            ui.label(format!("Selected: {} songs", self.selected_songs.len()));
        }
        
        ScrollArea::vertical().max_height(500.0).show(ui, |ui| {
            for (i, song) in self.demo_songs.iter().enumerate() {
                let selected = self.selected_songs.contains(&i);
                if ui.selectable_label(selected, format!("{} - {}", song.title, song.artist)).clicked() {
                    // Handle selection (single click for single selection, Ctrl+click for multiple)
                    if ui.input(|i| i.modifiers.ctrl) {
                        // Ctrl+click for multiple selection
                        if selected {
                            self.selected_songs.retain(|&x| x != i);
                        } else {
                            self.selected_songs.push(i);
                        }
                    } else {
                        // Single click for single selection
                        self.selected_songs.clear();
                        self.selected_songs.push(i);
                        self.selected_song_index = Some(i);
                    }
                }
            }
        });
        ui.separator();
        ui.horizontal(|ui| {
            if ui.button("Add Song").clicked() {
                if let Some(path) = FileDialog::new()
                    .add_filter("Audio", &["mp3", "wav", "flac", "ogg", "m4a"])
                    .pick_file() {
                    let file_path = path.display().to_string();
                    let title = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "Unknown".to_string());
                    let song = Song {
                        title,
                        artist: "Unknown".to_string(),
                        file_path,
                        duration: None,
                    };
                    self.demo_songs.push(song);
                }
            }
            if ui.button("Add Folder").clicked() {
                if let Some(folder_path) = FileDialog::new()
                    .pick_folder() {
                    self.add_folder_songs(&folder_path);
                }
            }
            if ui.button("Remove Selected").clicked() {
                self.remove_selected_songs();
            }
            if ui.button("Clear All").clicked() {
                self.clear_all_songs();
            }
        });
    }

    fn render_controls_panel(&mut self, ui: &mut Ui, audio_manager: Arc<Mutex<AudioManager>>) {
        ui.heading("Controls");
        ui.separator();
        ui.horizontal(|ui| {
            if ui.button("⏮ Prev").clicked() {
                self.handle_previous(audio_manager.clone());
            }
            
            // Play/Pause button with proper state
            let button_text = if self.is_playing && !self.is_paused {
                "⏸ Pause"
            } else {
                "▶ Play"
            };
            
            if ui.button(button_text).clicked() {
                self.handle_play_pause(audio_manager.clone());
            }
            
            if ui.button("⏭ Next").clicked() {
                self.handle_next(audio_manager.clone());
            }
            if ui.button("⏹ Stop").clicked() {
                self.handle_stop(audio_manager.clone());
            }
        });
        ui.separator();
        ui.label("Volume:");
        let volume_changed = ui.add(egui::Slider::new(&mut self.volume, 0.0..=1.0).text("Volume")).changed();
        if volume_changed {
            self.handle_volume_change(audio_manager.clone());
        }
        ui.separator();
        ui.label("Now Playing:");
        if let Some(idx) = self.selected_song_index {
            let song = &self.demo_songs[idx];
            ui.label(format!("{} - {}", song.title, song.artist));
        } else {
            ui.label("No song selected");
        }
        
        // Show playback status
        ui.separator();
        let status = if self.is_playing && !self.is_paused {
            "▶ Playing"
        } else if self.is_paused {
            "⏸ Paused"
        } else {
            "⏹ Stopped"
        };
        ui.label(format!("Status: {}", status));
    }

    fn handle_play_pause(&mut self, audio_manager: Arc<Mutex<AudioManager>>) {
        if let Ok(mut manager) = audio_manager.try_lock() {
            if self.is_playing && !self.is_paused {
                // Currently playing, so pause
                manager.pause();
                self.is_paused = true;
                self.is_playing = false;
            } else if self.is_paused {
                // Currently paused, so resume
                manager.resume();
                self.is_playing = true;
                self.is_paused = false;
            } else {
                // Not playing, so start playing selected song
                if let Some(idx) = self.selected_song_index {
                    let song = &self.demo_songs[idx];
                    if let Err(e) = manager.play_file(&song.file_path) {
                        eprintln!("Failed to play file: {}", e);
                    } else {
                        self.is_playing = true;
                        self.is_paused = false;
                    }
                }
            }
        }
    }

    fn handle_stop(&mut self, audio_manager: Arc<Mutex<AudioManager>>) {
        if let Ok(mut manager) = audio_manager.try_lock() {
            manager.stop();
            self.is_playing = false;
            self.is_paused = false;
        }
    }

    fn handle_volume_change(&mut self, audio_manager: Arc<Mutex<AudioManager>>) {
        if let Ok(mut manager) = audio_manager.try_lock() {
            manager.set_volume(self.volume);
        }
    }

    fn handle_previous(&mut self, audio_manager: Arc<Mutex<AudioManager>>) {
        if self.demo_songs.is_empty() {
            return;
        }

        // If no song is selected, select the last song
        if self.selected_song_index.is_none() {
            self.selected_song_index = Some(self.demo_songs.len() - 1);
        } else {
            // Move to previous song, wrapping around to the end
            let current_index = self.selected_song_index.unwrap();
            if current_index == 0 {
                self.selected_song_index = Some(self.demo_songs.len() - 1);
            } else {
                self.selected_song_index = Some(current_index - 1);
            }
        }

        // Auto-play the selected song if we were already playing
        if self.is_playing && !self.is_paused {
            self.play_selected_song(audio_manager);
        }
    }

    fn handle_next(&mut self, audio_manager: Arc<Mutex<AudioManager>>) {
        if self.demo_songs.is_empty() {
            return;
        }

        // If no song is selected, select the first song
        if self.selected_song_index.is_none() {
            self.selected_song_index = Some(0);
        } else {
            // Move to next song, wrapping around to the beginning
            let current_index = self.selected_song_index.unwrap();
            if current_index == self.demo_songs.len() - 1 {
                self.selected_song_index = Some(0);
            } else {
                self.selected_song_index = Some(current_index + 1);
            }
        }

        // Auto-play the selected song if we were already playing
        if self.is_playing && !self.is_paused {
            self.play_selected_song(audio_manager);
        }
    }

    fn play_selected_song(&mut self, audio_manager: Arc<Mutex<AudioManager>>) {
        if let Some(idx) = self.selected_song_index {
            if let Ok(mut manager) = audio_manager.try_lock() {
                let song = &self.demo_songs[idx];
                if let Err(e) = manager.play_file(&song.file_path) {
                    eprintln!("Failed to play file: {}", e);
                } else {
                    self.is_playing = true;
                    self.is_paused = false;
                }
            }
        }
    }

    fn auto_advance_to_next_song(&mut self, audio_manager: Arc<Mutex<AudioManager>>) {
        if self.demo_songs.is_empty() {
            return;
        }

        // If no song is selected, select the first song
        if self.selected_song_index.is_none() {
            self.selected_song_index = Some(0);
            self.play_selected_song(audio_manager);
            return;
        }

        let current_index = self.selected_song_index.unwrap();
        
        // Check if there's a next song
        if current_index < self.demo_songs.len() - 1 {
            // Move to next song
            self.selected_song_index = Some(current_index + 1);
            self.play_selected_song(audio_manager);
        } else {
            // No more songs, stop playback
            if let Ok(mut manager) = audio_manager.try_lock() {
                manager.stop();
                self.is_playing = false;
                self.is_paused = false;
            }
        }
    }

    fn add_folder_songs(&mut self, folder_path: &std::path::Path) {
        let mut added_songs = Vec::new();
        let supported_extensions = ["mp3", "wav", "flac", "ogg", "m4a"];
        let walkdir = WalkDir::new(folder_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file() && supported_extensions.contains(&e.path().extension().unwrap_or_default().to_string_lossy().to_string().as_str()));

        for entry in walkdir {
            let path = entry.path();
            let title = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "Unknown".to_string());
            let artist = "Unknown".to_string(); // No artist info available from folder
            let file_path = path.display().to_string();
            let song = Song {
                title,
                artist,
                file_path,
                duration: None,
            };
            added_songs.push(song);
        }
        self.demo_songs.extend(added_songs);
    }

    fn remove_selected_songs(&mut self) {
        if self.selected_songs.is_empty() {
            return;
        }
        let mut new_songs = Vec::new();
        for (i, song) in self.demo_songs.iter().enumerate() {
            if !self.selected_songs.contains(&i) {
                new_songs.push(song.clone());
            }
        }
        self.demo_songs = new_songs;
        self.selected_songs.clear();
        self.selected_song_index = None;
    }

    fn clear_all_songs(&mut self) {
        self.demo_songs.clear();
        self.selected_songs.clear();
        self.selected_song_index = None;
    }
} 