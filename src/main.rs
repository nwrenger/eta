#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod code_editor;
pub mod panels;
pub mod terminal;

use std::{collections::HashMap, fs, path::PathBuf, sync::Arc};

use code_editor::FileData;
use eframe::{
    egui::{self},
    get_value, icon_data, set_value, Storage,
};
use serde::{Deserialize, Serialize};
use terminal::TermHandler;

fn main() -> Result<(), eframe::Error> {
    env_logger::init();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])
            .with_min_inner_size([600.0, 400.0])
            .with_icon(Arc::new(
                icon_data::from_png_bytes(include_bytes!("../assets/icon.png")).unwrap(),
            )),
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

#[derive(Serialize, Deserialize, Default)]
pub struct Project {
    #[serde(skip)]
    pub terminals: HashMap<PathBuf, TermHandler>,
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

    pub fn get_file(&self, path: &PathBuf) -> Option<&FileData> {
        self.files.get(path)
    }

    pub fn get_current_file(&self) -> Option<&FileData> {
        self.get_file(self.current_file.as_ref().unwrap_or(&PathBuf::default()))
    }

    pub fn get_terminal(&self, path: &PathBuf) -> Option<&TermHandler> {
        self.terminals.get(path)
    }

    pub fn get_current_terminal(&self) -> Option<&TermHandler> {
        self.terminals
            .get(self.project_path.as_ref().unwrap_or(&PathBuf::default()))
    }

    pub fn get_mut_terminal(&mut self, path: &PathBuf) -> Option<&mut TermHandler> {
        self.terminals.get_mut(path)
    }

    pub fn get_mut_current_terminal(&mut self) -> Option<&mut TermHandler> {
        self.terminals
            .get_mut(self.project_path.as_ref().unwrap_or(&PathBuf::default()))
    }

    pub fn remove_file(&mut self, path: &PathBuf) {
        self.files.remove(path);
        self.files_edited.remove(path);
        if let Some(current_file) = &self.current_file {
            if current_file == path {
                self.current_file = None;
            }
        }
    }

    fn remove_terminal(&mut self, path: &PathBuf) {
        self.terminals.remove(path);
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
                    self.project.project_path = Some(project_path.to_path_buf());
                    self.project.current_file = None;
                }
            }
        }

        // panels
        egui::TopBottomPanel::bottom("bottom_panel")
            .min_height(30.0)
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
