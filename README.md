# YT Downloader

A modern, user-friendly YouTube downloader written in Rust that wraps yt-dlp with beautiful terminal UI, interactive menus, and automation support.

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![yt-dlp](https://img.shields.io/badge/yt--dlp-enabled-blue?style=for-the-badge)

## ✨ Features

- 🎬 **Beautiful Interactive Mode** - menu driven interface with no command line arguments required
- 🤖 **Automation Mode** - full CLI support for scripting and batch operations
- 📥 **Single Video Downloads**
- 📜 **Full Playlist Downloads**
- 🎵 **Audio Only Extraction** - MP3 / M4A with selectable quality presets
- 📊 **Format Inspection** - list and select exact available formats
- ⚡ **Real-time Progress Bar** - animated download status with speed, ETA and elapsed time
- ✅ **Resume Support** - continues interrupted downloads automatically
- 🎨 **Colorful Terminal UI** - with proper styling, icons and visual feedback

## 📋 Prerequisites

This project requires `yt-dlp` to be installed and available in your system PATH:

- Download from [yt-dlp official repository](https://github.com/yt-dlp/yt-dlp)
- Or install via package manager:
  ```bash
  # Windows (winget)
  winget install yt-dlp.yt-dlp

  # Linux
  sudo apt install yt-dlp

  # macOS
  brew install yt-dlp
  ```

## 🚀 Installation & Build

```bash
# Clone the repository
git clone https://github.com/faridhafizh/youtube-downloader.git
cd youtube-downloader

# Build release version
cargo build --release

# The binary will be available at:
./target/release/yt-downloader
```

## 💡 Usage

### Interactive Mode (Default)
Just run without any arguments to get the full menu interface:
```bash
yt-downloader
```

Options available:
1. Download a single video
2. Download audio only (MP3/M4A)
3. Download an entire playlist
4. List available formats and choose one
5. Change download directory
6. Exit

### Automation Mode
For scripting or headless usage, pass the URL directly:

```bash
# Basic video download
yt-downloader --url "https://www.youtube.com/watch?v=dQw4w9WgXcQ"

# Download entire playlist
yt-downloader --url "https://www.youtube.com/playlist?list=PL..." --playlist

# Custom output directory
yt-downloader --url "https://www.youtube.com/watch?v=dQw4w9WgXcQ" --output-dir "./Videos"

# Specific format
yt-downloader --url "https://www.youtube.com/watch?v=dQw4w9WgXcQ" --format "137+140"

# Force interactive mode even with URL provided
yt-downloader --url "https://www.youtube.com/watch?v=dQw4w9WgXcQ" --interactive
```

## ⚙️ Command Line Options

| Argument | Short | Description | Default |
|----------|-------|-------------|---------|
| `--url` | `-u` | Video or Playlist URL for automation mode | - |
| `--format` | `-f` | Download format selector | `best` |
| `--output-dir` | `-d` | Output directory for downloaded files | `./downloads` |
| `--playlist` | `-p` | Enable full playlist download | `false` |
| `--interactive` | `-i` | Force interactive mode | `false` |
| `--help` | `-h` | Print help information | - |
| `--version` | `-V` | Print version information | - |

## 🔧 How it Works

This application acts as a modern user interface wrapper around the excellent `yt-dlp` tool. It:
1. Parses yt-dlp output in real-time
2. Provides a beautiful animated progress bar
3. Handles file naming and directory creation
4. Adds user friendly interactive menus
5. Automatically handles common download options
6. Provides clear error messages and status updates

All actual downloading logic is handled by yt-dlp, so you get all the features, compatibility and updates from that project with an improved terminal experience.

## 📦 Dependencies

- `clap` - Command line argument parsing
- `dialoguer` - Interactive terminal menus
- `console` - Terminal styling and colors
- `indicatif` - Progress bar implementation
- `regex` - Output parsing

## 📝 License

This project is open source software.