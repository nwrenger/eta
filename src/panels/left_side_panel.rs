use std::{
    ffi::OsStr,
    fs, io,
    path::{Path, PathBuf},
};

use eframe::{
    egui::{self, Key, Modifiers, Response, RichText, Sense, TextEdit, Ui},
    epaint::Color32,
};

use crate::{code_editor::FileData, Project};

#[derive(PartialEq)]
enum EntryType {
    Root,
    Directory,
    File,
}

pub fn init(ui: &mut Ui, project: &mut Project) {
    ui.add_space(5.0);
    egui::ScrollArea::vertical()
        .auto_shrink(true)
        .max_height(ui.available_height() - 3.0)
        .show(ui, |ui| {
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
                if let Some(path) = project.project_path.clone() {
                    file_side_bar(ui, &path, project).unwrap_or_default();
                    let response =
                        ui.allocate_response(ui.available_size(), egui::Sense::click_and_drag());

                    response.context_menu(|ui| {
                        ctx_menu(
                            ui,
                            project,
                            project.project_path.clone().unwrap_or_default(),
                            &project
                                .project_path
                                .clone()
                                .unwrap_or_default()
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy(),
                            EntryType::Root,
                            &response,
                        )
                    });
                } else {
                    ui.label("No Project Opened...");
                }
            });
        });
}

fn file_side_bar(ui: &mut Ui, path: &PathBuf, project: &mut Project) -> io::Result<()> {
    let mut entries = fs::read_dir(path)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;

    entries.sort_by(|a, b| {
        let a_metadata = a.metadata().expect("Failed to get metadata for a");
        let b_metadata = b.metadata().expect("Failed to get metadata for b");

        let a_is_dir = a_metadata.is_dir();
        let b_is_dir = b_metadata.is_dir();

        if a_is_dir && !b_is_dir {
            std::cmp::Ordering::Less
        } else if !a_is_dir && b_is_dir {
            std::cmp::Ordering::Greater
        } else {
            a.file_name().unwrap().cmp(b.file_name().unwrap())
        }
    });

    for entry in entries {
        let metadata = fs::metadata(&entry).expect("Unable to read metadata");
        let file_name = &*entry.file_name().unwrap_or_default().to_string_lossy();

        if metadata.is_dir() {
            let id = ui.make_persistent_id(&*entry);
            let collapse = egui::collapsing_header::CollapsingState::load_with_default_open(
                ui.ctx(),
                id,
                false,
            );
            let header = collapse.show_header(ui, |ui| {
                ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
                    let select = ui
                        .selectable_label(false, file_name)
                        .interact(Sense::click_and_drag());
                    select.context_menu(|ui| {
                        ctx_menu(
                            ui,
                            project,
                            entry.to_path_buf(),
                            file_name,
                            EntryType::Directory,
                            &select,
                        )
                    });
                    if select.dragged() {
                        println!("dragging");
                    }
                });
            });
            header.body(|ui| {
                ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
                    file_side_bar(ui, &entry, project)
                });
            });
        }

        if metadata.is_file() {
            let text = if project.is_file_edited(&entry) {
                RichText::new(file_name.to_string() + " *").color(Color32::WHITE)
            } else {
                RichText::new(file_name)
            };
            let select = ui
                .selectable_value(&mut project.current_file, Some(entry.to_path_buf()), text)
                .interact(Sense::click_and_drag());
            if !project.is_current_file_edited() && select.clicked() {
                let def_edit = FileData::default();
                let editor = &project
                    .files
                    .get(project.current_file.as_ref().unwrap_or(&PathBuf::default()))
                    .unwrap_or(&def_edit)
                    .editor;
                project.files.insert(
                    project
                        .current_file
                        .as_ref()
                        .unwrap_or(&PathBuf::default())
                        .to_path_buf(),
                    FileData {
                        text: fs::read_to_string(
                            project.current_file.as_ref().unwrap_or(&PathBuf::new()),
                        )
                        .unwrap_or_default(),
                        editor: editor.clone(),
                    },
                );
            }
            if select.dragged() {
                println!("dragging");
            }
            select.context_menu(|ui| {
                ctx_menu(
                    ui,
                    project,
                    entry.to_path_buf(),
                    file_name,
                    EntryType::File,
                    &select,
                )
            });
        }
    }

    Ok(())
}

fn ctx_menu(
    ui: &mut Ui,
    project: &mut Project,
    path: PathBuf,
    file_name: &str,
    entry_type: EntryType,
    response: &Response,
) {
    // state
    let mut editable: (String, String, String) = ui
        .memory_mut(|w| w.data.get_persisted(ui.id()))
        .unwrap_or((String::new(), String::new(), file_name.to_string()));
    // update like current name for renaming data on changes
    if response.clicked_by(egui::PointerButton::Secondary) {
        editable = (String::new(), String::new(), file_name.to_string());
    }

    ui.label(file_name);
    if entry_type == EntryType::Root || entry_type == EntryType::Directory {
        ui.menu_button("Add Directory", |ui| {
            let input_field = ui.add(
                TextEdit::singleline(&mut editable.0)
                    .hint_text("Directory Name")
                    .cursor_at_end(true),
            );
            if ui.button("Add").clicked()
                || (input_field.lost_focus()
                    && ui.input_mut(|r| r.consume_key(Modifiers::NONE, Key::Enter)))
            {
                fs::create_dir(path.join(&editable.0)).unwrap();
                editable.0 = String::new();
                ui.close_menu();
            }
        });
        ui.menu_button("Add File", |ui| {
            let input_field = ui.add(TextEdit::singleline(&mut editable.1).hint_text("File Name"));
            if ui.button("Add").clicked()
                || (input_field.lost_focus()
                    && ui.input_mut(|r| r.consume_key(Modifiers::NONE, Key::Enter)))
            {
                fs::write(path.join(&editable.1), "").unwrap();
                editable.1 = String::new();
                ui.close_menu();
            }
        });
    }
    if entry_type == EntryType::Directory || entry_type == EntryType::File {
        ui.menu_button("Rename", |ui| {
            let input_field = ui.add(TextEdit::singleline(&mut editable.2).hint_text("Name"));
            if ui.button("Apply").clicked()
                || (input_field.lost_focus()
                    && ui.input_mut(|r| r.consume_key(Modifiers::NONE, Key::Enter)))
            {
                let new_path = rename(&path, &editable.2).unwrap_or(path.clone());
                editable.2 = new_path
                    .file_name()
                    .unwrap_or(OsStr::new(file_name))
                    .to_string_lossy()
                    .to_string();
                if project.current_file.clone().unwrap_or_default() == path {
                    project.current_file = Some(new_path);
                    project.files.insert(
                        project
                            .current_file
                            .as_ref()
                            .unwrap_or(&PathBuf::default())
                            .to_path_buf(),
                        FileData {
                            text: fs::read_to_string(
                                project.current_file.as_ref().unwrap_or(&PathBuf::new()),
                            )
                            .unwrap_or_default(),
                            ..Default::default()
                        },
                    );
                }
                ui.close_menu();
            }
        });
        if ui.button("Delete").clicked() {
            match entry_type {
                EntryType::Directory => fs::remove_dir_all(&path).unwrap(),
                EntryType::File => fs::remove_file(&path).unwrap(),
                _ => {}
            }
            if &path == project.current_file.as_ref().unwrap_or(&PathBuf::new())
                || project
                    .current_file
                    .as_ref()
                    .unwrap_or(&PathBuf::new())
                    .starts_with(path.clone())
            {
                project.current_file = None;
                project.remove_file(&project.current_file.clone().unwrap_or_default())
            }
            project.remove_file(&path);
            ui.close_menu();
        }
    }
    ui.memory_mut(|w| w.data.insert_persisted(ui.id(), editable));
}

fn rename(original_path: &Path, new_name: &str) -> std::io::Result<PathBuf> {
    let new_path = original_path.with_file_name(new_name);

    if !new_path.exists() {
        fs::rename(original_path, &new_path)?;
        return Ok(new_path);
    }

    Err(std::io::Error::from_raw_os_error(0))
}
