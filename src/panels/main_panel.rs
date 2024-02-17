use std::{fs, path::PathBuf};

use eframe::egui::{self, ScrollArea, Ui};

use crate::{code_editor::code_editor, Project};

pub fn init(ui: &mut Ui, project: &mut Project) {
    ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
        egui::ScrollArea::vertical()
            .auto_shrink(true)
            .show(ui, |ui| {
                let binding = PathBuf::new();
                let raw_text = project
                    .file_path
                    .as_ref()
                    .unwrap_or(&binding)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy();
                let text =
                    if project.file_edited.is_some() && project.file_edited == project.file_path {
                        format!("{} *", raw_text)
                    } else {
                        raw_text.to_string()
                    };

                let header_color =
                    if project.file_edited.is_some() && project.file_edited == project.file_path {
                        egui::Color32::LIGHT_BLUE
                    } else {
                        egui::Color32::WHITE
                    };

                let header_label = egui::RichText::new(text).color(header_color);

                ui.heading(header_label);
                if let Some(contents) = &mut project.file_content {
                    ScrollArea::horizontal().auto_shrink(false).show(ui, |ui| {
                        let text_edit = ui.add(code_editor("main_panel_ce".into(), contents));

                        if project.file_edited.is_some()
                            && project.file_edited == project.file_path
                            && ui.input_mut(|i| {
                                i.consume_key(egui::Modifiers::COMMAND, egui::Key::S)
                            })
                        {
                            if let Some(file_path) = &project.file_path {
                                fs::write(file_path, contents).unwrap_or_default();
                                project.file_edited = None;
                            }
                        }

                        if text_edit.changed() {
                            project.file_edited = project.file_path.clone();
                        }
                    });
                }
            });
    });
}
