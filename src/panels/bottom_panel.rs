use crate::Project;
use eframe::egui::{self, Ui};

pub fn init(ui: &mut Ui, project: &mut Project) {
    ui.vertical(|ui| {
        ui.add_space(5.0);
        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                let binding = project.project_path.clone().unwrap_or_default();
                let header_text = binding.file_name().unwrap_or_default().to_string_lossy();
                egui::ComboBox::from_label(header_text)
                    .selected_text("Editor")
                    .show_ui(ui, |ui| {
                        if ui.selectable_label(false, "Open New Project...").clicked() {
                            if let Some(project_path) = &rfd::FileDialog::new().pick_folder() {
                                if Some(project_path) != project.project_path.as_ref() {
                                    *project = Project {
                                        project_path: Some(project_path.to_path_buf()),
                                        ..project.clone()
                                    };
                                    project.current_file = None;
                                }
                            }
                        }
                        if ui.selectable_label(false, "Clear Cache").clicked() {
                            *project = Project::default();
                        }
                    });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .selectable_label(project.terminal_opened, "Terminal")
                        .clicked()
                    {
                        project.terminal_opened = !project.terminal_opened;
                    }
                });
            });
        });

        if project.terminal_opened {
            ui.separator();
            ui.scope(|ui| {
                ui.set_height(200.0);
                ui.label("Not Implemented Yet!");
            });
        }
    });
}
