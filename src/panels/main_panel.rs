use std::path::PathBuf;

use eframe::egui::{self, ScrollArea, Ui};

use crate::{code_editor::code_editor, Project};

pub fn init(ui: &mut Ui, project: &mut Project) {
    ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
        egui::ScrollArea::vertical()
            .auto_shrink(true)
            .show(ui, |ui| {
                let binding = PathBuf::new();
                let raw_text = project
                    .current_file
                    .as_ref()
                    .unwrap_or(&binding)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy();
                let text = if project.is_current_file_edited() {
                    format!("{} *", raw_text)
                } else {
                    raw_text.to_string()
                };

                let header_color = if project.is_current_file_edited() {
                    egui::Color32::WHITE
                } else {
                    egui::Color32::GRAY
                };

                let header_label = egui::RichText::new(text).color(header_color);

                ui.heading(header_label);
                let mut text_changed = false;
                if let Some(current_file) = project.current_file.clone() {
                    let contents = project.files.get_mut(&current_file);
                    if let Some(contents) = contents {
                        ScrollArea::horizontal().auto_shrink(false).show(ui, |ui| {
                            let text_edit = ui.add(code_editor("main_panel_ce".into(), contents));

                            text_changed = text_edit.changed();
                        });
                        if text_changed {
                            project.files_edited.insert(current_file, true);
                        }
                    }
                }
            });
    });
}
