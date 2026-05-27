use egui::{Button, Ui};

use crate::ui::app::App;

impl App {
    pub fn application_settings(&mut self, ui: &mut Ui) {
        if ui.add(Button::new("About").frame(false)).clicked() {
            self.show_about = true;
        }

        if ui.add(Button::new("Report Issue").frame(false)).clicked() {
            let _ = std::process::Command::new("xdg-open")
                .arg("https://github.com/avitran0/deadlocked/issues")
                .status();
        }
    }
}
