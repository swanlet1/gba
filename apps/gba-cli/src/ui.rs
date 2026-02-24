//! UI/TUI implementation for GBA CLI.

use ratatui::{
    Frame,
    backend::CrosstermBackend,
    crossterm::{
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
    widgets::{Block, Borders, Paragraph},
};

use std::io::{self, Stdout};

use crate::error::Result;

/// Terminal UI state.
#[allow(dead_code)]
pub struct Tui {
    /// Terminal backend.
    _backend: CrosstermBackend<Stdout>,
}

impl Tui {
    /// Create a new TUI.
    ///
    /// # Errors
    ///
    /// Returns an error if the terminal cannot be initialized.
    #[allow(dead_code)]
    pub fn new() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);

        Ok(Self { _backend: backend })
    }

    /// Draw the UI frame.
    ///
    /// # Errors
    ///
    /// Returns an error if drawing fails.
    #[allow(dead_code)]
    pub fn draw(&mut self) -> Result<()> {
        // Frame drawing will be implemented when TUI logic is added
        Ok(())
    }

    /// Exit the TUI.
    ///
    /// # Errors
    ///
    /// Returns an error if cleanup fails.
    #[allow(dead_code)]
    pub fn exit(&mut self) -> Result<()> {
        disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen)?;
        Ok(())
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        let _ = self.exit();
    }
}

impl Default for Tui {
    fn default() -> Self {
        Self::new().expect("Failed to initialize TUI")
    }
}

/// Draw a simple message in the terminal.
///
/// # Errors
///
/// Returns an error if drawing fails.
#[allow(dead_code)]
pub fn draw_message(frame: &mut Frame, title: &str, content: &str) {
    let paragraph =
        Paragraph::new(content).block(Block::default().title(title).borders(Borders::ALL));
    frame.render_widget(paragraph, frame.area());
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_tui_creation() {
        // Note: TUI tests require terminal, may not work in headless environments
        // This test is a placeholder for structure
    }
}
