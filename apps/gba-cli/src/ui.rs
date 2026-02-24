//! UI/TUI implementation for GBA CLI.
//!
//! This module provides terminal user interface functionality using ratatui.

use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    crossterm::{
        event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use std::io::{self, Stdout};
use tracing::debug;

use crate::error::Result;

/// TUI state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TuiState {
    /// Initial state.
    Initial,
    /// Running state.
    Running,
    /// Paused state.
    #[allow(dead_code)]
    Paused,
    /// Completed state.
    #[allow(dead_code)]
    Completed,
    /// Error state.
    #[allow(dead_code)]
    Error,
}

/// TUI state.
#[allow(dead_code)]
pub struct Tui {
    /// Terminal instance.
    terminal: Terminal<CrosstermBackend<Stdout>>,
    /// Current state.
    state: TuiState,
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
        let terminal = Terminal::new(backend)?;

        debug!("TUI initialized");

        Ok(Self {
            terminal,
            state: TuiState::Initial,
        })
    }

    /// Draw the UI frame.
    ///
    /// # Errors
    ///
    /// Returns an error if drawing fails.
    #[allow(dead_code)]
    pub fn draw(&mut self) -> Result<()> {
        let state = self.state;
        self.terminal.draw(|f| {
            let size = f.area();

            // Create main layout
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints(
                    [
                        Constraint::Length(3), // Header
                        Constraint::Min(0),    // Main content
                        Constraint::Length(3), // Footer
                    ]
                    .as_ref(),
                )
                .split(size);

            // Render header
            Self::render_header(f, chunks[0]);

            // Render main content
            Self::render_main_content(f, chunks[1], state);

            // Render footer
            Self::render_footer(f, chunks[2]);
        })?;
        Ok(())
    }

    /// Run the main TUI loop.
    ///
    /// # Errors
    ///
    /// Returns an error if the TUI loop fails.
    #[allow(dead_code)]
    pub fn run(&mut self) -> Result<()> {
        debug!("Starting TUI loop");

        self.state = TuiState::Running;

        loop {
            self.draw()?;

            // Check for events
            if event::poll(std::time::Duration::from_millis(100))?
                && let Event::Key(key) = event::read()?
                && key == KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty())
            {
                break;
            }

            // Check if we should exit
            if matches!(self.state, TuiState::Completed | TuiState::Error) {
                break;
            }
        }

        debug!("TUI loop completed");
        Ok(())
    }

    /// Render a single frame.
    #[allow(dead_code)]
    fn render_frame(&self, f: &mut Frame, state: TuiState) {
        let size = f.area();

        // Create main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints(
                [
                    Constraint::Length(3), // Header
                    Constraint::Min(0),    // Main content
                    Constraint::Length(3), // Footer
                ]
                .as_ref(),
            )
            .split(size);

        // Render header
        Self::render_header(f, chunks[0]);

        // Render main content
        Self::render_main_content(f, chunks[1], state);

        // Render footer
        Self::render_footer(f, chunks[2]);
    }

    /// Render the header section.
    fn render_header(f: &mut Frame, area: Rect) {
        let title = Paragraph::new("GBA - GeekTime Bootcamp Agent")
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            );

        f.render_widget(title, area);
    }

    /// Render the main content section.
    fn render_main_content(f: &mut Frame, area: Rect, state: TuiState) {
        let content = match state {
            TuiState::Initial => "Initializing...",
            TuiState::Running => "Running task...",
            TuiState::Paused => "Paused. Press 'r' to resume or 'q' to quit.",
            TuiState::Completed => "Task completed successfully!",
            TuiState::Error => "An error occurred.",
        };

        let paragraph = Paragraph::new(content)
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: false })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Status")
                    .title_style(Style::default().fg(Color::Yellow)),
            );

        f.render_widget(paragraph, area);
    }

    /// Render the footer section.
    fn render_footer(f: &mut Frame, area: Rect) {
        let help_text = "Press 'q' to quit";

        let paragraph = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));

        f.render_widget(paragraph, area);
    }

    /// Exit the TUI.
    ///
    /// # Errors
    ///
    /// Returns an error if cleanup fails.
    #[allow(dead_code)]
    pub fn exit(&mut self) -> Result<()> {
        debug!("Exiting TUI");
        disable_raw_mode()?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen)?;
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
/// # Arguments
///
/// * `f` - The terminal frame.
/// * `title` - The title for the message box.
/// * `content` - The content to display.
#[allow(dead_code)]
pub fn draw_message(f: &mut Frame, title: &str, content: &str) {
    let paragraph = Paragraph::new(content).wrap(Wrap { trim: false }).block(
        Block::default()
            .title(title)
            .title_style(Style::default().fg(Color::Yellow))
            .borders(Borders::ALL),
    );

    f.render_widget(paragraph, f.area());
}

/// Draw a progress indicator.
///
/// # Arguments
///
/// * `f` - The terminal frame.
/// * `area` - The area to draw in.
/// * `message` - The message to display.
/// * `progress` - Progress value (0.0 to 1.0).
#[allow(dead_code)]
pub fn draw_progress(f: &mut Frame, area: Rect, message: &str, progress: f32) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(1), // Message
                Constraint::Length(1), // Progress bar
            ]
            .as_ref(),
        )
        .split(area);

    // Message
    let msg_paragraph = Paragraph::new(message).style(Style::default().fg(Color::White));
    f.render_widget(msg_paragraph, chunks[0]);

    // Progress bar
    let bar_width = area.width as usize - 2;
    let filled = (progress * bar_width as f32) as usize;
    let filled_bar = "=".repeat(filled);
    let empty_bar = " ".repeat(bar_width - filled);
    let bar_text = format!("[{}{}] {:.0}%", filled_bar, empty_bar, progress * 100.0);

    let bar_paragraph = Paragraph::new(bar_text).style(Style::default().fg(Color::Green));
    f.render_widget(bar_paragraph, chunks[1]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tui_state() {
        assert_eq!(TuiState::Initial, TuiState::Initial);
        assert_ne!(TuiState::Initial, TuiState::Running);
    }
}
