use eframe::egui;

/// Generic trait that injects information into a ui
pub trait FileInfo {
    fn info(&mut self, ui: &mut egui::Ui);
}

/// Simple FileInfo to inject a simpel message
pub struct NoFile {
    message: String,
}

impl NoFile {
    pub fn new(msg: &str) -> Self {
        Self {
            message: msg.to_string(),
        }
    }
}

impl FileInfo for NoFile {
    fn info(&mut self, ui: &mut egui::Ui) {
        ui.label(&self.message);
    }
}
