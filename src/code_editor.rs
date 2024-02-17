use eframe::{
    egui::{
        self, text::CCursor, text_selection::CCursorRange, Event, Id, Response, TextBuffer,
        TextStyle, Ui,
    },
    epaint::Color32,
};

pub fn code_editor_ui(ui: &mut Ui, id: Id, text: &mut dyn TextBuffer) -> Response {
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
                        // let mut ccursor = text.delete_selected(&cursor_range);

                        let mut ccursor = CCursor::new(0);

                        text.insert_text_at(&mut ccursor, &text_to_insert, usize::MAX);

                        Some(CCursorRange::one(ccursor))
                    } else {
                        None
                    }
                }
                _ => None,
            };
            if did_mutate_text.is_some() {
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

        // Retrieve or initialize the scroll offset
        let mut scroll_offset = ui
            .memory_mut(|mem| mem.data.get_persisted(id))
            .unwrap_or(0.0);

        let scroll_delta = if response.hovered() {
            ui.input(|i| i.smooth_scroll_delta.y)
        } else {
            0.0
        };

        scroll_offset -= scroll_delta;

        let total_text_height = line_count as f32 * line_height;

        // Adjust the maximum scroll offset calculation
        if total_text_height > rect.height() {
            // The last line should be able to scroll to the top of the viewport
            let max_scroll_offset = total_text_height - (rect.height() - line_height).max(0.0);
            scroll_offset = scroll_offset.clamp(0.0, max_scroll_offset);
        } else {
            // If all content fits within the container, disable scrolling
            scroll_offset = 0.0;
        }

        // Persist the updated scroll offset
        ui.memory_mut(|mem: &mut egui::Memory| mem.data.insert_persisted(id, scroll_offset));

        // Calculate visible lines considering the updated clamp logic
        let visible_lines = ((rect.height() / line_height).floor() as usize).min(line_count);
        let first_visible_line = (scroll_offset / line_height).floor() as usize;

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
            line_number_position - egui::vec2(0.0, scroll_offset % line_height);
        let adjusted_text_position = text_position - egui::vec2(0.0, scroll_offset % line_height);

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

    response
}

pub fn code_editor(id: Id, text: &mut dyn TextBuffer) -> impl egui::Widget + '_ {
    move |ui: &mut egui::Ui| code_editor_ui(ui, id, text)
}
