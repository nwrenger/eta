use std::{fs, path::PathBuf};

use eframe::egui::{
    self, text_edit::TextEditOutput, Modifiers, RichText, ScrollArea, TextBuffer, TextEdit, Ui,
};

use crate::Project;

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
                    ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                        let mut text_edit = text_edit_with_line_numbers(ui, contents).unwrap();

                        if ui.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::Y)) {
                            if let Some(text_cursor_range) = text_edit.cursor_range {
                                let selected_chars = text_cursor_range.as_sorted_char_range();
                                let selected_text = contents.char_range(selected_chars.clone());
                                let upper_case = selected_text.to_uppercase();
                                let new_text = if selected_text == upper_case {
                                    selected_text.to_lowercase()
                                } else {
                                    upper_case
                                };
                                contents.delete_char_range(selected_chars.clone());
                                contents.insert_text(&new_text, selected_chars.start);
                            }
                        }
                        if ui.input_mut(|i| {
                            i.consume_key(
                                Modifiers {
                                    alt: false,
                                    ctrl: false,
                                    shift: true,
                                    mac_cmd: false,
                                    command: true,
                                },
                                egui::Key::Z,
                            )
                        }) {
                            println!("redo");
                        }

                        // todo: temp, fix when implementing state system
                        text_edit.state.clear_undoer();

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

                        if text_edit.response.changed() {
                            project.file_edited = project.file_path.clone();
                        }
                    });
                }
            });
    });
}

fn text_edit_with_line_numbers(ui: &mut Ui, text: &mut String) -> Option<TextEditOutput> {
    let mut text_edit_output: Option<TextEditOutput> = None;

    egui::Grid::new("text_edit_with_line_numbers_grid")
        .num_columns(2)
        .spacing([10.0, 0.0])
        .show(ui, |ui| {
            ui.vertical(|ui| {
                let line_count = text.lines().count();
                for line_number in 1..=line_count + 1 {
                    ui.label(RichText::new(format!("{:>4}", line_number)).size(12.5));
                    ui.add_space(-ui.spacing().item_spacing.y);
                }
            });

            // Column for the text edit
            ui.vertical(|ui| {
                let response = TextEdit::multiline(text)
                    .code_editor()
                    .desired_rows(0)
                    .desired_width(f32::INFINITY)
                    .show(ui);

                text_edit_output = Some(response);
            });
        });

    text_edit_output
}
