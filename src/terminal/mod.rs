pub mod config;
pub mod error;
mod into;
pub mod prelude;
pub mod render;
pub mod term;

use eframe::egui;
use egui::{Response, Ui, Vec2, Widget};

pub use config::definitions::TermResult;
pub use config::term_config::{Config, Style};
pub use term::TermHandler;

pub struct Terminal<'a> {
    terminal: &'a mut TermHandler,
    size: Option<Vec2>,
}

impl Widget for Terminal<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let size = match self.size {
            Some(s) => s,
            None => ui.available_size(),
        };
        self.terminal
            .draw(ui, size)
            .expect("terminal should not error")
    }
}

impl<'a> Terminal<'a> {
    pub fn new(terminal: &'a mut TermHandler) -> Self {
        Self {
            terminal,
            size: None,
        }
    }

    pub fn with_size(mut self, size: Vec2) -> Self {
        self.size = Some(size);
        self
    }

    pub fn exit_status(&mut self) -> Option<u32> {
        self.terminal.exit_status()
    }
}
