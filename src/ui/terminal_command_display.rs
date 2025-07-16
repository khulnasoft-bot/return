use iced::{Element, widget::{column, row, text, container, Rule}, Length, Color};
use log::info;

/// Represents a single command entry in the terminal display.
#[derive(Debug, Clone)]
pub struct CommandEntry {
    pub id: usize,
    pub command: String,
    pub status: CommandStatus,
}

/// Represents the status of a command.
#[derive(Debug, Clone, PartialEq)]
pub enum CommandStatus {
    Success,
    Failed,
    Running,
    Pending,
}

/// Messages that can be sent to the `TerminalCommandDisplay` for updates.
#[derive(Debug, Clone)]
pub enum Message {
    // No specific messages for this display-only component for now,
    // but could be extended for interaction (e.g., re-running a command).
}

/// Manages and displays a list of terminal commands.
#[derive(Debug, Clone)]
pub struct TerminalCommandDisplay {
    commands: Vec<CommandEntry>,
    next_id: usize,
}

impl TerminalCommandDisplay {
    /// Creates a new `TerminalCommandDisplay` instance.
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            next_id: 0,
        }
    }

    /// Adds a new command to the display.
    pub fn add_command(&mut self, command: String, status: CommandStatus) {
        self.commands.push(CommandEntry {
            id: self.next_id,
            command,
            status,
        });
        self.next_id += 1;
    }

    /// Updates the status of an existing command.
    pub fn update_command_status(&mut self, id: usize, status: CommandStatus) {
        if let Some(entry) = self.commands.iter_mut().find(|c| c.id == id) {
            entry.status = status;
        }
    }

    /// Renders the `TerminalCommandDisplay` widget.
    pub fn view(&self) -> Element<Message> {
        let command_elements: Vec<Element<Message>> = self.commands.iter().map(|entry| {
            let parts = self.highlight_command(&entry.command);
            let command_text = row![].spacing(0); // Use a row to hold spans if needed

            let mut command_row = row![].spacing(0);
            for (text_part, color) in parts {
                command_row = command_row.push(text(text_part).style(color));
            }

            let status_indicator = match entry.status {
                CommandStatus::Success => text("✓").style(Color::from_rgb(0.0, 0.8, 0.0)), // Green check
                CommandStatus::Failed => text("✗").style(Color::from_rgb(0.8, 0.0, 0.0)), // Red cross
                CommandStatus::Running => text("…").style(Color::from_rgb(0.8, 0.6, 0.0)), // Orange ellipsis
                CommandStatus::Pending => text("•").style(Color::from_rgb(0.5, 0.5, 0.5)), // Gray dot
            };

            container(
                row![
                    status_indicator,
                    command_row.width(Length::Fill)
                ]
                .spacing(8)
            )
            .padding([4, 8])
            .into()
        }).collect();

        column(command_elements)
            .width(Length::Fill)
            .height(Length::Fill)
            .spacing(2)
            .into()
    }

    /// Applies basic syntax highlighting to a command string.
    /// Returns a vector of (text_part, color) tuples.
    fn highlight_command(&self, command: &str) -> Vec<(String, Color)> {
        let mut parts = Vec::new();
        let mut current_part = String::new();
        let mut in_string = false;
        let mut in_highlight = false; // For the specific "|| git checkout -b" highlight

        let keywords = ["git", "ls", "cd", "npm", "cargo", "docker", "ssh", "curl", "echo"];
        let operators = ["&&", "||", ";", "|", ">", ">>", "<"];
        let flags = ["-m", "-i", "--force-with-lease", "-b"];

        let words: Vec<&str> = command.split_whitespace().collect();
        let mut i = 0;

        while i < words.len() {
            let word = words[i];
            let mut word_color = Color::WHITE; // Default color

            // Check for specific highlight (hardcoded for the example)
            if word == "||" && i + 3 < words.len() && words[i+1] == "git" && words[i+2] == "checkout" && words[i+3] == "-b" {
                parts.push((format!("{} ", word), Color::from_rgb(0.2, 0.6, 0.8))); // Highlight background
                parts.push((format!("{} ", words[i+1]), Color::from_rgb(0.0, 0.8, 0.0))); // Green git
                parts.push((format!("{} ", words[i+2]), Color::WHITE));
                parts.push((format!("{} ", words[i+3]), Color::WHITE));
                i += 4;
                continue;
            }

            if keywords.contains(&word) {
                word_color = Color::from_rgb(0.0, 0.8, 0.0); // Green for keywords
            } else if operators.contains(&word) {
                word_color = Color::from_rgb(0.8, 0.6, 0.0); // Orange for operators
            } else if flags.contains(&word) {
                word_color = Color::from_rgb(0.0, 0.6, 0.8); // Cyan for flags
            } else if word.starts_with('"') || word.starts_with('\'') {
                in_string = true;
                word_color = Color::from_rgb(0.8, 0.4, 0.2); // Orange for strings
            } else if in_string && (word.ends_with('"') || word.ends_with('\'')) {
                in_string = false;
                word_color = Color::from_rgb(0.8, 0.4, 0.2); // Orange for strings
            } else if in_string {
                word_color = Color::from_rgb(0.8, 0.4, 0.2); // Orange for strings
            }

            parts.push((format!("{} ", word), word_color));
            i += 1;
        }

        // Remove trailing space from the last part
        if let Some((last_text, last_color)) = parts.last_mut() {
            if last_text.ends_with(' ') {
                last_text.pop();
            }
        }

        parts
    }
}

pub fn init() {
    info!("terminal_command_display module loaded");
}
