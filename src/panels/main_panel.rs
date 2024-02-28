use eframe::egui::{self, Ui};

use crate::{code_editor::ExtendedCodeEditorSpawner, Project};

pub fn init(ui: &mut Ui, project: &mut Project) {
    ui.with_layout(egui::Layout::top_down_justified(egui::Align::Max), |ui| {
        if let Some(current_file) = project.current_file.clone() {
            let contents = project.files.get_mut(&current_file);
            if let Some(contents) = contents {
                let text_edit = ui.ext_code_ui(contents);
                if text_edit.changed() {
                    project.files_edited.insert(current_file, true);
                }
            }
        }
    });
}
