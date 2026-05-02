// gui.rs (hanya dikompilasi jika fitur 'gui' aktif)
pub struct YtDlpGui {
    url: String,
    format: String,
    output_dir: String,
    is_playlist: bool,
    audio_only: bool,
    // Status download
    downloads: Vec<DownloadStatus>,
    log: Vec<String>,
}

struct DownloadStatus {
    filename: String,
    progress: f32,  // 0..100
    speed: String,
    completed: bool,
}

impl eframe::App for YtDlpGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🎬 YouTube Downloader");
            ui.horizontal(|ui| {
                ui.label("URL:");
                ui.text_edit_singleline(&mut self.url);
            });
            // Tombol start download
            if ui.button("⬇ Download").clicked() {
                self.start_downloads();
            }
            // Progress bars
            for ds in &self.downloads {
                ui.label(&ds.filename);
                ui.add(egui::ProgressBar::new(ds.progress / 100.0).show_percentage());
            }
            // Log area
            egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                for line in &self.log {
                    ui.label(line);
                }
            });
        });
        ctx.request_repaint(); // Untuk update real-time
    }
}