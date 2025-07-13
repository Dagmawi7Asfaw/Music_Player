use crate::audio::AudioManager;
use crate::playlist::{PlaylistManager, Song};
use egui::{Context, ScrollArea, Ui, RichText, Color32, FontId, Visuals, style::Margin};
use std::sync::Arc;
use tokio::sync::Mutex;
use rfd::FileDialog;
use walkdir::WalkDir;

pub struct MusicPlayerUI {
    volume: f32,
    selected_song_index: Option<usize>,
    is_playing: bool,
    is_paused: bool,
    demo_songs: Vec<Song>,
    selected_songs: Vec<usize>,
    current_position: std::time::Duration,
    total_duration: Option<std::time::Duration>,
    playback_start: Option<std::time::Instant>,
    paused_at: Option<std::time::Duration>,
    pending_next: bool,
    pending_next_time: Option<std::time::Instant>,
}

impl MusicPlayerUI {
    pub fn new() -> Self {
        Self {
            volume: 0.5,
            selected_song_index: None,
            is_playing: false,
            is_paused: false,
            demo_songs: Vec::new(),
            selected_songs: Vec::new(),
            current_position: std::time::Duration::from_secs(0),
            total_duration: None,
            playback_start: None,
            paused_at: None,
            pending_next: false,
            pending_next_time: None,
        }
    }

    pub fn update(
        &mut self,
        ctx: &Context,
        audio_manager: Arc<Mutex<AudioManager>>,
        _playlist_manager: Arc<Mutex<PlaylistManager>>,
    ) {
        // Apply a professional dark theme with accent color
        let mut style = (*ctx.style()).clone();
        style.visuals = Visuals::dark();
        style.visuals.widgets.active.bg_fill = Color32::from_rgb(40, 80, 160); // accent blue
        style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(60, 100, 200);
        style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(30, 30, 40);
        style.visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(24, 24, 28);
        style.visuals.selection.bg_fill = Color32::from_rgb(40, 80, 160);
        style.visuals.selection.stroke = egui::Stroke::new(2.0, Color32::from_rgb(80, 180, 255));
        style.spacing.item_spacing = egui::vec2(12.0, 8.0);
        style.spacing.button_padding = egui::vec2(16.0, 8.0);
        style.visuals.window_rounding = 8.0.into();
        style.visuals.window_shadow = egui::epaint::Shadow::big_dark();
        ctx.set_style(style);

        // Always update playback state and auto-advance
        self.update_playback_state(&audio_manager);

        egui::CentralPanel::default().frame(
            egui::Frame::none().fill(Color32::from_rgb(24, 24, 28)).inner_margin(Margin::same(16.0))
        ).show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(RichText::new("🎵 Rust Music Player").font(FontId::proportional(32.0)).color(Color32::from_rgb(80, 180, 255)));
            });
            ui.add_space(8.0);
            ui.separator();
            ui.columns(2, |columns| {
                self.render_playlist_panel(&mut columns[0]);
                self.render_controls_panel(&mut columns[1], audio_manager.clone());
            });
        });
    }

    fn update_playback_state(&mut self, audio_manager: &Arc<Mutex<AudioManager>>) {
        if self.pending_next {
            if let Some(start) = self.pending_next_time {
                if start.elapsed().as_secs_f32() >= 2.0 {
                    self.pending_next = false;
                    self.pending_next_time = None;
                    self.auto_advance_to_next_song(audio_manager.clone());
                }
            }
            return;
        }
        if let Ok(manager) = audio_manager.try_lock() {
            self.is_playing = manager.is_playing();
            self.is_paused = manager.is_paused();

            // Check if current song has finished and set pending_next
            if self.is_playing && !self.is_paused && manager.is_finished() {
                if let Some(total) = self.total_duration {
                    self.current_position = total;
                }
                self.pending_next = true;
                self.pending_next_time = Some(std::time::Instant::now());
                return;
            }

            // Update progress timer
            if self.is_playing && !self.is_paused {
                self.current_position = manager.get_current_position();
                self.total_duration = manager.get_total_duration();
            }
        }
    }

    fn render_playlist_panel(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.set_width(ui.available_width());
            ui.heading(RichText::new("Playlist").font(FontId::proportional(24.0)).color(Color32::WHITE));
            ui.separator();
            if !self.selected_songs.is_empty() {
                ui.label(RichText::new(format!("Selected: {} songs", self.selected_songs.len())).color(Color32::from_rgb(80, 180, 255)));
            }
            ScrollArea::vertical().max_height(600.0).show(ui, |ui| {
                for (i, song) in self.demo_songs.iter().enumerate() {
                    let selected = self.selected_songs.contains(&i);
                    let label = RichText::new(format!("{} - {}", song.title, song.artist))
                        .font(FontId::proportional(18.0))
                        .color(if selected { Color32::from_rgb(80, 180, 255) } else { Color32::WHITE });
                    let resp = ui.selectable_label(selected, label).on_hover_text("Click to select. Ctrl+Click for multi-select.");
                    if resp.clicked() {
                        if ui.input(|i| i.modifiers.ctrl) {
                            if selected {
                                self.selected_songs.retain(|&x| x != i);
                            } else {
                                self.selected_songs.push(i);
                            }
                        } else {
                            self.selected_songs.clear();
                            self.selected_songs.push(i);
                            self.selected_song_index = Some(i);
                        }
                    }
                }
            });
            ui.separator();
            ui.horizontal(|ui| {
                if ui.add(egui::Button::new(RichText::new("Add Song").font(FontId::proportional(16.0)))).clicked() {
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
                if ui.add(egui::Button::new(RichText::new("Add Folder").font(FontId::proportional(16.0)))).clicked() {
                    if let Some(folder_path) = FileDialog::new().pick_folder() {
                        self.add_folder_songs(&folder_path);
                    }
                }
                if ui.add(egui::Button::new(RichText::new("Remove Selected").font(FontId::proportional(16.0)))).clicked() {
                    self.remove_selected_songs();
                }
                if ui.add(egui::Button::new(RichText::new("Clear All").font(FontId::proportional(16.0)))).clicked() {
                    self.clear_all_songs();
                }
            });
        });
    }

    fn render_controls_panel(&mut self, ui: &mut Ui, audio_manager: Arc<Mutex<AudioManager>>) {
        ui.group(|ui| {
            ui.set_width(ui.available_width());
            ui.heading(RichText::new("Controls").font(FontId::proportional(24.0)).color(Color32::WHITE));
            ui.separator();
            ui.horizontal(|ui| {
                let prev = ui.add(egui::Button::new(RichText::new("⏮ Prev").font(FontId::proportional(16.0))));
                let play_pause_label = if self.is_playing && !self.is_paused {
                    "⏸ Pause"
                } else {
                    "▶ Play"
                };
                let play_pause = ui.add(egui::Button::new(RichText::new(play_pause_label).font(FontId::proportional(16.0))));
                let next = ui.add(egui::Button::new(RichText::new("⏭ Next").font(FontId::proportional(16.0))));
                let stop = ui.add(egui::Button::new(RichText::new("⏹ Stop").font(FontId::proportional(16.0))));
                if prev.clicked() { self.handle_previous(audio_manager.clone()); }
                if play_pause.clicked() { self.handle_play_pause(audio_manager.clone()); }
                if next.clicked() { self.handle_next(audio_manager.clone()); }
                if stop.clicked() { self.handle_stop(audio_manager.clone()); }
            });
            ui.add_space(8.0);
            ui.label(RichText::new("Volume:").font(FontId::proportional(16.0)));
            let volume_slider = ui.add(egui::Slider::new(&mut self.volume, 0.0..=1.0).text("Volume"));
            if volume_slider.changed() {
                self.handle_volume_change(audio_manager.clone());
            }
            ui.separator();
            ui.label(RichText::new("Now Playing:").font(FontId::proportional(16.0)).color(Color32::from_rgb(80, 180, 255)));
            if let Some(idx) = self.selected_song_index {
                let song = &self.demo_songs[idx];
                ui.label(RichText::new(format!("{} - {}", song.title, song.artist)).font(FontId::proportional(18.0)).color(Color32::WHITE));
                ui.separator();
                ui.label(RichText::new("Progress:").font(FontId::proportional(16.0)));
                let (elapsed, frac) = if self.pending_next {
                    let total = self.total_duration.unwrap_or(std::time::Duration::from_secs(1));
                    (total, 1.0)
                } else {
                    let elapsed = if self.is_playing && !self.is_paused {
                        if let Some(start) = self.playback_start {
                            start.elapsed()
                        } else {
                            std::time::Duration::from_secs(0)
                        }
                    } else if self.is_paused {
                        self.paused_at.unwrap_or(std::time::Duration::from_secs(0))
                    } else {
                        std::time::Duration::from_secs(0)
                    };
                    let mut elapsed_secs = elapsed.as_secs_f32();
                    let mut frac = 0.0;
                    if let Some(total) = self.total_duration {
                        let total_secs = total.as_secs_f32();
                        if elapsed_secs > total_secs {
                            elapsed_secs = total_secs;
                        }
                        frac = (elapsed_secs / total_secs).min(1.0);
                    }
                    (std::time::Duration::from_secs_f32(elapsed_secs), frac)
                };
                if self.total_duration.is_some() {
                    ui.add(egui::ProgressBar::new(frac).desired_width(200.0).show_percentage());
                }
                let display_secs = elapsed.as_secs() as u64;
                let current_mins = display_secs / 60;
                let current_secs_remainder = display_secs % 60;
                let total_secs = self.total_duration.map(|d| d.as_secs()).unwrap_or(0);
                let total_mins = total_secs / 60;
                let total_secs_remainder = total_secs % 60;
                ui.label(RichText::new(format!("{:02}:{:02} / {:02}:{:02}", current_mins, current_secs_remainder, total_mins, total_secs_remainder)).font(FontId::proportional(16.0)).color(Color32::WHITE));
            } else {
                ui.label(RichText::new("No song selected").font(FontId::proportional(16.0)).color(Color32::GRAY));
            }
            ui.separator();
            let status = if self.pending_next {
                "⏳ Waiting..."
            } else if self.is_playing && !self.is_paused {
                "▶ Playing"
            } else if self.is_paused {
                "⏸ Paused"
            } else {
                "⏹ Stopped"
            };
            ui.label(RichText::new(format!("Status: {}", status)).font(FontId::proportional(16.0)).color(Color32::from_rgb(80, 180, 255)));
        });
    }

    fn handle_play_pause(&mut self, audio_manager: Arc<Mutex<AudioManager>>) {
        if let Ok(mut manager) = audio_manager.try_lock() {
            if self.is_playing && !self.is_paused {
                // Currently playing, so pause
                manager.pause();
                self.is_paused = true;
                self.is_playing = false;
                if let Some(start) = self.playback_start {
                    self.paused_at = Some(start.elapsed());
                }
            } else if self.is_paused {
                // Currently paused, so resume
                manager.resume();
                self.is_playing = true;
                self.is_paused = false;
                if let Some(paused) = self.paused_at {
                    self.playback_start = Some(std::time::Instant::now() - paused);
                }
                self.paused_at = None;
            } else {
                // Not playing, so start playing selected song
                if let Some(idx) = self.selected_song_index {
                    let song = &self.demo_songs[idx];
                    if let Err(e) = manager.play_file(&song.file_path) {
                        eprintln!("Failed to play file: {}", e);
                    } else {
                        self.is_playing = true;
                        self.is_paused = false;
                        self.playback_start = Some(std::time::Instant::now());
                        self.paused_at = None;
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
            self.current_position = std::time::Duration::from_secs(0);
            self.total_duration = None;
            self.playback_start = None;
            self.paused_at = None;
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
                    self.playback_start = Some(std::time::Instant::now());
                    self.paused_at = None;
                    self.current_position = std::time::Duration::from_secs(0);
                    self.total_duration = manager.get_total_duration();
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
                self.current_position = std::time::Duration::from_secs(0);
                self.total_duration = None;
                self.playback_start = None;
                self.paused_at = None;
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