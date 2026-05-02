#![cfg(feature = "gui")]

use eframe::egui;
use egui::{Color32, FontFamily, Frame, Margin, RichText, ScrollArea, Stroke, Visuals};
use std::sync::mpsc;
use std::time::SystemTime;
use yt_downloader::{build_command, download_with_events, DownloadEvent, DownloadOptions};

// ─────────────────────────────────────────────────────────────────────────────
// 🎨 THEME CONFIGURATION (Compatible with egui 0.27.2)
// ─────────────────────────────────────────────────────────────────────────────
fn apply_midnight_theme(ctx: &egui::Context) {
    let mut visuals = Visuals::dark();
    
    // Accent colors
    visuals.selection.bg_fill = Color32::from_rgb(102, 126, 234);
    visuals.selection.stroke = Stroke::new(1.0, Color32::from_rgb(74, 85, 226));
    
    // Widget states
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(30, 35, 50);
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(60, 70, 90));
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(200, 210, 230));
    
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(45, 52, 75);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.5, Color32::from_rgb(102, 126, 234));
    
    visuals.widgets.active.bg_fill = Color32::from_rgb(58, 68, 102);
    visuals.widgets.active.bg_stroke = Stroke::new(2.0, Color32::from_rgb(120, 140, 255));
    
    // Backgrounds
    visuals.window_fill = Color32::from_rgb(18, 22, 35);
    visuals.panel_fill = Color32::from_rgb(18, 22, 35);
    visuals.extreme_bg_color = Color32::from_rgb(12, 15, 25);
    visuals.code_bg_color = Color32::from_rgb(25, 30, 45);
    
    // Text - use override_text_color instead of text_color (egui 0.27.2)
    visuals.override_text_color = Some(Color32::from_rgb(220, 225, 240));
    visuals.hyperlink_color = Color32::from_rgb(102, 126, 234);
    
    // Spacing & rounding - use f32 literals
    visuals.window_rounding = egui::Rounding::same(12.0);
    // Skip window_shadow - not available in egui 0.27.2
    
    ctx.set_visuals(visuals);
}

// ─────────────────────────────────────────────────────────────────────────────
// 🧩 MAIN APPLICATION
// ─────────────────────────────────────────────────────────────────────────────
fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([720.0, 650.0])
            .with_min_inner_size([600.0, 500.0])
            .with_title("🎬 YouTube Downloader")
            .with_app_id("yt_downloader_modern"),
        ..Default::default()
    };
    
    eframe::run_native(
        "YouTube Downloader",
        options,
        Box::new(|cc| {
            apply_midnight_theme(&cc.egui_ctx);
            Box::new(YtDlpGui::default())
        }),
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// 📦 DATA STRUCTURE
// ─────────────────────────────────────────────────────────────────────────────
struct YtDlpGui {
    url: String,
    format: String,
    output_dir: String,
    is_playlist: bool,
    audio_only: bool,
    audio_format: String,
    audio_quality: String,
    
    download_active: bool,
    current_progress: f32,
    current_speed: String,
    current_filename: String,
    completed_files: usize,
    total_files: Option<usize>,
    
    log: Vec<LogEntry>,
    
    event_rx: Option<mpsc::Receiver<DownloadEvent>>,
    child_thread: Option<std::thread::JoinHandle<()>>,
}

#[derive(Clone)]
enum LogType { Info, Success, Warning, Error, Progress }

#[derive(Clone)]
struct LogEntry {
    message: String,
    log_type: LogType,
    timestamp: String,
}

impl Default for YtDlpGui {
    fn default() -> Self {
        Self {
            url: String::new(),
            format: "best".to_string(),
            output_dir: "./downloads".to_string(),
            is_playlist: false,
            audio_only: false,
            audio_format: "mp3".to_string(),
            audio_quality: "320k".to_string(),
            download_active: false,
            current_progress: 0.0,
            current_speed: String::new(),
            current_filename: String::new(),
            completed_files: 0,
            total_files: None,
            log: vec![LogEntry::info("✨ Welcome! Enter a YouTube URL to get started.")],
            event_rx: None,
            child_thread: None,
        }
    }
}

impl LogEntry {
    fn new(message: impl Into<String>, log_type: LogType) -> Self {
        // Simple timestamp without chrono dependency
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .ok()
            .map(|d| {
                let secs = d.as_secs() % 86400;
                format!("{:02}:{:02}:{:02}", secs / 3600, (secs % 3600) / 60, secs % 60)
            })
            .unwrap_or_default();
            
        Self {
            message: message.into(),
            log_type,
            timestamp,
        }
    }
    
    fn info(msg: impl Into<String>) -> Self { Self::new(msg, LogType::Info) }
    fn success(msg: impl Into<String>) -> Self { Self::new(msg, LogType::Success) }
    fn warning(msg: impl Into<String>) -> Self { Self::new(msg, LogType::Warning) }
    fn error(msg: impl Into<String>) -> Self { Self::new(msg, LogType::Error) }
    fn progress(msg: impl Into<String>) -> Self { Self::new(msg, LogType::Progress) }
    
    fn color(&self) -> Color32 {
        match self.log_type {
            LogType::Info => Color32::from_rgb(200, 210, 230),
            LogType::Success => Color32::from_rgb(34, 197, 94),
            LogType::Warning => Color32::from_rgb(251, 191, 36),
            LogType::Error => Color32::from_rgb(239, 68, 68),
            LogType::Progress => Color32::from_rgb(102, 126, 234),
        }
    }
    
    fn icon(&self) -> &'static str {
        match self.log_type {
            LogType::Info => "ℹ️",
            LogType::Success => "✅",
            LogType::Warning => "⚠️",
            LogType::Error => "❌",
            LogType::Progress => "⚡",
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 🔄 EGUI APP IMPLEMENTATION
// ─────────────────────────────────────────────────────────────────────────────
impl eframe::App for YtDlpGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_events();
        self.draw_ui(ctx);
        
        if self.download_active {
            ctx.request_repaint();
        }
    }
}

impl YtDlpGui {
    fn process_events(&mut self) {
        if let Some(rx) = self.event_rx.take() {
            let mut should_drop = false;

            while let Ok(event) = rx.try_recv() {
                match event {
                    DownloadEvent::Progress { percent, speed, filename } => {
                        self.current_progress = percent;
                        self.current_speed = speed.clone(); // Clone to avoid move issue
                        self.current_filename = filename.clone();
                        if percent % 25.0 < 0.1 && percent > 0.0 {
                            self.log.push(LogEntry::progress(
                                format!("{}: {}% @ {}", filename, percent as i32, speed)
                            ));
                        }
                    }
                    DownloadEvent::FileStarted(name) => {
                        self.log.push(LogEntry::info(format!("▶ Starting: {}", name)));
                    }
                    DownloadEvent::FileCompleted(name) => {
                        self.log.push(LogEntry::success(format!("✅ Completed: {}", name)));
                        self.completed_files += 1;
                        self.current_progress = 0.0;
                        self.current_filename.clear();
                    }
                    DownloadEvent::Finished { total_files } => {
                        self.total_files = Some(total_files);
                        self.log.push(LogEntry::success(
                            format!("🎉 All done! {} file(s) downloaded successfully.", total_files)
                        ));
                        self.download_active = false;
                        should_drop = true;
                        self.cleanup_thread();
                    }
                    DownloadEvent::Error(e) => {
                        self.log.push(LogEntry::error(format!("💥 Error: {}", e)));
                        self.download_active = false;
                        should_drop = true;
                        self.cleanup_thread();
                    }
                }
            }

            if !should_drop {
                self.event_rx = Some(rx);
            }
        }
    }
    
    fn cleanup_thread(&mut self) {
        if let Some(handle) = self.child_thread.take() {
            let _ = handle.join();
        }
    }

    fn draw_ui(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(Frame::central_panel(&ctx.style()).inner_margin(Margin::same(20.0)))
            .show(ctx, |ui| {
                self.draw_header(ui);
                ui.add_space(8.0);
                
                // Use horizontal layout instead of SidePanel for simplicity
                ui.horizontal(|ui| {
                    // Left column - Controls
                    ui.vertical(|ui| {
                        ui.set_min_width(350.0);
                        self.draw_input_section(ui);
                        ui.add_space(12.0);
                        self.draw_options_section(ui);
                        ui.add_space(12.0);
                        self.draw_action_section(ui);
                    });
                    
                    ui.add_space(20.0);
                    
                    // Right column - Status & Log
                    ui.vertical(|ui| {
                        ui.set_min_width(280.0);
                        self.draw_progress_section(ui);
                        ui.add_space(16.0);
                        self.draw_log_section(ui);
                    });
                });
            });
    }

    fn draw_header(&self, ui: &mut egui::Ui) {
        Frame::none().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("🎬").font(egui::FontId::proportional(24.0)));
                ui.label(RichText::new("YouTube Downloader")
                    .font(egui::FontId::proportional(20.0))
                    .strong()
                    .color(Color32::from_rgb(102, 126, 234)));
            });
            ui.label(RichText::new("Download videos & audio with ease")
                .font(egui::FontId::proportional(12.0))
                .color(Color32::from_gray(140)));
        });
    }

    fn draw_input_section(&mut self, ui: &mut egui::Ui) {
        // Use a local closure pattern to avoid borrow conflicts
        Frame::group(&ui.style())
            .fill(Color32::from_rgb(30, 35, 50))
            .stroke(Stroke::new(1.0, Color32::from_rgb(60, 70, 90)))
            .rounding(egui::Rounding::same(10.0))
            .inner_margin(Margin::same(12.0))
            .show(ui, |ui| {
                ui.label(RichText::new("🔗 Video URL")
                    .font(egui::FontId::proportional(13.0))
                    .strong()
                    .color(Color32::from_rgb(180, 190, 210)));
                ui.add_space(8.0);
                
                ui.add_sized(
                    [ui.available_width(), 32.0],
                    egui::TextEdit::singleline(&mut self.url)
                        .hint_text("https://youtube.com/watch?v=...")
                        .desired_width(f32::INFINITY),
                );
                
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Format:").size(11.0).color(Color32::from_gray(150)));
                    for preset in ["best", "mp4", "webm"] {
                        if ui.selectable_label(self.format == preset, preset).clicked() {
                            self.format = preset.to_string();
                        }
                    }
                });
            });
    }

    fn draw_options_section(&mut self, ui: &mut egui::Ui) {
        Frame::group(&ui.style())
            .fill(Color32::from_rgb(30, 35, 50))
            .stroke(Stroke::new(1.0, Color32::from_rgb(60, 70, 90)))
            .rounding(egui::Rounding::same(10.0))
            .inner_margin(Margin::same(12.0))
            .show(ui, |ui| {
                ui.label(RichText::new("⚙️ Options")
                    .font(egui::FontId::proportional(13.0))
                    .strong()
                    .color(Color32::from_rgb(180, 190, 210)));
                ui.add_space(8.0);
                
                // Output directory
                ui.horizontal(|ui| {
                    ui.label("📁");
                    ui.add(egui::TextEdit::singleline(&mut self.output_dir).desired_width(180.0));
                    if ui.small_button("Browse").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            self.output_dir = path.display().to_string();
                        }
                    }
                });
                ui.add_space(4.0);
                
                ui.checkbox(&mut self.is_playlist, "📋 Playlist download");
                ui.checkbox(&mut self.audio_only, "🎵 Audio only mode");
                
                if self.audio_only {
                    ui.indent("audio_opts", |ui| {
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.label("Format:");
                            // ComboBox without add_sized to avoid Widget trait issue
                            egui::ComboBox::from_id_source("audio_format")
                                .selected_text(&self.audio_format)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.audio_format, "mp3".to_string(), "MP3");
                                    ui.selectable_value(&mut self.audio_format, "m4a".to_string(), "M4A");
                                    ui.selectable_value(&mut self.audio_format, "wav".to_string(), "WAV");
                                    ui.selectable_value(&mut self.audio_format, "flac".to_string(), "FLAC");
                                });
                            
                            ui.label("Quality:");
                            egui::ComboBox::from_id_source("audio_quality")
                                .selected_text(&self.audio_quality)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.audio_quality, "320k".to_string(), "320k");
                                    ui.selectable_value(&mut self.audio_quality, "256k".to_string(), "256k");
                                    ui.selectable_value(&mut self.audio_quality, "192k".to_string(), "192k");
                                    ui.selectable_value(&mut self.audio_quality, "128k".to_string(), "128k");
                                });
                        });
                    });
                }
            });
    }

    fn draw_action_section(&mut self, ui: &mut egui::Ui) {
        Frame::none().show(ui, |ui| {
            let button_enabled = !self.download_active && !self.url.is_empty();
            
            if ui.add_enabled(
                button_enabled,
                egui::Button::new(RichText::new("⬇ Start Download").font(egui::FontId::proportional(14.0)))
                    .fill(Color32::from_rgb(102, 126, 234))
                    .stroke(Stroke::new(0.0, Color32::TRANSPARENT))
                    .min_size(egui::vec2(ui.available_width(), 44.0))
            ).clicked() {
                self.start_download();
            }
            
            if self.download_active {
                ui.add_space(8.0);
                if ui.button(RichText::new("⏹ Cancel").color(Color32::from_rgb(239, 68, 68)))
                    .clicked() {
                    self.cancel_download();
                }
            }
            
            if !button_enabled && !self.download_active {
                ui.label(RichText::new("• Enter a valid URL to begin")
                    .size(11.0)
                    .color(Color32::from_gray(120)));
            }
        });
    }

    fn draw_progress_section(&mut self, ui: &mut egui::Ui) {
        Frame::group(&ui.style())
            .fill(Color32::from_rgb(30, 35, 50))
            .stroke(Stroke::new(1.0, Color32::from_rgb(60, 70, 90)))
            .rounding(egui::Rounding::same(10.0))
            .inner_margin(Margin::same(12.0))
            .show(ui, |ui| {
                ui.label(RichText::new("📊 Progress")
                    .font(egui::FontId::proportional(13.0))
                    .strong()
                    .color(Color32::from_rgb(180, 190, 210)));
                ui.add_space(8.0);
                
                if !self.download_active && self.completed_files == 0 {
                    ui.centered_and_justified(|ui| {
                        ui.label(RichText::new("⏳ Waiting to start...")
                            .color(Color32::from_gray(140)));
                    });
                    return;
                }
                
                if !self.current_filename.is_empty() {
                    ui.label(RichText::new("📄").size(10.0));
                    // Use text wrap via Ui, not RichText
                    ui.label(RichText::new(&self.current_filename).size(12.0));
                    ui.add_space(4.0);
                }
                
                let progress = (self.current_progress / 100.0).clamp(0.0, 1.0);
                let mut progress_bar = egui::ProgressBar::new(progress)
                    .show_percentage()
                    .animate(true)
                    .fill(Color32::from_rgb(102, 126, 234));
                    
                if let Some(total) = self.total_files {
                    progress_bar = progress_bar.text(format!(
                        "{}% • File {}/{}", 
                        self.current_progress as i32, 
                        self.completed_files + 1, 
                        total
                    ));
                }
                
                ui.add(progress_bar);
                ui.add_space(8.0);
                
                ui.horizontal(|ui| {
                    if !self.current_speed.is_empty() {
                        ui.label(RichText::new(format!("⚡ {}", self.current_speed))
                            .size(11.0)
                            .color(Color32::from_rgb(102, 126, 234)));
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(format!("✅ {} done", self.completed_files))
                            .size(11.0)
                            .color(Color32::from_rgb(34, 197, 94)));
                    });
                });
            });
    }

    fn draw_log_section(&mut self, ui: &mut egui::Ui) {
        Frame::group(&ui.style())
            .fill(Color32::from_rgb(30, 35, 50))
            .stroke(Stroke::new(1.0, Color32::from_rgb(60, 70, 90)))
            .rounding(egui::Rounding::same(10.0))
            .inner_margin(Margin::same(12.0))
            .show(ui, |ui| {
                ui.label(RichText::new("📝 Activity Log")
                    .font(egui::FontId::proportional(13.0))
                    .strong()
                    .color(Color32::from_rgb(180, 190, 210)));
                ui.add_space(8.0);
                
                ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .auto_shrink([false, false])
                    .max_height(220.0)
                    .show(ui, |ui| {
                        for entry in &self.log {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(entry.icon()).size(10.0));
                                ui.label(RichText::new(&entry.timestamp)
                                    .size(9.0)
                                    .color(Color32::from_gray(100)));
                                ui.label(RichText::new(&entry.message)
                                    .size(11.0)
                                    .color(entry.color()));
                            });
                            ui.add_space(2.0);
                        }
                    });
                
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("Clear").clicked() {
                            self.log.clear();
                            self.log.push(LogEntry::info("🧹 Log cleared."));
                        }
                    });
                });
            });
    }

    fn start_download(&mut self) {
        let opts = DownloadOptions {
            url: self.url.clone(),
            format: self.format.clone(),
            output_dir: self.output_dir.clone(),
            is_playlist: self.is_playlist,
            audio_only: self.audio_only,
            audio_format: if self.audio_only && !self.audio_format.is_empty() {
                Some(self.audio_format.clone())
            } else { None },
            audio_quality: if self.audio_only && !self.audio_quality.is_empty() {
                Some(self.audio_quality.clone())
            } else { None },
        };

        let (tx, rx) = mpsc::channel();
        self.event_rx = Some(rx);
        self.download_active = true;
        self.reset_progress_state();
        self.log.push(LogEntry::progress("🚀 Download initiated..."));

        let handle = std::thread::spawn(move || {
            let cmd = build_command(&opts);
            let tx_error = tx.clone();
            if let Err(e) = download_with_events(cmd, tx) {
                let _ = tx_error.send(DownloadEvent::Error(e.to_string()));
            }
        });
        self.child_thread = Some(handle);
    }
    
    fn cancel_download(&mut self) {
        self.log.push(LogEntry::warning("⚠️ Cancellation requested..."));
        self.download_active = false;
        self.cleanup_thread();
    }
    
    fn reset_progress_state(&mut self) {
        self.current_progress = 0.0;
        self.current_speed.clear();
        self.current_filename.clear();
        self.completed_files = 0;
        self.total_files = None;
    }
}