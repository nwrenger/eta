use eframe::{
    egui::{
        self,
        os::OperatingSystem,
        text::CCursor,
        text_selection::{CCursorRange, CursorRange},
        Event, Id, Key, Modifiers, Response, TextBuffer, TextStyle, Ui,
    },
    epaint::{Color32, Galley},
};
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Serialize, Debug, Deserialize)]
struct ScrollData {
    offset: f32,
    cursor: CCursor,
    cursor_range: CursorRange,
}

pub fn code_editor_ui(ui: &mut Ui, id: Id, text: &mut dyn TextBuffer) -> Response {
    let os = ui.ctx().os();
    let mut state: ScrollData = ui
        .memory_mut(|mem| mem.data.get_persisted(id))
        .unwrap_or_default();
    fn my_memoized_highlighter(_: &str) -> egui::text::LayoutJob {
        Default::default()
    }
    let layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
        let mut layout_job: egui::text::LayoutJob = my_memoized_highlighter(string);
        layout_job.wrap.max_width = wrap_width;
        ui.fonts(|f| f.layout_job(layout_job))
    };

    let galley = layouter(ui, text.as_str(), ui.available_width());

    // size
    let desired_size = ui.available_size();
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click_and_drag());

    // getting keys
    if response.has_focus() {
        let keys = ui.input(|i| i.events.clone());
        for key in keys {
            let did_mutate_text = match key {
                Event::Text(text_to_insert) => {
                    // Newlines are handled by `Key::Enter`.
                    if !text_to_insert.is_empty()
                        && text_to_insert != "\n"
                        && text_to_insert != "\r"
                    {
                        // state.cursor = text.delete_selected(&state.cursor_range);

                        text.insert_text_at(&mut state.cursor, &text_to_insert, usize::MAX);

                        Some(CCursorRange::one(state.cursor))
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
                    &mut state.cursor_range,
                    text,
                    &galley,
                    &modifiers,
                    key,
                ),
                _ => None,
            };
            if let Some(new_cursor_range) = did_mutate_text {
                state.cursor_range = CursorRange {
                    primary: galley.from_ccursor(new_cursor_range.primary),
                    secondary: galley.from_ccursor(new_cursor_range.secondary),
                };
                response.mark_changed();
            }
        }
    }

    // info shit
    response.widget_info(|| egui::WidgetInfo::text_edit(text.as_str(), text.as_str()));

    // painting
    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        let painter = ui.painter();
        painter.rect(rect, 2.0, Color32::from_gray(35), visuals.bg_stroke);

        let line_number_position = rect.min;
        let text_position = rect.min + egui::Vec2 { x: 50.0, y: 1.0 };
        let line_height = TextStyle::Monospace.resolve(ui.style()).size;
        let line_count = text.as_str().lines().count();

        let scroll_delta = if response.hovered() {
            ui.input(|i| i.smooth_scroll_delta.y)
        } else {
            0.0
        };

        state.offset -= scroll_delta;

        let total_text_height = line_count as f32 * line_height;

        // Adjust the maximum scroll offset calculation
        if total_text_height > rect.height() {
            // The last line should be able to scroll to the top of the viewport
            let max_scroll_offset = total_text_height - (rect.height() - line_height).max(0.0);
            state.offset = state.offset.clamp(0.0, max_scroll_offset);
        } else {
            // If all content fits within the container, disable scrolling
            state.offset = 0.0;
        }

        // Persist the updated scroll offset
        ui.memory_mut(|mem: &mut egui::Memory| mem.data.insert_persisted(id, state.offset));

        // Calculate visible lines considering the updated clamp logic
        let visible_lines = ((rect.height() / line_height).floor() as usize).min(line_count);
        let first_visible_line = (state.offset / line_height).floor() as usize;

        // Now you can accurately calculate which lines are visible
        let visible_text_lines = text
            .as_str()
            .lines()
            .skip(first_visible_line)
            .take(visible_lines)
            .collect::<Vec<&str>>()
            .join("\n");

        // Similar for line numbers
        let visible_line_numbers = (first_visible_line + 1..=first_visible_line + visible_lines)
            .map(|num| num.to_string() + "\n")
            .collect::<String>();

        let adjusted_line_number_position =
            line_number_position - egui::vec2(0.0, state.offset % line_height);
        let adjusted_text_position = text_position - egui::vec2(0.0, state.offset % line_height);

        painter.with_clip_rect(rect).text(
            adjusted_line_number_position,
            egui::Align2::LEFT_TOP,
            visible_line_numbers,
            TextStyle::Monospace.resolve(ui.style()),
            ui.visuals().text_color(),
        );

        painter.with_clip_rect(rect).text(
            adjusted_text_position,
            egui::Align2::LEFT_TOP,
            visible_text_lines,
            TextStyle::Monospace.resolve(ui.style()),
            ui.visuals().text_color(),
        );
    }

    if response.clicked() && !response.lost_focus() {
        ui.memory_mut(|mem| mem.request_focus(response.id));
    }

    // save data
    ui.memory_mut(|mem| mem.data.insert_persisted(id, state));

    response
}

pub fn code_editor(id: Id, text: &mut dyn TextBuffer) -> impl egui::Widget + '_ {
    move |ui: &mut egui::Ui| code_editor_ui(ui, id, text)
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
                    dbg!(key);
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
