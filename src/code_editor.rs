use std::fmt::Debug;

use eframe::{
    egui::{
        self,
        os::OperatingSystem,
        text::CCursor,
        text_edit::TextEditState,
        text_selection::{CCursorRange, CursorRange},
        CursorIcon, Event, EventFilter, Key, Modifiers, Response, TextBuffer, TextStyle, Ui,
    },
    epaint::{Color32, Galley, Vec2},
};
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct FileData {
    pub text: String,
    pub editor: EditorData,
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct EditorData {
    pub scroll_offset: f32,
    pub state: TextEditState,
}

impl Debug for EditorData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EditorData")
            .field("scroll_offset", &self.scroll_offset)
            .field(
                "state",
                &format!("{:?} {}", &self.state.cursor, &self.state.singleline_offset),
            )
            .finish()
    }
}

pub fn code_editor_ui(ui: &mut Ui, data: &mut FileData) -> Response {
    let FileData { text, editor } = data;
    let text: &mut dyn TextBuffer = text;
    let os = ui.ctx().os();

    // size
    let desired_size = ui.available_size();
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click_and_drag());
    let id = response.id;

    let painter = ui.painter();

    let line_count = text.as_str().split('\n').count();
    let text_offset = egui::Vec2 {
        x: (line_count.to_string().len() as f32 * 10.0).max(40.0),
        y: 0.0,
    };
    let line_number_position = rect.min;
    let text_position = rect.min + text_offset;
    let line_height = TextStyle::Monospace.resolve(ui.style()).size;

    let scroll_delta = if response.hovered() {
        // cursor
        ui.ctx().set_cursor_icon(CursorIcon::Text);

        //scrolling
        ui.input(|i| i.smooth_scroll_delta.y)
    } else {
        0.0
    };

    editor.scroll_offset -= scroll_delta;

    let total_text_height = line_count as f32 * line_height;

    // Adjust the maximum scroll scroll_offset calculation
    if total_text_height > rect.height() {
        // The last line should be able to scroll to the top of the viewport
        editor.scroll_offset = editor
            .scroll_offset
            .clamp(0.0, total_text_height - rect.height() / 2.0);
    } else {
        // If all content fits within the container, disable scrolling
        editor.scroll_offset = 0.0;
    }

    // Calculate visible lines considering the updated clamp logic
    let visible_lines = ((rect.height() / line_height).floor() as usize).min(line_count);
    let first_visible_line = (editor.scroll_offset / line_height).floor() as usize;

    // Now calculate the visible text lines, adjusted to not exceed the total line count
    let visible_text_lines = text
        .as_str()
        .lines()
        .skip(first_visible_line)
        .take(visible_lines)
        .collect::<Vec<&str>>()
        .join("\n");

    // Similar for line numbers
    let visible_line_numbers = (first_visible_line + 1
        ..=(first_visible_line + visible_lines).clamp(0, line_count))
        .map(|num| num.to_string() + "\n")
        .collect::<String>();

    let adjusted_line_number_position =
        line_number_position - egui::vec2(0.0, editor.scroll_offset % line_height);
    let adjusted_text_position =
        text_position - egui::vec2(0.0, editor.scroll_offset % line_height);

    let galley = painter.layout(
        visible_text_lines,
        TextStyle::Monospace.resolve(ui.style()),
        ui.visuals().text_color(),
        f32::INFINITY,
    );

    let mut cursor_range = editor.state.cursor.range(&galley).unwrap_or_default();

    // once before key input
    let mut undoer = editor.state.undoer();
    undoer.add_undo(&(cursor_range.as_ccursor_range(), text.as_str().to_owned()));
    editor.state.set_undoer(undoer);

    // getting keys
    if response.has_focus() {
        // filter
        let event_filter = EventFilter {
            tab: true,
            horizontal_arrows: true,
            vertical_arrows: true,
            ..Default::default()
        };
        ui.memory_mut(|mem| mem.set_focus_lock_filter(id, event_filter));

        // key presses
        let keys = ui.input(|i| i.events.clone());
        for key in keys {
            let did_mutate_text = match key {
                // First handle events that only changes the selection cursor, not the text:
                key if cursor_range.on_event(os, &key, &galley, id) => None,

                Event::Copy => {
                    if cursor_range.is_empty() {
                        ui.ctx().copy_text(text.as_str().to_string());
                    } else {
                        ui.ctx()
                            .copy_text(cursor_range.slice_str(text.as_str()).to_owned());
                    }
                    None
                }
                Event::Cut => {
                    if cursor_range.is_empty() {
                        ui.ctx().copy_text(text.take());
                        Some(CCursorRange::default())
                    } else {
                        ui.ctx()
                            .copy_text(cursor_range.slice_str(text.as_str()).to_owned());
                        Some(CCursorRange::one(text.delete_selected(&cursor_range)))
                    }
                }
                Event::Paste(text_to_insert) => {
                    if !text_to_insert.is_empty() {
                        let mut ccursor = text.delete_selected(&cursor_range);

                        text.insert_text_at(&mut ccursor, &text_to_insert, usize::MAX);

                        Some(CCursorRange::one(ccursor))
                    } else {
                        None
                    }
                }
                Event::Text(text_to_insert) => {
                    // Newlines are handled by `Key::Enter`.
                    if !text_to_insert.is_empty()
                        && text_to_insert != "\n"
                        && text_to_insert != "\r"
                    {
                        let mut ccursor = text.delete_selected(&cursor_range);

                        text.insert_text_at(&mut ccursor, &text_to_insert, usize::MAX);

                        Some(CCursorRange::one(ccursor))
                    } else {
                        None
                    }
                }
                Event::Key {
                    key: Key::Tab,
                    pressed: true,
                    modifiers,
                    ..
                } => {
                    let mut ccursor = text.delete_selected(&cursor_range);
                    if modifiers.shift {
                        // TODO(emilk): support removing indentation over a selection?
                        text.decrease_indentation(&mut ccursor);
                    } else {
                        text.insert_text_at(&mut ccursor, "\t", usize::MAX);
                    }
                    Some(CCursorRange::one(ccursor))
                }
                Event::Key {
                    key: Key::Enter,
                    pressed: true,
                    ..
                } => {
                    let mut ccursor = text.delete_selected(&cursor_range);
                    text.insert_text_at(&mut ccursor, "\n", usize::MAX);
                    // TODO(emilk): if code editor, auto-indent by same leading tabs, + one if the lines end on an opening bracket
                    Some(CCursorRange::one(ccursor))
                }
                Event::Key {
                    key: Key::Z,
                    pressed: true,
                    modifiers,
                    ..
                } if modifiers.matches_logically(Modifiers::COMMAND) => {
                    if let Some((undo_ccursor_range, undo_txt)) = editor.state.undoer().undo(&(
                        editor.state.cursor.char_range().unwrap(),
                        text.as_str().to_owned(),
                    )) {
                        text.replace_with(undo_txt);
                        Some(*undo_ccursor_range)
                    } else {
                        None
                    }
                }

                Event::Key {
                    modifiers,
                    key,
                    pressed: true,
                    ..
                } => check_for_mutating_key_press(
                    os,
                    &mut cursor_range,
                    text,
                    &galley,
                    &modifiers,
                    key,
                ),
                _ => None,
            };
            if let Some(new_ccursor_range) = did_mutate_text {
                // Layout again to avoid frame delay, and to keep `text` and `galley` in sync. TODO: Add this
                // *galley = layouter(ui, text.as_str(), wrap_width);

                // Set cursor_range using new galley:
                cursor_range = CursorRange {
                    primary: galley.from_ccursor(new_ccursor_range.primary),
                    secondary: galley.from_ccursor(new_ccursor_range.secondary),
                };

                response.mark_changed();
            }
        }
    }
    editor.state.cursor.set_range(Some(cursor_range));

    // once after key input
    let mut undoer = editor.state.undoer();
    undoer.add_undo(&(cursor_range.as_ccursor_range(), text.as_str().to_owned()));
    editor.state.set_undoer(undoer);

    // info shit
    response.widget_info(|| egui::WidgetInfo::text_edit(text.as_str(), text.as_str()));

    // painting
    if ui.is_rect_visible(rect) {
        let cursor_pos = galley.pos_from_cursor(&cursor_range.primary).min;
        let visuals = ui.style().interact(&response);
        painter.rect(rect, 2.0, Color32::from_gray(35), visuals.bg_stroke);

        painter.with_clip_rect(rect).text(
            adjusted_line_number_position,
            egui::Align2::LEFT_TOP,
            visible_line_numbers,
            TextStyle::Monospace.resolve(ui.style()),
            ui.visuals().text_color(),
        );

        painter
            .with_clip_rect(rect)
            .galley(adjusted_text_position, galley, Color32::WHITE);

        // Adjust the cursor position with the scrolling offset
        let adjusted_cursor_pos =
            cursor_pos + adjusted_text_position.to_vec2() - egui::vec2(0.0, editor.scroll_offset);

        let cursor_is_visible =
            adjusted_cursor_pos.y >= rect.min.y && adjusted_cursor_pos.y <= rect.max.y;
        // Render the cursor
        if ui.memory(|r| r.has_focus(id)) && cursor_is_visible {
            let cursor_width = 2.0;
            let cursor_height = line_height + 1.0;

            let cursor_rect = egui::Rect::from_min_size(
                adjusted_cursor_pos,
                Vec2::new(cursor_width, cursor_height),
            );

            painter.with_clip_rect(rect).rect_filled(
                cursor_rect,
                0.5,
                ui.visuals().strong_text_color(),
            );
        }
    }

    if response.clicked() && !response.lost_focus() {
        ui.memory_mut(|mem| mem.request_focus(id));
    }

    response
}

pub fn code_editor(data: &mut FileData) -> impl egui::Widget + '_ {
    move |ui: &mut egui::Ui| code_editor_ui(ui, data)
}

/// Returns `Some(new_cursor)` if we did mutate `text`.
fn check_for_mutating_key_press(
    os: OperatingSystem,
    cursor_range: &mut CursorRange,
    text: &mut dyn TextBuffer,
    galley: &Galley,
    modifiers: &Modifiers,
    key: Key,
) -> Option<CCursorRange> {
    match key {
        Key::Backspace => {
            let ccursor = if modifiers.mac_cmd {
                text.delete_paragraph_before_cursor(galley, cursor_range)
            } else if let Some(cursor) = cursor_range.single() {
                if modifiers.alt || modifiers.ctrl {
                    // alt on mac, ctrl on windows
                    text.delete_previous_word(cursor.ccursor)
                } else {
                    text.delete_previous_char(cursor.ccursor)
                }
            } else {
                text.delete_selected(cursor_range)
            };
            Some(CCursorRange::one(ccursor))
        }

        Key::Delete if !modifiers.shift || os != OperatingSystem::Windows => {
            let ccursor = if modifiers.mac_cmd {
                text.delete_paragraph_after_cursor(galley, cursor_range)
            } else if let Some(cursor) = cursor_range.single() {
                if modifiers.alt || modifiers.ctrl {
                    // alt on mac, ctrl on windows
                    text.delete_next_word(cursor.ccursor)
                } else {
                    text.delete_next_char(cursor.ccursor)
                }
            } else {
                text.delete_selected(cursor_range)
            };
            let ccursor = CCursor {
                prefer_next_row: true,
                ..ccursor
            };
            Some(CCursorRange::one(ccursor))
        }

        Key::H if modifiers.ctrl => {
            let ccursor = text.delete_previous_char(cursor_range.primary.ccursor);
            Some(CCursorRange::one(ccursor))
        }

        Key::K if modifiers.ctrl => {
            let ccursor = text.delete_paragraph_after_cursor(galley, cursor_range);
            Some(CCursorRange::one(ccursor))
        }

        Key::U if modifiers.ctrl => {
            let ccursor = text.delete_paragraph_before_cursor(galley, cursor_range);
            Some(CCursorRange::one(ccursor))
        }

        Key::W if modifiers.ctrl => {
            let ccursor = if let Some(cursor) = cursor_range.single() {
                text.delete_previous_word(cursor.ccursor)
            } else {
                text.delete_selected(cursor_range)
            };
            Some(CCursorRange::one(ccursor))
        }

        _ => None,
    }
}
