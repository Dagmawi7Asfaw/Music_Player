use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Song {
    pub title: String,
    pub artist: String,
    pub file_path: String,
    pub duration: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    pub name: String,
    pub songs: Vec<Song>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl Playlist {
    pub fn new(name: String) -> Self {
        Self {
            name,
            songs: Vec::new(),
            created_at: chrono::Utc::now(),
        }
    }

    pub fn add_song(&mut self, song: Song) {
        self.songs.push(song);
    }

    pub fn remove_song(&mut self, index: usize) -> Option<Song> {
        if index < self.songs.len() {
            Some(self.songs.remove(index))
        } else {
            None
        }
    }

    pub fn get_song(&self, index: usize) -> Option<&Song> {
        self.songs.get(index)
    }

    pub fn len(&self) -> usize {
        self.songs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.songs.is_empty()
    }
}

pub struct PlaylistManager {
    playlists: HashMap<String, Playlist>,
    current_playlist: Option<String>,
}

impl PlaylistManager {
    pub fn new() -> Self {
        Self {
            playlists: HashMap::new(),
            current_playlist: None,
        }
    }

    pub fn create_playlist(&mut self, name: String) -> Result<()> {
        if self.playlists.contains_key(&name) {
            return Err(anyhow::anyhow!("Playlist '{}' already exists", name));
        }
        
        let playlist = Playlist::new(name.clone());
        self.playlists.insert(name.clone(), playlist);
        self.current_playlist = Some(name.clone());
        info!("Created playlist: {}", name);
        Ok(())
    }

    pub fn add_song_to_current_playlist(&mut self, song: Song) -> Result<()> {
        if let Some(playlist_name) = &self.current_playlist {
            if let Some(playlist) = self.playlists.get_mut(playlist_name) {
                playlist.add_song(song);
                info!("Added song to playlist: {}", playlist_name);
                return Ok(());
            }
        }
        Err(anyhow::anyhow!("No current playlist selected"))
    }

    pub fn remove_song_from_current_playlist(&mut self, index: usize) -> Result<Song> {
        if let Some(playlist_name) = &self.current_playlist {
            if let Some(playlist) = self.playlists.get_mut(playlist_name) {
                if let Some(song) = playlist.remove_song(index) {
                    info!("Removed song from playlist: {}", playlist_name);
                    return Ok(song);
                }
            }
        }
        Err(anyhow::anyhow!("Failed to remove song"))
    }

    pub fn get_current_playlist(&self) -> Option<&Playlist> {
        self.current_playlist
            .as_ref()
            .and_then(|name| self.playlists.get(name))
    }

    pub fn get_current_playlist_mut(&mut self) -> Option<&mut Playlist> {
        self.current_playlist
            .as_ref()
            .and_then(|name| self.playlists.get_mut(name))
    }

    pub fn set_current_playlist(&mut self, name: &str) -> Result<()> {
        if self.playlists.contains_key(name) {
            self.current_playlist = Some(name.to_string());
            info!("Set current playlist: {}", name);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Playlist '{}' not found", name))
        }
    }

    pub fn get_playlist_names(&self) -> Vec<String> {
        self.playlists.keys().cloned().collect()
    }

    pub fn scan_music_directory(&mut self, directory: &str) -> Result<Vec<Song>> {
        let mut songs = Vec::new();
        
        for entry in WalkDir::new(directory)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            if let Some(extension) = path.extension() {
                let ext = extension.to_string_lossy().to_lowercase();
                if matches!(ext.as_str(), "mp3" | "wav" | "flac" | "ogg" | "m4a") {
                    if let Some(file_name) = path.file_stem() {
                        let title = file_name.to_string_lossy().to_string();
                        let file_path = path.to_string_lossy().to_string();
                        
                        let song = Song {
                            title,
                            artist: "Unknown".to_string(),
                            file_path,
                            duration: None,
                        };
                        
                        songs.push(song);
                    }
                }
            }
        }
        
        info!("Scanned {} songs from directory: {}", songs.len(), directory);
        Ok(songs)
    }

    pub fn save_playlist(&self, name: &str, file_path: &str) -> Result<()> {
        if let Some(playlist) = self.playlists.get(name) {
            let json = serde_json::to_string_pretty(playlist)?;
            std::fs::write(file_path, json)?;
            info!("Saved playlist '{}' to {}", name, file_path);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Playlist '{}' not found", name))
        }
    }

    pub fn load_playlist(&mut self, file_path: &str) -> Result<()> {
        let content = std::fs::read_to_string(file_path)?;
        let playlist: Playlist = serde_json::from_str(&content)?;
        self.playlists.insert(playlist.name.clone(), playlist.clone());
        info!("Loaded playlist '{}' from {}", playlist.name, file_path);
        Ok(())
    }
} 