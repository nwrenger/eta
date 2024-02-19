#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod code_editor;
pub mod panels;

use std::{collections::HashMap, fs, path::PathBuf};

use code_editor::FileData;
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
    eframe::run_native(
        "eta",
        options,
        Box::new(|cc| Box::new(App::new(cc.storage))),
    )
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

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Project {
    pub project_path: Option<PathBuf>,
    pub current_file: Option<PathBuf>,
    pub files: HashMap<PathBuf, FileData>,
    pub files_edited: HashMap<PathBuf, bool>,
}

impl Project {
    pub fn is_file_edited(&self, path: &PathBuf) -> bool {
        self.files_edited.get(path).is_some()
    }

    pub fn is_current_file_edited(&self) -> bool {
        self.is_file_edited(self.current_file.as_ref().unwrap_or(&PathBuf::default()))
    }

    pub fn get_file(&self, path: &PathBuf) -> Option<FileData> {
        self.files.get(path).cloned()
    }

    pub fn get_current_file(&self) -> Option<FileData> {
        self.get_file(self.current_file.as_ref().unwrap_or(&PathBuf::default()))
    }
}

impl eframe::App for App {
    fn save(&mut self, storage: &mut dyn Storage) {
        set_value(storage, "project", &self.project);
    }
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // commands
        if ctx.input_mut(|i| {
            self.project.is_current_file_edited()
                && i.consume_key(egui::Modifiers::COMMAND, egui::Key::S)
        }) {
            if let Some(file_path) = &self.project.current_file {
                if let Some(content) = &self.project.files.get(file_path) {
                    fs::write(file_path, content.text.clone()).unwrap_or_default();
                    self.project.files_edited.remove(file_path);
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
