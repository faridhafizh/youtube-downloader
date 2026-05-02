use anyhow::Context;
use clap::Parser;
use console::style;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use yt_downloader::{
    build_command, download_with_events, is_yt_dlp_installed, AudioQualityPreset, DownloadEvent,
    DownloadOptions,
};

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

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if !is_yt_dlp_installed() {
        eprintln!(
            "{} 'yt-dlp' is not installed or not found in PATH.",
            style("Error:").red().bold()
        );
        eprintln!("Install it from https://github.com/yt-dlp/yt-dlp");
        std::process::exit(1);
    }

    std::fs::create_dir_all(&args.output_dir)
        .with_context(|| format!("Failed to create output directory {:?}", args.output_dir))?;

    if args.url.is_some() && !args.interactive {
        run_automation(&args)
    } else {
        run_interactive()
    }
}

fn run_automation(args: &Args) -> anyhow::Result<()> {
    let url = args.url.as_ref().unwrap();
    let opts = DownloadOptions {
        url: url.clone(),
        format: args.format.clone(),
        output_dir: args.output_dir.to_string_lossy().to_string(),
        is_playlist: args.playlist,
        ..Default::default()
    };
    let cmd = build_command(&opts);
    run_with_progress_cli(cmd)?;
    Ok(())
}

fn run_interactive() -> anyhow::Result<()> {
    let theme = ColorfulTheme::default();
    let mut current_dir = PathBuf::from("./downloads");

    loop {
        println!(
            "\n{}",
            style("🎬 YouTube Downloader (interactive)").bold().cyan()
        );
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
            .interact()?;

        match selection {
            0 => download_video_cli(&theme, false, false, &mut current_dir)?,
            1 => download_audio_only_cli(&theme, &mut current_dir)?,
            2 => download_video_cli(&theme, true, false, &mut current_dir)?,
            3 => list_and_download_format_cli(&theme, &mut current_dir)?,
            4 => {
                let new_dir: String = Input::with_theme(&theme)
                    .with_prompt("New download directory")
                    .default(current_dir.to_string_lossy().to_string())
                    .interact_text()?;
                current_dir = PathBuf::from(new_dir);
                println!(
                    "{} Directory changed to {}",
                    style("✔").green(),
                    current_dir.display()
                );
            }
            5 => {
                println!("{}", style("👋 Goodbye!").green());
                break;
            }
            _ => unreachable!(),
        }
    }
    Ok(())
}

fn download_video_cli(
    theme: &ColorfulTheme,
    is_playlist: bool,
    audio_only: bool,
    current_dir: &mut PathBuf,
) -> anyhow::Result<()> {
    let url: String = Input::with_theme(theme)
        .with_prompt("Enter video/playlist URL")
        .interact_text()?;

    let (format, audio_format, audio_quality) = if audio_only {
        let presets = AudioQualityPreset::options();
        let names: Vec<&str> = presets.iter().map(|(name, _)| *name).collect();
        let idx = Select::with_theme(theme)
            .with_prompt("Select audio quality")
            .items(&names)
            .default(0)
            .interact()?;
        let (fmt, af, aq) = presets[idx].1.to_opts();
        (fmt.to_string(), af.map(String::from), aq.map(String::from))
    } else {
        let fmt: String = Input::with_theme(theme)
            .with_prompt("Enter format (e.g. 'best', '18', or leave empty for default 'best')")
            .default("best".into())
            .interact_text()?;
        (fmt, None, None)
    };

    let dir: String = Input::with_theme(theme)
        .with_prompt("Output directory")
        .default(current_dir.to_string_lossy().to_string())
        .interact_text()?;
    *current_dir = PathBuf::from(&dir);

    let opts = DownloadOptions {
        url,
        format,
        output_dir: dir,
        is_playlist,
        audio_only,
        audio_format,
        audio_quality,
    };

    let cmd = build_command(&opts);
    run_with_progress_cli(cmd)?;
    Ok(())
}

fn download_audio_only_cli(
    theme: &ColorfulTheme,
    current_dir: &mut PathBuf,
) -> anyhow::Result<()> {
    let items = &["Single video audio", "Full playlist audio"];
    let idx = Select::with_theme(theme)
        .with_prompt("What do you want to download?")
        .items(items)
        .default(0)
        .interact()?;
    download_video_cli(theme, idx == 1, true, current_dir)
}

fn list_and_download_format_cli(
    theme: &ColorfulTheme,
    current_dir: &mut PathBuf,
) -> anyhow::Result<()> {
    let url: String = Input::with_theme(theme)
        .with_prompt("Enter the video URL to inspect formats")
        .interact_text()?;

    println!("{}", style("🔍 Fetching available formats...").cyan());
    let output = Command::new("yt-dlp")
        .arg("--list-formats")
        .arg(&url)
        .output()
        .context("Failed to run yt-dlp --list-formats")?;

    if !output.status.success() {
        eprintln!(
            "{} Could not retrieve formats. Is the URL valid?",
            style("Error:").red().bold()
        );
        return Ok(());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("{}", stdout);

    let download_now = Confirm::with_theme(theme)
        .with_prompt("Do you want to download this video with a specific format code?")
        .default(true)
        .interact()?;

    if download_now {
        let format_code: String = Input::with_theme(theme)
            .with_prompt("Enter the format code (e.g. 137+140)")
            .interact_text()?;

        let dir: String = Input::with_theme(theme)
            .with_prompt("Output directory")
            .default(current_dir.to_string_lossy().to_string())
            .interact_text()?;
        *current_dir = PathBuf::from(&dir);

        let opts = DownloadOptions {
            url,
            format: format_code,
            output_dir: dir,
            ..Default::default()
        };
        let cmd = build_command(&opts);
        run_with_progress_cli(cmd)?;
    }
    Ok(())
}

/// Menampilkan progress bar di CLI menggunakan channel events.
fn run_with_progress_cli(mut cmd: Command) -> anyhow::Result<()> {
    let (tx, rx) = mpsc::channel();

    // Spawn thread untuk menjalankan unduhan
    let handle = std::thread::spawn(move || download_with_events(cmd, tx));

    // Progress bar di thread utama
    let pb = indicatif::ProgressBar::new(100);
    pb.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("{spinner:.magenta.bold} {elapsed_precise:.dim} [{bar:60}] {percent:>3}% | ETA: {eta:>5} | {msg}")?
            .progress_chars("█▓▒░ "),
    );
    pb.set_message("Initializing...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let mut file_count = 0;
    let mut current_filename = String::new();

    for event in rx {
        match event {
            DownloadEvent::Progress { percent, speed, filename } => {
                let short = std::path::Path::new(&filename)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&filename);
                pb.set_position(percent as u64);
                pb.set_message(format!("📄 {} @ {}", short, speed));
                current_filename = filename.clone();
            }
            DownloadEvent::FileStarted(name) => {
                pb.set_message(format!("Starting: {}", name));
                file_count += 1;
            }
            DownloadEvent::FileCompleted(name) => {
                pb.println(format!(" ✅  Completed: {}", name));
                pb.reset();
                pb.set_message("Waiting for next file...");
            }
            DownloadEvent::Finished { total_files } => {
                pb.finish_with_message("✨ Download completed successfully");
                println!("\n{}", style("───────────────────────────────────────────────────────").dim());
                println!(" {} Total files downloaded: {}", style("✅").green().bold(), total_files);
                println!(" {} Total time elapsed:    {:?}", style("⏱️").cyan(), pb.elapsed());
                println!("{}", style("───────────────────────────────────────────────────────").dim());
            }
            DownloadEvent::Error(e) => {
                pb.finish_with_message("❌ Download failed");
                eprintln!("{} {}", style("Error:").red().bold(), e);
            }
        }
    }

    match handle.join().unwrap() {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}