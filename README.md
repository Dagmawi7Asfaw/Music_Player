# Rust Music Player

A modern, cross-platform music player built with Rust, featuring a clean GUI and robust audio playback capabilities.

## Features

- 🎵 **Audio Playback**: Support for MP3, WAV, FLAC, OGG, and M4A formats
- 🎛️ **Playlist Management**: Create, manage, and organize your music playlists
- 🖱️ **File Picker**: Add songs directly from your disk with a native file dialog
- 🔊 **Volume Control**: Adjust playback volume with a real-time slider
- ⏯️ **Playback Controls**: Play, pause, stop, and navigate between tracks
- 🎨 **Modern GUI**: Clean, responsive interface built with egui
- 🔄 **Cross-Platform**: Runs on Linux, Windows, and macOS

## Screenshots

*Screenshots will be added here once the application is more polished*

## Prerequisites

- **Rust**: Latest stable version (1.70+)
- **Cargo**: Rust's package manager (included with Rust)
- **Audio Backend**: ALSA (Linux), Core Audio (macOS), or WASAPI (Windows)

## Installation

1. **Clone the repository**:

   ```bash
   git clone https://github.com/Dagmawi7Asfaw/Music_Player.git
   cd Music_Player
   ```

2. **Build the project**:

   ```bash
   cargo build --release
   ```

3. **Run the application**:

   ```bash
   cargo run
   ```

## Usage

### Basic Controls

- **Add Song**: Click "Add Song" to open a file picker and select audio files
- **Play/Pause**: Click the play/pause button to control playback
- **Stop**: Click the stop button to halt playback
- **Volume**: Use the slider to adjust playback volume
- **Remove Song**: Select a song and click "Remove Song" to delete it from the playlist

### Playlist Management

- Select songs from the playlist panel on the left
- Songs are displayed with title and artist information
- The currently playing song is highlighted
- Playback status is shown at the bottom of the controls panel

## Project Structure

```
src/
├── main.rs          # Application entry point
├── app.rs           # Main application logic and state management
├── audio.rs         # Audio playback engine using rodio
├── playlist.rs      # Playlist management and file scanning
├── ui.rs            # User interface components using egui
└── utils.rs         # Utility functions and helpers
```

## Dependencies

### Core Dependencies

- **eframe**: egui framework for the GUI
- **rodio**: Cross-platform audio playback
- **serde**: Serialization for playlist persistence
- **anyhow**: Error handling
- **tokio**: Async runtime
- **tracing**: Logging framework

### Audio Support

- **symphonia**: Audio format decoding
- **cpal**: Cross-platform audio I/O

### File Management

- **rfd**: Native file dialogs
- **walkdir**: Directory scanning

## Development

### Building for Development

```bash
cargo build
```

### Running Tests

```bash
cargo test
```

### Code Formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

## Roadmap

### Planned Features

- [ ] **Playlist Persistence**: Save and load playlists
- [ ] **Audio Visualization**: Real-time audio spectrum display
- [ ] **Keyboard Shortcuts**: Global hotkeys for playback control
- [ ] **Search Functionality**: Find songs in large playlists
- [ ] **Audio Effects**: Equalizer and audio filters
- [ ] **Metadata Editing**: Edit song information
- [ ] **Streaming Support**: Play from online sources
- [ ] **Theme Support**: Dark/light mode and custom themes

### Technical Improvements

- [ ] **Async Audio**: Non-blocking audio operations
- [ ] **Memory Optimization**: Efficient handling of large playlists
- [ ] **Error Recovery**: Graceful handling of audio errors
- [ ] **Performance Profiling**: Optimize for large music libraries

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Troubleshooting

### Common Issues

**Audio not playing:**

- Ensure your system's audio is working
- Check that the audio file format is supported
- Verify file permissions

**File picker not working:**

- On Linux, ensure you have a desktop environment running
- On Windows, check that the application has file access permissions

**Build errors:**

- Update Rust to the latest stable version
- Run `cargo clean` and try building again
- Check that all dependencies are properly installed

### Platform-Specific Notes

**Linux:**

- Requires ALSA or PulseAudio
- May need additional audio codecs installed

**Windows:**

- Uses WASAPI for audio output
- May require Visual Studio Build Tools

**macOS:**

- Uses Core Audio
- May require additional permissions for file access

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- **egui**: For the excellent immediate mode GUI framework
- **rodio**: For cross-platform audio playback
- **symphonia**: For comprehensive audio format support
- **Rust Community**: For the amazing ecosystem and tools

## Version History

- **v0.1.0**: Initial release with basic playback and playlist management
  - Basic audio playback (MP3, WAV, FLAC, OGG, M4A)
  - Playlist management with add/remove functionality
  - File picker integration
  - Volume control
  - Play/pause/stop controls

---

*Built with ❤️ using Rust*
