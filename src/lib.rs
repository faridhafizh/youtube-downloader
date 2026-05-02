use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc;

/// Event yang dikirim selama proses unduhan.
#[derive(Debug, Clone)]
pub enum DownloadEvent {
    Progress {
        percent: f32,
        speed: String,
        filename: String,
    },
    FileStarted(String),
    FileCompleted(String),
    Finished {
        total_files: usize,
    },
    Error(String),
}

/// Opsi untuk mengonfigurasi unduhan.
#[derive(Debug, Clone)]
pub struct DownloadOptions {
    pub url: String,
    pub format: String,
    pub output_dir: String,
    pub is_playlist: bool,
    pub audio_only: bool,
    pub audio_format: Option<String>,
    pub audio_quality: Option<String>,
}

impl Default for DownloadOptions {
    fn default() -> Self {
        Self {
            url: String::new(),
            format: "best".into(),
            output_dir: "./downloads".into(),
            is_playlist: false,
            audio_only: false,
            audio_format: None,
            audio_quality: None,
        }
    }
}

/// Bangun perintah yt-dlp berdasarkan opsi.
pub fn build_command(opts: &DownloadOptions) -> Command {
    let mut cmd = Command::new("yt-dlp");
    cmd.arg("-f").arg(&opts.format);
    cmd.arg("-o")
        .arg(format!("{}/%(title)s.%(ext)s", opts.output_dir));
    cmd.arg("--continue"); // mendukung resume, tanpa --no-part
    if opts.is_playlist {
        cmd.arg("--yes-playlist");
    } else {
        cmd.arg("--no-playlist");
    }
    if opts.audio_only {
        cmd.arg("--extract-audio");
        if let Some(ref af) = opts.audio_format {
            cmd.arg("--audio-format").arg(af);
        }
        if let Some(ref aq) = opts.audio_quality {
            cmd.arg("--audio-quality").arg(aq);
        }
    }
    cmd.arg(&opts.url);
    cmd
}

/// Jalankan unduhan dan kirim event melalui sender.
/// Menggunakan template progres stabil yt-dlp agar parsing tidak mudah rusak.
pub fn download_with_events(
    mut cmd: Command,
    sender: mpsc::Sender<DownloadEvent>,
) -> Result<(), anyhow::Error> {
    cmd.env("PYTHONUNBUFFERED", "1");
    cmd.arg("--color").arg("never");
    cmd.arg("--newline");
    cmd.arg("--no-warnings");
    // Template progres stabil (yt-dlp >= 2023)
    cmd.arg("--progress-template")
        .arg("download:%(progress._percent_str)s|%(progress._speed_str)s|%(info.filename)s");

    let mut child = cmd
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| anyhow::anyhow!("Failed to start yt-dlp: {}", e))?;

    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| anyhow::anyhow!("Could not capture stderr"))?;
    let reader = BufReader::new(stderr);

    let mut completed_files = 0usize;

    for line in reader.lines() {
        let line = line.map_err(|e| anyhow::anyhow!("Read error: {}", e))?;
        if let Some(data) = line.strip_prefix("download:") {
            let parts: Vec<&str> = data.splitn(3, '|').collect();
            if parts.len() == 3 {
                let percent_str = parts[0].trim();
                let speed = parts[1].trim().to_string();
                let filename = parts[2].trim().to_string();

                if let Ok(percent) = percent_str.parse::<f32>() {
                    let p = percent.clamp(0.0, 100.0);
                    let _ = sender.send(DownloadEvent::Progress {
                        percent: p,
                        speed,
                        filename: filename.clone(),
                    });
                    if p >= 100.0 {
                        let _ = sender.send(DownloadEvent::FileCompleted(filename));
                        completed_files += 1;
                    }
                }
            }
        } else if let Some(filename) = line.strip_prefix("[download] Destination: ") {
            let _ = sender.send(DownloadEvent::FileStarted(filename.trim().to_string()));
        }
    }

    let status = child
        .wait()
        .map_err(|e| anyhow::anyhow!("Wait failed: {}", e))?;
    if status.success() {
        let _ = sender.send(DownloadEvent::Finished {
            total_files: completed_files,
        });
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "yt-dlp exited with status {}",
            status.code().unwrap_or(-1)
        ))
    }
}

/// Cek apakah yt-dlp terpasang.
pub fn is_yt_dlp_installed() -> bool {
    Command::new("yt-dlp")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

// ── Preset kualitas audio untuk mode interaktif ──────────────────────
pub enum AudioQualityPreset {
    BestAuto,
    Mp3_320k,
    Mp3_256k,
    Mp3_192k,
    Mp3_128k,
    M4aAac,
}

impl AudioQualityPreset {
    pub fn options() -> &'static [(&'static str, Self)] {
        &[
            ("🎵 Best available audio (auto)", Self::BestAuto),
            ("🔊 MP3 320kbps", Self::Mp3_320k),
            ("🔊 MP3 256kbps", Self::Mp3_256k),
            ("🔊 MP3 192kbps", Self::Mp3_192k),
            ("🔊 MP3 128kbps", Self::Mp3_128k),
            ("🎼 M4A (AAC)", Self::M4aAac),
        ]
    }

    /// Mendapatkan nilai format, audio_format, audio_quality untuk preset ini.
    pub fn to_opts(&self) -> (&str, Option<&str>, Option<&str>) {
        match self {
            Self::BestAuto => ("bestaudio", None, None),
            Self::Mp3_320k => ("bestaudio", Some("mp3"), Some("320k")),
            Self::Mp3_256k => ("bestaudio", Some("mp3"), Some("256k")),
            Self::Mp3_192k => ("bestaudio", Some("mp3"), Some("192k")),
            Self::Mp3_128k => ("bestaudio", Some("mp3"), Some("128k")),
            Self::M4aAac => ("bestaudio", Some("m4a"), None),
        }
    }
}