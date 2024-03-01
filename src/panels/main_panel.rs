use ecolor::{Color32, HexColor};
use eframe::egui::{self, Ui};

use crate::{
    code_editor::ExtendedCodeEditorSpawner,
    terminal::{prelude::TerminalSpawner, render::CursorType},
    Project,
};

pub fn init(ui: &mut Ui, project: &mut Project) {
    let terminal_heigth: f32 = ui.available_height() * 0.25;
    ui.vertical(|ui| {
        // text input window
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
            if let Some(current_file) = project.current_file.clone() {
                let contents = project.files.get_mut(&current_file);
                if let Some(contents) = contents {
                    // update height on shown terminal
                    if project.terminal.is_some() {
                        ui.set_height(ui.available_height() - (terminal_heigth + 2.5));
                    }
                    let text_edit = ui.ext_code_ui(contents);
                    if text_edit.changed() {
                        project.files_edited.insert(current_file, true);
                    }
                }
            }
        });

        // terminal
        if let Some(terminal) = &mut project.terminal {
            egui::panel::TopBottomPanel::bottom("terminal_panel")
                .min_height(terminal_heigth)
                .show(ui.ctx(), |ui| {
                    terminal.style.cursor_trail = true;
                    terminal.style.cursor_trail_color = None;
                    terminal.style.default_focus_cursor =
                        CursorType::Beam(HexColor::Hex8(Color32::WHITE));
                    terminal.style.default_unfocus_cursor = CursorType::None;

                    ui.terminal_sized(terminal, egui::vec2(ui.available_width(), terminal_heigth));
                });
        }
    });
}
