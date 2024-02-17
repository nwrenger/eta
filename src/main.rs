#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod code_editor;
pub mod panels;

use std::{fs, path::PathBuf};

use eframe::{
    egui::{self},
    get_value, set_value, Storage,
};
use serde::{Deserialize, Serialize};

fn main() -> Result<(), eframe::Error> {
    env_logger::init();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])
            .with_min_inner_size([600.0, 400.0]),
        ..Default::default()
    };
    eframe::run_native("co", options, Box::new(|cc| Box::new(App::new(cc.storage))))
}

struct App {
    project: Project,
}

impl App {
    fn new(storage: Option<&dyn Storage>) -> Self {
        Self {
            project: storage
                .and_then(|s| get_value(s, "project"))
                .unwrap_or_default(),
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct Project {
    pub project_path: Option<PathBuf>,
    pub file_path: Option<PathBuf>,
    pub file_content: Option<String>,
    pub file_edited: Option<PathBuf>,
}

impl eframe::App for App {
    fn save(&mut self, storage: &mut dyn Storage) {
        set_value(storage, "project", &self.project);
    }
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // commands
        if ctx.input_mut(|i| {
            self.project.file_edited.is_some()
                && self.project.file_edited == self.project.file_path
                && i.consume_key(egui::Modifiers::COMMAND, egui::Key::S)
        }) {
            if let Some(file_path) = &self.project.file_path {
                if let Some(content) = &self.project.file_content {
                    fs::write(file_path, content).unwrap_or_default();
                    self.project.file_edited = None;
                }
            }
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::O)) {
            if let Some(project_path) = &rfd::FileDialog::new().pick_folder() {
                if Some(project_path) != self.project.project_path.as_ref() {
                    self.project = Project {
                        project_path: Some(project_path.to_path_buf()),
                        ..Default::default()
                    };
                }
            }
        }

        // panels
        egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(false)
            .default_height(40.0)
            .show(ctx, |ui| {
                panels::bottom_panel::init(ui, &mut self.project);
            });
        egui::SidePanel::left("left_side_panel")
            .default_width(200.0)
            .min_width(150.0)
            .max_width(250.0)
            .show(ctx, |ui| {
                panels::left_side_panel::init(ui, &mut self.project);
            });
        egui::CentralPanel::default().show(ctx, |ui| {
            panels::main_panel::init(ui, &mut self.project);
        });
    }
}
