//! This module handles user input, including text input, command history,
//! and intelligent suggestions for commands and files.
//! It provides an `EnhancedTextInput` widget for the Iced GUI.

use iced::{Element, widget::{text_input, column, row, container, button, text, pick_list}, Length};
use iced::keyboard::{self, KeyCode, Modifiers};
use iced::{keyboard::Event as KeyEvent, Event as IcedEvent};
use std::collections::{VecDeque, HashMap};
use anyhow::Result;
use log::info;

/// Represents the state and logic for an enhanced text input field.
#[derive(Debug, Clone)]
pub struct EnhancedTextInput {
    value: String,
    suggestions: Vec<Suggestion>,
    active_suggestion: Option<usize>,
    history: VecDeque<String>,
    history_index: Option<usize>,
    live_preview: String,
    // New fields for AI model selection
    available_ai_models: Vec<String>,
    selected_ai_model: Option<String>,
}

/// Represents a single suggestion for the input field.
#[derive(Debug, Clone)]
pub struct Suggestion {
    pub text: String,
    pub description: Option<String>,
    pub suggestion_type: SuggestionType,
    pub score: f30,
}

/// Defines the type of a suggestion.
#[derive(Debug, Clone, PartialEq)]
pub enum SuggestionType {
    Command,
    File,
    Directory,
    Flag,
    History,
    Alias,
}

/// Direction for navigating suggestions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
}

/// Direction for navigating command history.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryDirection {
    Up,
    Down,
}

/// Messages that can be sent to the `EnhancedTextInput` for updates.
#[derive(Debug, Clone)]
pub enum Message {
    InputChanged(String),
    Submit,
    SuggestionSelected(usize),
    NavigateSuggestions(Direction),
    ApplySuggestion,
    HistoryNavigated(HistoryDirection),
    // New messages for AI model selection and icon clicks
    AiModelSelected(String),
    ToggleAiMode,
    ToggleLightbulb,
    ToggleMicrophone,
    ToggleAtSymbol,
    ToggleImage,
}

impl EnhancedTextInput {
    /// Creates a new `EnhancedTextInput` instance.
    pub fn new() -> Self {
        Self {
            value: String::new(),
            suggestions: Vec::new(),
            active_suggestion: None,
            history: VecDeque::new(),
            history_index: None,
            live_preview: String::new(),
            available_ai_models: vec![
                "claude 4 sonnet".to_string(),
                "gpt-4o".to_string(),
                "llama3".to_string(),
            ],
            selected_ai_model: Some("claude 4 sonnet".to_string()),
        }
    }

    /// Updates the state of the `EnhancedTextInput` based on incoming messages.
    ///
    /// # Arguments
    ///
    /// * `message` - The `Message` to process.
    pub fn update(&mut self, message: Message) {
        match message {
            Message::InputChanged(value) => {
                self.value = value;
                self.update_suggestions();
                // Auto-select the first suggestion and update live preview
                self.active_suggestion = self.suggestions.first().map(|_| 0);
                self.update_live_preview();
            }
            Message::Submit => {
                self.add_to_history(self.value.clone());
                self.value.clear();
                self.suggestions.clear();
                self.active_suggestion = None;
                self.live_preview.clear();
            }
            Message::SuggestionSelected(index) => {
                if let Some(suggestion) = self.suggestions.get(index) {
                    self.value = suggestion.text.clone();
                    self.suggestions.clear();
                    self.active_suggestion = None;
                    self.live_preview.clear();
                }
            }
            Message::NavigateSuggestions(direction) => {
                if self.suggestions.is_empty() {
                    return;
                }
                let new_index = match self.active_suggestion {
                    Some(i) => match direction {
                        Direction::Up => i.checked_sub(1).unwrap_or(self.suggestions.len() - 1),
                        Direction::Down => (i + 1) % self.suggestions.len(),
                    },
                    None => match direction {
                        Direction::Up => self.suggestions.len() - 1,
                        Direction::Down => 0,
                    },
                };
                self.active_suggestion = Some(new_index);
                self.update_live_preview();
            }
            Message::ApplySuggestion => {
                if let Some(index) = self.active_suggestion {
                    if let Some(suggestion) = self.suggestions.get(index) {
                        self.value = suggestion.text.clone();
                        self.suggestions.clear();
                        self.active_suggestion = None;
                        self.live_preview.clear();
                    }
                }
            }
            Message::HistoryNavigated(direction) => {
                if let Some(cmd) = self.navigate_history(direction) {
                    self.value = cmd;
                    self.suggestions.clear();
                    self.active_suggestion = None;
                    self.live_preview.clear();
                }
            }
            Message::AiModelSelected(model) => {
                self.selected_ai_model = Some(model);
                info!("AI Model selected: {:?}", self.selected_ai_model);
            }
            Message::ToggleAiMode => {
                info!("Toggle AI Mode clicked!");
                // Implement AI mode toggle logic here
            }
            Message::ToggleLightbulb => {
                info!("Toggle Lightbulb clicked!");
                // Implement lightbulb (suggestions/hints) toggle logic here
            }
            Message::ToggleMicrophone => {
                info!("Toggle Microphone clicked!");
                // Implement microphone (voice input) toggle logic here
            }
            Message::ToggleAtSymbol => {
                info!("Toggle At Symbol clicked!");
                // Implement @ (mentions/context) toggle logic here
            }
            Message::ToggleImage => {
                info!("Toggle Image clicked!");
                // Implement image (insert image) toggle logic here
            }
        }
    }

    /// Returns the current value of the text input.
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Adds a command to the history.
    ///
    /// Commands are added to the front of the history and duplicates are avoided.
    /// The history is capped at 1000 entries.
    ///
    /// # Arguments
    ///
    /// * `command` - The command string to add to history.
    pub fn add_to_history(&mut self, command: String) {
        if !command.trim().is_empty() && self.history.front() != Some(&command) {
            self.history.push_front(command);
            if self.history.len() > 1000 {
                self.history.pop_back();
            }
        }
        self.history_index = None;
    }

    /// Navigates through the command history.
    ///
    /// # Arguments
    ///
    /// * `direction` - The direction to navigate (`Up` for older, `Down` for newer).
    ///
    /// # Returns
    ///
    /// An `Option<String>` containing the command from history, or `None` if navigation is not possible.
    fn navigate_history(&mut self, direction: HistoryDirection) -> Option<String> {
        match direction {
            HistoryDirection::Up => {
                let new_index = match self.history_index {
                    None => Some(0),
                    Some(i) if i < self.history.len() - 1 => Some(i + 1),
                    Some(i) => Some(i),
                };
                
                if let Some(index) = new_index {
                    self.history_index = new_index;
                    self.history.get(index).cloned()
                } else {
                    None
                }
            }
            HistoryDirection::Down => {
                match self.history_index {
                    Some(0) => {
                        self.history_index = None;
                        Some(String::new())
                    }
                    Some(i) => {
                        self.history_index = Some(i - 1);
                        self.history.get(i - 1).cloned()
                    }
                    None => None,
                }
            }
        }
    }

    /// Updates the list of suggestions based on the current input value.
    ///
    /// This method generates command and history suggestions and sorts them by relevance.
    fn update_suggestions(&mut self) {
        let mut suggestions = Vec::new();
        let current_input = self.value.trim();

        if current_input.is_empty() {
            suggestions.extend(self.get_command_suggestions(""));
        } else {
            let last_word = current_input.split_whitespace().last().unwrap_or("");

            if current_input.split_whitespace().count() <= 1 {
                suggestions.extend(self.get_command_suggestions(last_word));
            }
            
            suggestions.extend(self.get_history_suggestions(current_input));
        }

        // Sort suggestions by score in descending order
        suggestions.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        suggestions.truncate(10); // Limit to top 10 suggestions

        self.suggestions = suggestions;
    }

    /// Updates the live preview text based on the active suggestion.
    fn update_live_preview(&mut self) {
        self.live_preview = if let Some(index) = self.active_suggestion {
            self.suggestions.get(index).map(|s| s.text.clone()).unwrap_or_default()
        } else {
            String::new()
        };
    }

    /// Generates command suggestions based on a given prefix.
    ///
    /// # Arguments
    ///
    /// * `prefix` - The input prefix to filter commands by.
    ///
    /// # Returns
    ///
    /// A `Vec<Suggestion>` containing matching command suggestions.
    fn get_command_suggestions(&self, prefix: &str) -> Vec<Suggestion> {
        let common_commands = [
            "ls", "cd", "pwd", "mkdir", "rmdir", "rm", "cp", "mv", "cat", "less", "more",
            "grep", "find", "which", "whereis", "man", "info", "help", "history",
            "ps", "top", "htop", "kill", "killall", "jobs", "bg", "fg", "nohup",
            "git", "npm", "yarn", "cargo", "docker", "kubectl", "ssh", "scp", "rsync",
            "echo", "ping", "curl", "wget", "chmod", "chown", "df", "du", "tar", "zip", "unzip",
        ];

        common_commands
            .iter()
            .filter(|cmd| cmd.starts_with(prefix))
            .map(|cmd| Suggestion {
                text: cmd.to_string(),
                description: Some(self.get_command_description(cmd)),
                suggestion_type: SuggestionType::Command,
                score: self.calculate_fuzzy_score(cmd, prefix),
            })
            .collect()
    }

    /// Generates history suggestions based on a given prefix.
    ///
    /// # Arguments
    ///
    /// * `prefix` - The input prefix to filter history entries by.
    ///
    /// # Returns
    ///
    /// A `Vec<Suggestion>` containing matching history suggestions.
    fn get_history_suggestions(&self, prefix: &str) -> Vec<Suggestion> {
        self.history
            .iter()
            .filter(|cmd| cmd.contains(prefix) && cmd != &self.value)
            .take(5) // Limit history suggestions
            .map(|cmd| Suggestion {
                text: cmd.clone(),
                description: Some("From history".to_string()),
                suggestion_type: SuggestionType::History,
                score: self.calculate_fuzzy_score(cmd, prefix) * 0.9, // Slightly lower score for history
            })
            .collect()
    }

    /// Provides a brief description for common commands.
    ///
    /// # Arguments
    ///
    /// * `command` - The command string.
    ///
    /// # Returns
    ///
    /// A `String` containing the description.
    fn get_command_description(&self, command: &str) -> String {
        match command {
            "ls" => "List directory contents".to_string(),
            "cd" => "Change directory".to_string(),
            "pwd" => "Print working directory".to_string(),
            "git" => "Git version control".to_string(),
            "npm" => "Node package manager".to_string(),
            "cargo" => "Rust package manager".to_string(),
            "docker" => "Container management".to_string(),
            "echo" => "Display a line of text".to_string(),
            "ping" => "Send ICMP ECHO_REQUEST packets to network hosts".to_string(),
            "curl" => "Transfer data from or to a server".to_string(),
            _ => format!("Execute {}", command),
        }
    }

    /// Calculates a fuzzy match score between text and a query.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to score against.
    /// * `query` - The query string.
    ///
    /// # Returns
    ///
    /// A `f32` representing the fuzzy match score. Higher is better.
    fn calculate_fuzzy_score(&self, text: &str, query: &str) -> f32 {
        if query.is_empty() {
            return 0.0;
        }
        let text_lower = text.to_lowercase();
        let query_lower = query.to_lowercase();

        if text_lower.starts_with(&query_lower) {
            1.0
        } else if text_lower.contains(&query_lower) {
            0.7
        } else {
            let mut score = 0.0;
            let mut query_chars = query_lower.chars().peekable();
            
            for (i, ch) in text_lower.chars().enumerate() {
                if let Some(&query_ch) = query_chars.peek() {
                    if ch == query_ch {
                        score += 0.1;
                        if i < 5 { score += 0.05; } // Bonus for early matches
                        query_chars.next();
                    }
                }
            }
            
            score
        }
    }

    /// Renders the `EnhancedTextInput` widget.
    ///
    /// # Arguments
    ///
    /// * `prompt_indicator` - The prompt string to display before the input field (e.g., "$ ").
    /// * `placeholder` - The default placeholder text for the input field.
    ///
    /// # Returns
    ///
    /// An `Element<Message>` representing the rendered input field and suggestions.
    pub fn view(&self, prompt_indicator: &str, placeholder: &str) -> Element<Message> {
        // Determine the placeholder text, prioritizing live preview
        let current_placeholder = if !self.live_preview.is_empty() && self.value.is_empty() {
            &self.live_preview
        } else if !self.live_preview.is_empty() && self.live_preview.starts_with(&self.value) {
            &self.live_preview[self.value.len()..]
        }
        else {
            placeholder
        };

        // Create the main text input widget
        let input = text_input(current_placeholder, &self.value)
            .on_input(Message::InputChanged)
            .on_submit(Message::Submit)
            .padding(12)
            .size(16);

        // Combine prompt indicator and input field
        let input_with_prompt = row![
            text(prompt_indicator).size(16),
            input
        ].spacing(8)
        .width(Length::Fill);

        // Render suggestions if available
        let suggestions_view = if !self.suggestions.is_empty() {
            let suggestion_elements: Vec<Element<Message>> = self.suggestions
                .iter()
                .enumerate()
                .map(|(i, suggestion)| {
                    let is_active = self.active_suggestion == Some(i);
                    
                    container(
                        row![
                            text(&suggestion.text).size(14),
                            if let Some(desc) = &suggestion.description {
                                text(desc)
                                    .size(12)
                                    .style(|theme| iced::widget::text::Appearance {
                                        color: Some(theme.palette().text.scale_alpha(0.7)),
                                    })
                            } else {
                                text("")
                            }
                        ]
                        .spacing(8)
                    )
                    .padding(8)
                    .style(move |theme| {
                        if is_active {
                            container::Appearance {
                                background: Some(theme.palette().primary.scale_alpha(0.1).into()),
                                ..Default::default()
                            }
                        } else {
                            container::Appearance::default()
                        }
                    })
                    .on_press(Message::SuggestionSelected(i))
                    .into()
                })
                .collect();

            container(column(suggestion_elements).spacing(2))
                .padding(4)
                .style(|theme| container::Appearance {
                    background: Some(theme.palette().background.into()),
                    border: iced::Border {
                        color: theme.palette().text.scale_alpha(0.2),
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                })
                .into()
        } else {
            column![].into()
        };

        // AI model dropdown
        let ai_model_dropdown = pick_list(
            self.available_ai_models.clone(),
            self.selected_ai_model.clone(),
            Message::AiModelSelected,
        )
        .padding(8)
        .width(Length::Shrink);

        // Action buttons/icons
        let action_buttons = row![
            button(text("A").size(16)).on_press(Message::ToggleAiMode).padding(8),
            button(text("💡").size(16)).on_press(Message::ToggleLightbulb).padding(8),
            button(text("🎤").size(16)).on_press(Message::ToggleMicrophone).padding(8),
            button(text("@").size(16)).on_press(Message::ToggleAtSymbol).padding(8),
            button(text("🖼️").size(16)).on_press(Message::ToggleImage).padding(8),
            ai_model_dropdown,
        ]
        .spacing(8)
        .align_items(iced::Alignment::Center);

        column![
            input_with_prompt,
            suggestions_view,
            Rule::horizontal(1), // Separator line
            action_buttons
        ]
        .spacing(4)
        .padding(8)
        .into()
    }
}

/// Initializes the input module.
pub fn init() {
    info!("input module loaded");
}
