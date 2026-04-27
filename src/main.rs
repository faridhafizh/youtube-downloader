use clap::Parser;
use console::style;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// A simple YouTube downloader using yt-dlp
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Video/Playlist URL (automation mode)
    #[arg(short, long)]
    url: Option<String>,

    /// Download format (e.g. 'best', '137+140', 'mp4')
    #[arg(short, long, default_value = "best")]
    format: String,

    /// Output directory
    #[arg(short = 'd', long, default_value = "./downloads")]
    output_dir: PathBuf,

    /// Download entire playlist
    #[arg(short = 'p', long, default_value_t = false)]
    playlist: bool,

    /// Force interactive mode even if URL is provided
    #[arg(short = 'i', long, default_value_t = false)]
    interactive: bool,
}

fn main() {
    let args = Args::parse();

    // Check if yt-dlp is available
    if !is_yt_dlp_installed() {
        eprintln!("{} 'yt-dlp' is not installed or not found in PATH.", style("Error:").red().bold());
        eprintln!("Install it from https://github.com/yt-dlp/yt-dlp");
        return;
    }

    // Ensure output directory exists
    if let Err(e) = std::fs::create_dir_all(&args.output_dir) {
        eprintln!("Failed to create output directory: {}", e);
        return;
    }

    // Decide mode: automation if URL provided and not forced interactive
    if args.url.is_some() && !args.interactive {
        run_automation(&args);
    } else {
        run_interactive();
    }
}

fn is_yt_dlp_installed() -> bool {
    Command::new("yt-dlp")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Automation mode: use the provided CLI arguments.
fn run_automation(args: &Args) {
    let url = args.url.as_ref().unwrap();
    let mut cmd = Command::new("yt-dlp");

    cmd.arg("-f").arg(&args.format);
    cmd.arg("-o").arg(args.output_dir.join("%(title)s.%(ext)s"));
    cmd.arg("--continue");
    cmd.arg("--no-part");
    if args.playlist {
        cmd.arg("--yes-playlist");
    } else {
        cmd.arg("--no-playlist");
    }
    cmd.arg(url);

    // Use the new progress bar downloader
    if let Err(e) = run_with_progress(cmd) {
        eprintln!("{} {}", style("Error:").red().bold(), e);
    }
}

/// Interactive mode: step-by-step menu using dialoguer.
fn run_interactive() {
    let theme = ColorfulTheme::default();

    loop {
        // Clean header
        println!("\n{}", style("🎬 YouTube Downloader (interactive)").bold().cyan());
        println!("{}", style("──────────────────────────────────────").dim());

    let options = &[
        "📥 Download a single video",
        "🎵 Download audio only (MP3/M4A)",
        "📜 Download a playlist",
        "📊 List available formats (and choose one)",
        "📁 Change download directory",
        "🚪 Exit",
    ];

        let selection = Select::with_theme(&theme)
            .with_prompt("What would you like to do?")
            .items(options)
            .default(0)
            .interact()
            .unwrap();

        match selection {
            0 => download_video(false, false),
            1 => download_audio_only(),
            2 => download_video(true, false),
            3 => list_and_download_format(),
            4 => {
                println!(
                    "{} Current download directory is set via the --output-dir flag (default ./downloads).",
                    style("ℹ").cyan()
                );
                println!("Use that flag in automation mode to change it permanently.");
            }
            5 => {
                println!("{}", style("👋 Goodbye!").green());
                break;
            }
            _ => unreachable!(),
        }
    }
}

/// Ask for URL and start download (single or playlist).
fn download_video(is_playlist: bool, audio_only: bool) {
    let theme = ColorfulTheme::default();

    let url: String = Input::with_theme(&theme)
        .with_prompt("Enter video/playlist URL")
        .interact_text()
        .unwrap();

    let format: String = if audio_only {
        let audio_options = &[
            "🎵 Best available audio (auto)",
            "🔊 MP3 320kbps",
            "🔊 MP3 256kbps",
            "🔊 MP3 192kbps",
            "🔊 MP3 128kbps",
            "🎼 M4A (AAC)",
        ];
        
        let audio_quality = Select::with_theme(&theme)
            .with_prompt("Select audio quality")
            .items(audio_options)
            .default(0)
            .interact()
            .unwrap();
            
        match audio_quality {
            0 => "bestaudio".into(),
            1 => "bestaudio[ext=mp3]/bestaudio --audio-format mp3 --audio-quality 320k".into(),
            2 => "bestaudio[ext=mp3]/bestaudio --audio-format mp3 --audio-quality 256k".into(),
            3 => "bestaudio[ext=mp3]/bestaudio --audio-format mp3 --audio-quality 192k".into(),
            4 => "bestaudio[ext=mp3]/bestaudio --audio-format mp3 --audio-quality 128k".into(),
            5 => "bestaudio[ext=m4a]/bestaudio".into(),
            _ => "bestaudio".into()
        }
    } else {
        Input::with_theme(&theme)
            .with_prompt("Enter format (e.g. 'best', '18', or leave empty for default 'best')")
            .default("best".into())
            .interact_text()
            .unwrap()
    };

    let output_dir: String = Input::with_theme(&theme)
        .with_prompt("Output directory")
        .default("./downloads".into())
        .interact_text()
        .unwrap();

    let mut cmd = Command::new("yt-dlp");
    cmd.arg("-f").arg(&format);
    cmd.arg("-o").arg(format!("{}/%(title)s.%(ext)s", output_dir));
    cmd.arg("--continue");
    cmd.arg("--no-part");
    if !is_playlist {
        cmd.arg("--no-playlist");
    }
    cmd.arg(&url);

    if audio_only {
        cmd.arg("--extract-audio");
        cmd.arg("--audio-quality").arg("0");
    }

    if let Err(e) = run_with_progress(cmd) {
        eprintln!("{} {}", style("Download failed:").red().bold(), e);
    }
}

/// Download audio only mode handler
fn download_audio_only() {
    let theme = ColorfulTheme::default();
    
    let download_type = Select::with_theme(&theme)
        .with_prompt("What do you want to download?")
        .items(&["Single video audio", "Full playlist audio"])
        .default(0)
        .interact()
        .unwrap();
        
    download_video(download_type == 1, true);
}

/// List available formats for a URL, then optionally download with selected format.
fn list_and_download_format() {
    let theme = ColorfulTheme::default();

    let url: String = Input::with_theme(&theme)
        .with_prompt("Enter the video URL to inspect formats")
        .interact_text()
        .unwrap();

    println!("{}", style("🔍 Fetching available formats...").cyan());
    let output = Command::new("yt-dlp")
        .arg("--list-formats")
        .arg(&url)
        .output()
        .expect("Failed to run yt-dlp --list-formats");

    if !output.status.success() {
        eprintln!("{} Could not retrieve formats. Is the URL valid?", style("Error:").red().bold());
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("{}", stdout);

    let download_now = Confirm::with_theme(&theme)
        .with_prompt("Do you want to download this video with a specific format code?")
        .default(true)
        .interact()
        .unwrap();

    if download_now {
        let format_code: String = Input::with_theme(&theme)
            .with_prompt("Enter the format code (e.g. 137+140)")
            .interact_text()
            .unwrap();

        let output_dir: String = Input::with_theme(&theme)
            .with_prompt("Output directory")
            .default("./downloads".into())
            .interact_text()
            .unwrap();

        let mut cmd = Command::new("yt-dlp");
        cmd.arg("-f").arg(&format_code);
        cmd.arg("-o").arg(format!("{}/%(title)s.%(ext)s", output_dir));
        cmd.arg("--continue");
        cmd.arg("--no-part");
        cmd.arg("--no-playlist");
        cmd.arg(&url);

        if let Err(e) = run_with_progress(cmd) {
            eprintln!("{} {}", style("Download failed:").red().bold(), e);
        }
    }
}

// ── Progress bar helper ──────────────────────────────────────────────

/// Spawns the given yt-dlp command and displays an animated progress bar
/// by parsing its stderr output.
fn run_with_progress(mut cmd: Command) -> Result<(), String> {
    let pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.magenta.bold} {elapsed_precise:.dim} [{bar:60}] \
                {percent:>3}% {speed:>10} | ETA: {eta:>5} | {msg:.dim}"
            )
            .unwrap()
            .progress_chars("█▓▒░ "),
    );
    pb.set_message("Initializing download...");

    let mut child = cmd
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start yt-dlp: {}", e))?;

    let stderr = child.stderr.take().ok_or("Could not capture stderr")?;
    let reader = BufReader::new(stderr);

    // Regex patterns used for parsing
    let re_percent = Regex::new(r"(\d+\.?\d*)%").unwrap();
    let re_speed = Regex::new(r"(\d+\.?\d*\s*[KM]iB/s|\d+\.?\d*\s*[kK]B/s)").unwrap();
    let re_destination = Regex::new(r"\[download\] Destination: (.+)").unwrap();

    let mut file_count = 0;
    let mut current_file = String::new();

    for line in reader.lines() {
        let line = line.map_err(|e| format!("Read error: {}", e))?;

        // Look for a destination filename → update the bar message
        if let Some(caps) = re_destination.captures(&line) {
            let fname = caps.get(1).unwrap().as_str();
            current_file = fname.to_string();
            file_count += 1;

            let filename = std::path::Path::new(fname)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(fname);

            pb.set_message(format!("📄 {}", filename));
        }

        // Parse download speed
        if let Some(caps) = re_speed.captures(&line) {
            let speed = caps.get(1).unwrap().as_str();
            pb.set_message(speed.to_string());
        }

        // Look for a percentage
        if let Some(caps) = re_percent.captures(&line) {
            let percent_str = caps.get(1).unwrap().as_str();
            if let Ok(percent) = percent_str.parse::<f64>() {
                let pos = percent.min(100.0).max(0.0) as u64;
                pb.set_position(pos);

                if pos >= 100 {
                    pb.println(format!(" ✅  File completed: {}", current_file));
                    pb.reset();
                    pb.set_message("Waiting for next file...");
                }
            }
        }
    }

    let status = child.wait().map_err(|e| format!("Wait failed: {}", e))?;

    if status.success() {
        pb.finish_with_message("✨ Download completed successfully");
        println!("\n{}", style("───────────────────────────────────────────────────────").dim());
        println!(" {} Total files downloaded: {}", style("✅").green().bold(), file_count);
        println!(" {} Total time elapsed:    {:?}", style("⏱️").cyan(), pb.elapsed());
        println!("{}", style("───────────────────────────────────────────────────────").dim());
        println!();
        Ok(())
    } else {
        pb.finish_with_message("❌ Download failed");
        Err(format!(
            "yt-dlp exited with status {}",
            status.code().unwrap_or(-1)
        ))
    }
}
