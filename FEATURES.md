# FEATURES.md

## 1. Download Management & Control

- **Pause / Resume / Cancel**  
  Add controls for each active download so users can interrupt, continue later, or stop tasks entirely.

- **Queue System**  
  Allow users to add new URLs while downloads are in progress. New items are queued automatically.

- **Concurrent Download Limit**  
  Let users configure how many downloads run simultaneously (default: 2–3).

- **Retry Failed Downloads**  
  Support manual retry or automatic retry with exponential backoff.

- **History Panel**  
  Store previously downloaded URLs with status, timestamps, and results.

---

## 2. Format & Quality Selection

- **Preset Dropdown**  
  Quick presets such as:
  - Best video + audio
  - 1080p
  - 720p
  - Audio only (MP3)
  - Audio only (Opus)

- **Advanced Format String Input**  
  Allow custom `yt-dlp` format strings, for example:

bestvideo[height<=1080]+bestaudio/best


- **Fetch & Display Available Formats**  
Show a selectable table containing:

- Resolution
- Codec
- File size
- Stream type

- **Audio Extraction Options**  
Choose output format and bitrate:

- MP3
- AAC
- Opus
- FLAC

---

## 3. Playlist Handling

- **Playlist Toggle**  
Checkbox to enable full playlist download.

- **Select Items / Range**  
Allow custom ranges:

1-5,8,10-12


- **Archive Mode**  
Use `--download-archive` to skip already downloaded videos.

- **Show Playlist Info**  
Display playlist title and total video count before starting.

---

## 4. Output Customisation

- **Output Directory Picker**  
Use native folder dialog (`rfd` crate) instead of plain text path input.

- **Custom Filename Template**  
Examples:

%(playlist)s/%(title)s.%(ext)s


- **Overwrite Behaviour**

- Skip existing files
- Overwrite existing files
- Auto-rename duplicates

- **Subfolder Organisation**

- Group by channel
- Group by playlist

---

## 5. Advanced yt-dlp Features (GUI Toggles)

- **Subtitle Download**

- Enable subtitles
- Choose language (`en`, `all`, etc.)

- **Embed Metadata / Thumbnail**

- Embed metadata
- Embed thumbnail
- Embed chapters
- Save `info.json`

- **SponsorBlock**

Enable skipping sponsor segments:

--sponsorblock-mark all


- **Cookie File Support**  
Load cookies for age-restricted or private content.

- **Rate Limiting**  
Cap speed:

--limit-rate 5M


- **Proxy & Authentication**

- Proxy URL
- Username
- Password

---

## 6. UI / UX Polish

- **Dark Mode Toggle**  
Use `ctx.set_visuals()` for light/dark theme switching.

- **Drag-and-Drop URL**  
Accept dropped links or text from browser.

- **Clipboard Monitoring**  
Detect YouTube URLs copied to clipboard and suggest download.

- **Thumbnail Preview**  
Fetch and show thumbnail when URL is pasted.

- **Log Filtering**

- Search box
- Error filter
- Warning filter
- Info filter

- **System Tray**

- Minimise to tray
- Completion notifications

- **Resizable Panels**  
Use:

- `egui::TopBottomPanel`
- `egui::SidePanel`

Suggested layout:

- URL input
- Download queue
- Logs

---

## 7. Settings & Persistence

- **Save Preferences**  
Persist settings using JSON / TOML via `serde`.

Examples:

- Output directory
- Default format
- Audio-only mode
- Concurrent downloads
- Theme

- **Import / Export Settings**  
Backup or share config files.

- **Remember Window State**  
Restore last size and position on launch.

---

## 8. Performance & Architecture

- **Async Downloads**  
Run `yt-dlp` in background thread/task using:

- `tokio`
- channels for progress updates

- **Efficient Repainting**

Replace constant repaint:

ctx.request_repaint()


With smarter refresh:

ctx.request_repaint_after(Duration::from_millis(100))


Or repaint only when progress changes.

- **Bundled yt-dlp Updates**  
Check for updates and allow downloading latest binary.

- **Error Reporting**

Show clear popups / colored logs for:

- Missing `yt-dlp`
- Network failure
- Invalid format
- Command execution errors

---

## 9. Extra Goodies

- **Download Scheduler**

- Start at specific time
- Start when PC is idle

- **Audio Normalisation**

Use FFmpeg loudness normalization:

loudnorm


- **Video Trimming**

Allow start/end timestamps to download only a segment.

- **Browser Integration**

Run local HTTP server for browser extension requests.

- **CLI Support**

Allow:

app.exe "https://youtube.com/..."


to open GUI directly with URL prefilled.

---

# Recommended Priority Roadmap

## Phase 1 (Core Stability)

- Queue system
- Concurrent downloads
- Pause / Resume / Cancel
- Async architecture
- Settings persistence

## Phase 2 (Power Features)

- Format picker
- Playlist tools
- Thumbnail preview
- History panel
- Retry system

## Phase 3 (Premium UX)

- System tray
- Scheduler
- Browser extension
- Clipboard monitoring
- Update manager