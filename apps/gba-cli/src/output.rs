//! Output formatting and display for GBA CLI.
//!
//! This module provides utilities for formatted output to stdout.

use std::io::{self, Write};

/// Output formatter for CLI messages.
#[derive(Debug)]
pub struct OutputFormatter {
    /// Use colors in output.
    colors_enabled: bool,
}

impl OutputFormatter {
    /// Create a new output formatter.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new output formatter with color control.
    #[must_use]
    #[allow(dead_code)]
    pub const fn with_colors(mut self, colors_enabled: bool) -> Self {
        self.colors_enabled = colors_enabled;
        self
    }

    /// Print a success message.
    pub fn success(&self, message: &str) {
        let prefix = if self.colors_enabled {
            "\x1b[32m✓\x1b[0m"
        } else {
            "✓"
        };
        println!("{} {}", prefix, message);
    }

    /// Print an error message.
    #[allow(dead_code)]
    pub fn error(&self, message: &str) {
        let prefix = if self.colors_enabled {
            "\x1b[31m✗\x1b[0m"
        } else {
            "✗"
        };
        eprintln!("{} {}", prefix, message);
    }

    /// Print a warning message.
    pub fn warning(&self, message: &str) {
        let prefix = if self.colors_enabled {
            "\x1b[33m⚠\x1b[0m"
        } else {
            "⚠"
        };
        println!("{} {}", prefix, message);
    }

    /// Print an info message.
    pub fn info(&self, message: &str) {
        let prefix = if self.colors_enabled {
            "\x1b[36mℹ\x1b[0m"
        } else {
            "ℹ"
        };
        println!("{} {}", prefix, message);
    }

    /// Print a section header.
    pub fn section(&self, title: &str) {
        println!("\n{}", Self::bold(title, self.colors_enabled));
        println!("{}", Self::repeat_char("=", title.len()));
    }

    /// Print a subsection header.
    pub fn subsection(&self, title: &str) {
        println!("\n{}", Self::underline(title, self.colors_enabled));
    }

    /// Print a list item.
    pub fn list_item(&self, prefix: &str, content: &str) {
        println!("  {} {}", prefix, content);
    }

    /// Print a bullet list item.
    #[allow(dead_code)]
    pub fn bullet(&self, content: &str) {
        self.list_item("•", content);
    }

    /// Print a numbered list item.
    pub fn numbered(&self, index: usize, content: &str) {
        self.list_item(&format!("{}.", index), content);
    }

    /// Print a separator line.
    pub fn separator(&self) {
        println!();
        println!("{}", Self::repeat_char("-", 80));
        println!();
    }

    /// Print prompt output with formatting.
    pub fn prompt_output(&self, template: &str, content: &str) {
        self.section(template);
        println!("{}", content);
        self.separator();
    }

    /// Print prompt list.
    pub fn prompt_list(&self, prompts: &[String], verbose: bool) {
        self.section("Available Prompts");

        for (i, prompt) in prompts.iter().enumerate() {
            self.numbered(i + 1, prompt);
            if verbose {
                // In verbose mode, we could show template config
                self.list_item("  Template:", prompt);
            }
        }

        println!("\nTotal: {} prompts", prompts.len());
    }

    /// Print feature information.
    pub fn feature_info(&self, name: &str, id: &str, description: Option<&str>) {
        self.section("Feature Information");
        self.list_item("Name:", name);
        self.list_item("ID:", id);
        if let Some(desc) = description {
            self.list_item("Description:", desc);
        }
    }

    /// Print task status.
    #[allow(dead_code)]
    pub fn task_status(&self, status: TaskStatus) {
        let (icon, text) = match status {
            TaskStatus::Pending => ("○", "Pending"),
            TaskStatus::InProgress => ("◐", "In Progress"),
            TaskStatus::Completed => ("●", "Completed"),
            TaskStatus::Failed => ("✗", "Failed"),
        };

        let prefix = if self.colors_enabled {
            match status {
                TaskStatus::Pending => "\x1b[90m○\x1b[0m",
                TaskStatus::InProgress => "\x1b[33m◐\x1b[0m",
                TaskStatus::Completed => "\x1b[32m●\x1b[0m",
                TaskStatus::Failed => "\x1b[31m✗\x1b[0m",
            }
        } else {
            icon
        };

        println!("{} {}", prefix, text);
    }

    /// Print a progress bar.
    #[allow(dead_code)]
    pub fn progress(&self, current: usize, total: usize, message: &str) {
        let percentage = if total > 0 {
            (current * 100) / total
        } else {
            0
        };
        let bar_width = 40;
        let filled = (percentage * bar_width) / 100;

        let bar = if self.colors_enabled {
            format!(
                "\x1b[36m{}\x1b[0m{}",
                Self::repeat_char("=", filled),
                Self::repeat_char(" ", bar_width - filled)
            )
        } else {
            format!(
                "{}{}",
                Self::repeat_char("=", filled),
                Self::repeat_char(" ", bar_width - filled)
            )
        };

        print!(
            "\r{} [{}{}] {}/{} ({})",
            message, bar, percentage, current, total, percentage
        );
        io::stdout().flush().unwrap();
    }

    /// Clear the progress line.
    #[allow(dead_code)]
    pub fn clear_progress(&self, width: usize) {
        print!("\r{}\r", Self::repeat_char(" ", width));
        io::stdout().flush().unwrap();
    }

    /// Print formatted code block.
    #[allow(dead_code)]
    pub fn code_block(&self, language: Option<&str>, code: &str) {
        println!();
        if let Some(lang) = language {
            println!("```{}", lang);
        } else {
            println!("```");
        }
        println!("{}", code);
        println!("```");
    }

    /// Check if colors are enabled.
    #[must_use]
    #[allow(dead_code)]
    pub fn is_colors_enabled(&self) -> bool {
        self.colors_enabled
    }

    /// Helper function to create bold text.
    fn bold(text: &str, colors_enabled: bool) -> String {
        if colors_enabled {
            format!("\x1b[1m{}\x1b[0m", text)
        } else {
            text.to_string()
        }
    }

    /// Helper function to create underlined text.
    fn underline(text: &str, colors_enabled: bool) -> String {
        if colors_enabled {
            format!("\x1b[4m{}\x1b[0m", text)
        } else {
            text.to_string()
        }
    }

    /// Helper function to repeat a character.
    fn repeat_char(c: &str, count: usize) -> String {
        c.repeat(count)
    }
}

impl Default for OutputFormatter {
    fn default() -> Self {
        // Check if we should use colors based on terminal support
        let colors_enabled = atty::is(atty::Stream::Stdout);
        Self { colors_enabled }
    }
}

/// Task status for display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum TaskStatus {
    /// Task is pending.
    Pending,
    /// Task is in progress.
    InProgress,
    /// Task is completed.
    Completed,
    /// Task failed.
    Failed,
}

/// Check if the terminal supports colors.
#[must_use]
#[allow(dead_code)]
pub fn terminal_supports_colors() -> bool {
    atty::is(atty::Stream::Stdout)
}

/// Print a simple message without formatting.
#[allow(dead_code)]
pub fn print(message: &str) {
    println!("{}", message);
}

/// Print a message with a prefix.
#[allow(dead_code)]
pub fn print_with_prefix(prefix: &str, message: &str) {
    println!("{} {}", prefix, message);
}

/// Print an error message to stderr.
#[allow(dead_code)]
pub fn print_error(message: &str) {
    eprintln!("Error: {}", message);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_formatter() {
        let formatter = OutputFormatter::new().with_colors(false);
        // Just test that it doesn't panic
        formatter.success("Test success");
        formatter.error("Test error");
        formatter.warning("Test warning");
        formatter.info("Test info");
    }

    #[test]
    fn test_task_status() {
        assert_eq!(TaskStatus::Pending, TaskStatus::Pending);
        assert_ne!(TaskStatus::Pending, TaskStatus::Completed);
    }
}
