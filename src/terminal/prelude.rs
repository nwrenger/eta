use eframe::egui;
use egui::{Response, Ui, Vec2};

pub use super::term::TermHandler;
pub use super::Terminal;

pub trait TerminalSpawner {
    fn terminal(&mut self, term: &mut TermHandler) -> Response;
    fn terminal_sized(&mut self, term: &mut TermHandler, size: Vec2) -> Response;
}

impl TerminalSpawner for Ui {
    fn terminal(&mut self, term: &mut TermHandler) -> Response {
        self.add(Terminal::new(term))
    }

    fn terminal_sized(&mut self, term: &mut TermHandler, size: Vec2) -> Response {
        self.add(Terminal::new(term).with_size(size))
    }
}
