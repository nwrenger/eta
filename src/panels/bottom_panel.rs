use crate::{terminal::TermHandler, Project};
use eframe::egui::{self, Ui};
use portable_pty::CommandBuilder;

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
                            project.open_project();
                        }
                        if ui.selectable_label(false, "Clear Cache").clicked() {
                            *project = Project::default();
                        }
                    });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .selectable_label(project.get_current_terminal().is_some(), "Terminal")
                        .clicked()
                    {
                        if project.get_current_terminal().is_some() {
                            project
                                .remove_terminal(&project.project_path.clone().unwrap_or_default());
                        } else {
                            let mut cmd = CommandBuilder::new_default_prog();
                            let path = project.project_path.clone().unwrap_or_default();
                            cmd.cwd(&path);
                            let term = TermHandler::new(cmd);
                            project.terminals.insert(path, term);
                        }
                    }
                });
            });
        });
    });
}
