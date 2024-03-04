use std::fmt::Debug;

use eframe::{
    egui::{
        self,
        os::OperatingSystem,
        text::CCursor,
        text_edit::TextEditState,
        text_selection::{CCursorRange, CursorRange},
        vec2, CursorIcon, Event, EventFilter, Key, Modifiers, Response, Stroke, TextBuffer,
        TextStyle, Ui,
    },
    epaint::{Color32, Galley, Vec2},
};
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct FileData {
    pub text: String,
    pub editor: ExtendedCodeEditor,
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct ExtendedCodeEditor {
    pub scroll_offset: f32,
    pub target_scroll_offset: f32,
    pub state: TextEditState,
}

impl Debug for ExtendedCodeEditor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExtendedCodeEditor")
            .field("scroll_offset", &self.scroll_offset)
            .field("state", &self.state.cursor)
            .finish()
    }
}
impl ExtendedCodeEditor {
    pub fn _ui(ui: &mut Ui, data: &mut FileData) -> Response {
        let FileData { text, editor } = data;
        let text: &mut dyn TextBuffer = text;
        let prev_text = text.as_str().to_string();

        // init
        let os = ui.ctx().os();
        let desired_size = ui.available_size();
        let (rect, mut response) =
            ui.allocate_exact_size(desired_size, egui::Sense::click_and_drag());
        let id = response.id;
        let font = TextStyle::Monospace.resolve(ui.style());
        let painter = ui.painter();
        let char_limit = usize::MAX;

        // colors
        let secondary = ui.style().visuals.faint_bg_color;
        let stroke = ui.style().visuals.window_stroke;

        let line_count = text.as_str().split('\n').count();
        let text_offset = egui::Vec2 {
            x: (line_count.to_string().len() as f32 * 10.0).max(40.0),
            y: 0.0,
        };
        let line_number_position = rect.min;
        let text_position = rect.min + text_offset;
        let line_height = font.size;

        let line_numbers = (1..=line_count)
            .map(|num| num.to_string() + "\n")
            .collect::<String>();

        let scroll_delta = if response.hovered() {
            // cursor
            ui.ctx().set_cursor_icon(CursorIcon::Text);

            //scrolling
            ui.input(|i| i.smooth_scroll_delta.y)
        } else {
            0.0
        };

        const SMOOTHING_SPEED: f32 = 30.0;
        let dt = ui.input(|i| i.unstable_dt);

        editor.target_scroll_offset -= scroll_delta;

        // Smoothly move towards the target scroll offset
        let delta = editor.target_scroll_offset - editor.scroll_offset;
        if delta.abs() < 1.0 {
            editor.scroll_offset = editor.target_scroll_offset;
        } else {
            editor.scroll_offset += delta * (dt * SMOOTHING_SPEED).min(1.0);
        }

        let mut galley = painter.layout(
            text.as_str().to_string(),
            font.clone(),
            ui.visuals().text_color(),
            f32::INFINITY,
        );

        let total_text_height = galley.size().y;

        if total_text_height > rect.height() {
            editor.scroll_offset = editor
                .target_scroll_offset
                .clamp(0.0, total_text_height - rect.height() / 2.0);
            editor.target_scroll_offset = editor
                .target_scroll_offset
                .clamp(0.0, total_text_height - rect.height() / 2.0);
        } else {
            editor.scroll_offset = 0.0;
            editor.target_scroll_offset = 0.0;
        }

        let adjusted_line_number_position =
            line_number_position - egui::vec2(0.0, editor.scroll_offset);
        let adjusted_text_position = text_position - egui::vec2(0.0, editor.scroll_offset);

        let mut cursor_range = editor.state.cursor.range(&galley).unwrap_or_default();

        // once before key input
        let mut undoer = editor.state.undoer();
        undoer.feed_state(
            ui.input(|i| i.time),
            &(cursor_range.as_ccursor_range(), text.as_str().to_owned()),
        );
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
            let events = ui.input(|i| i.events.clone());
            for event in events {
                let did_mutate_text = match event {
                    // First handle events that only changes the selection cursor, not the text:
                    event if cursor_range.on_event(os, &event, &galley, id) => None,

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

                            text.insert_text_at(&mut ccursor, &text_to_insert, char_limit);

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

                            text.insert_text_at(&mut ccursor, &text_to_insert, char_limit);

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
                            text.insert_text_at(&mut ccursor, "\t", char_limit);
                        }
                        Some(CCursorRange::one(ccursor))
                    }
                    Event::Key {
                        key: Key::Enter,
                        pressed: true,
                        ..
                    } => {
                        let mut ccursor = text.delete_selected(&cursor_range);
                        text.insert_text_at(&mut ccursor, "\n", char_limit);
                        // TODO(emilk): if code editor, auto-indent by same leading tabs, + one if the lines end on an opening bracket
                        Some(CCursorRange::one(ccursor))
                    }
                    Event::Key {
                        key: Key::Z,
                        pressed: true,
                        modifiers,
                        ..
                    } if modifiers.matches_logically(Modifiers::COMMAND) => {
                        if let Some((undo_ccursor_range, undo_txt)) = editor
                            .state
                            .undoer()
                            .undo(&(cursor_range.as_ccursor_range(), text.as_str().to_string()))
                        {
                            text.replace_with(undo_txt);
                            Some(*undo_ccursor_range)
                        } else {
                            None
                        }
                    }
                    Event::Key {
                        key,
                        pressed: true,
                        modifiers,
                        ..
                    } if (modifiers.matches_logically(Modifiers::COMMAND) && key == Key::Y)
                        || (modifiers.matches_logically(Modifiers::SHIFT | Modifiers::COMMAND)
                            && key == Key::Z) =>
                    {
                        if let Some((redo_ccursor_range, redo_txt)) = editor
                            .state
                            .undoer()
                            .redo(&(cursor_range.as_ccursor_range(), text.as_str().to_string()))
                        {
                            text.replace_with(redo_txt);
                            Some(*redo_ccursor_range)
                        } else {
                            None
                        }
                    }
                    Event::Key {
                        modifiers,
                        key,
                        pressed: true,
                        ..
                    } => Self::check_for_mutating_key_press(
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
                    galley = painter.layout(
                        text.as_str().to_string(),
                        font.clone(),
                        ui.visuals().text_color(),
                        f32::INFINITY,
                    );

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
        undoer.feed_state(
            ui.input(|i| i.time),
            &(cursor_range.as_ccursor_range(), text.as_str().to_owned()),
        );
        editor.state.set_undoer(undoer);

        // info shit
        response.widget_info(|| egui::WidgetInfo::text_edit(prev_text.to_string(), text.as_str()));

        // painting
        if ui.is_rect_visible(rect) {
            let cursor_pos = galley.pos_from_cursor(&cursor_range.primary).min;
            painter.rect(rect, 1.0, secondary, stroke);

            painter.with_clip_rect(rect).text(
                adjusted_line_number_position,
                egui::Align2::LEFT_TOP,
                line_numbers,
                TextStyle::Monospace.resolve(ui.style()),
                ui.visuals().text_color(),
            );

            painter
                .with_clip_rect(rect)
                .galley(adjusted_text_position, galley, Color32::WHITE);

            let adjusted_cursor_pos = cursor_pos + adjusted_text_position.to_vec2();

            let cursor_is_visible = adjusted_cursor_pos.y >= rect.min.y - line_height
                && adjusted_cursor_pos.y <= rect.max.y;
            // Render the cursor
            if ui.memory(|r| r.has_focus(id)) && cursor_is_visible {
                let cursor_width = 1.5;
                let cursor_height = line_height + 2.0;
                let cursor_rect = egui::Rect::from_min_size(
                    adjusted_cursor_pos - vec2(cursor_width / 2., 0.),
                    Vec2::new(cursor_width, cursor_height),
                );

                painter.with_clip_rect(rect).rect(
                    cursor_rect,
                    egui::Rounding::same(0.75),
                    ui.visuals().strong_text_color(),
                    Stroke::NONE,
                );
            }
        }

        if response.clicked() && !response.lost_focus() {
            ui.memory_mut(|mem| mem.request_focus(id));
        }

        response
    }

    pub fn ui(data: &mut FileData) -> impl egui::Widget + '_ {
        move |ui: &mut egui::Ui| Self::_ui(ui, data)
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
}

pub trait ExtendedCodeEditorSpawner {
    fn ext_code_ui(&mut self, data: &mut FileData) -> Response;
}

impl ExtendedCodeEditorSpawner for Ui {
    fn ext_code_ui(&mut self, data: &mut FileData) -> Response {
        self.add(ExtendedCodeEditor::ui(data))
    }
}
