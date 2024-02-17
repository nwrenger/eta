use std::{fs, io, path::PathBuf};

use eframe::{
    egui::{self, Sense, Ui},
    epaint::{Color32, Vec2},
};

use crate::Project;

#[derive(PartialEq)]
enum EntryType {
    Directory,
    File,
}

pub fn init(ui: &mut Ui, project: &mut Project) {
    let header_text = "Explorer";
    ui.painter().rect_filled(
        egui::Rect {
            min: ui.max_rect().min + Vec2 { x: -2.0, y: 20.5 },
            max: ui.max_rect().max + Vec2 { x: 2.0, y: 0.0 },
        },
        1.0,
        Color32::from_gray(35),
    );
    ui.heading(header_text);
    egui::ScrollArea::vertical()
        .auto_shrink(true)
        .max_height(ui.available_height() - 3.0)
        .show(ui, |ui| {
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
                if let Some(path) = project.project_path.clone() {
                    file_side_bar(ui, &path, project).unwrap_or_default();
                } else if ui.button("Open Project").clicked() {
                    if let Some(project_path) = &rfd::FileDialog::new().pick_folder() {
                        if Some(project_path) != project.project_path.as_ref() {
                            *project = Project {
                                project_path: Some(project_path.to_path_buf()),
                                ..Default::default()
                            };
                        }
                    }
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
                    ui.set_enabled(project.file_edited.is_none());
                    let select = if project.file_edited.is_none() {
                        ui.selectable_label(false, file_name)
                            .interact(Sense::click_and_drag())
                    } else {
                        ui.selectable_label(false, file_name)
                            .interact(Sense::click_and_drag())
                            .interact(Sense {
                                click: false,
                                drag: false,
                                focusable: false,
                            })
                    };
                    select
                        .context_menu(|ui| ctx_menu(ui, project, file_name, EntryType::Directory));
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
            ui.set_enabled(project.file_edited.is_none());
            let select = if project.file_edited.is_none() {
                ui.selectable_value(&mut project.file_path, Some(entry.to_path_buf()), file_name)
                    .interact(Sense::click_and_drag())
            } else {
                ui.selectable_value(&mut project.file_path, Some(entry.to_path_buf()), file_name)
                    .interact(Sense {
                        click: false,
                        drag: false,
                        focusable: false,
                    })
            };
            if project.file_edited.is_none() {
                if select.clicked() {
                    project.file_content = Some(
                        fs::read_to_string(project.file_path.as_ref().unwrap_or(&PathBuf::new()))
                            .unwrap_or_default(),
                    );
                    if project.file_path != project.file_edited {
                        project.file_edited = None;
                    }
                }
                if select.dragged() {
                    println!("dragging");
                }
                select.context_menu(|ui| ctx_menu(ui, project, file_name, EntryType::File));
            }
        }
    }

    Ok(())
}

fn ctx_menu(ui: &mut Ui, _project: &mut Project, file_name: &str, entry_type: EntryType) {
    ui.label(file_name);
    if entry_type == EntryType::Directory {
        if ui.button("Add Directory").clicked() {
            ui.close_menu();
        }
        if ui.button("Add File").clicked() {
            ui.close_menu();
        }
    }
    if ui.button("Rename").clicked() {
        ui.close_menu();
    }
    if ui.button("Delete").clicked() {
        ui.close_menu();
    }
}
