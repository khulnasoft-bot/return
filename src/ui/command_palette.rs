use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;
use tui_textarea::{TextArea, Input, Key};
use crate::fuzzy_match::{FuzzyMatchManager, FuzzyMatchResult};
use std::collections::HashMap;
use iced::{
    widget::{column, container, row, text, text_input, button, scrollable},
    Element, Length, Color, alignment,
};
use iced::keyboard::{KeyCode, Modifiers};
use crate::main::Message; // Assuming Message is in main.rs
use crate::input::Message as InputMessage; // Assuming InputMessage is in input.rs
use log::info;

#[derive(Debug, Clone)]
pub enum CommandPaletteMessage {
    InputChanged(String),
    Submit,
    SelectCommand(usize),
    Navigate(Direction),
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandAction {
    pub id: String,
    pub name: String,
    pub description: String,
    pub message: Message, // The message to send when this command is executed
}

pub struct CommandPalette {
    input_area: TextArea<'static>,
    is_open: bool,
    commands: Vec<CommandAction>,
    filtered_commands: Vec<CommandAction>,
    selected_index: usize,
    fuzzy_matcher: FuzzyMatchManager,
    input_value: String,
}

impl CommandPalette {
    pub fn new() -> Self {
        let all_commands = Self::get_all_commands();
        Self {
            input_area: TextArea::default(),
            is_open: false,
            commands: all_commands.clone(),
            filtered_commands: all_commands,
            selected_index: 0,
            fuzzy_matcher: FuzzyMatchManager::new(),
            input_value: String::new(),
        }
    }

    fn get_all_commands() -> Vec<CommandAction> {
        vec![
            CommandAction {
                id: "toggle_agent_mode".to_string(),
                name: "Toggle AI Agent Mode".to_string(),
                description: "Activates or deactivates the AI assistant.".to_string(),
                message: Message::ToggleAgentMode,
            },
            CommandAction {
                id: "run_benchmarks".to_string(),
                name: "Run Performance Benchmarks".to_string(),
                description: "Executes a suite of performance tests.".to_string(),
                message: Message::RunBenchmarks,
            },
            CommandAction {
                id: "toggle_settings".to_string(),
                name: "Open Settings".to_string(),
                description: "Opens the application settings panel.".to_string(),
                message: Message::ToggleSettings,
            },
            CommandAction {
                id: "fetch_ai_usage".to_string(),
                name: "Fetch AI Usage Quota".to_string(),
                description: "Displays your current AI API usage quota.".to_string(),
                message: Message::BlockAction("".to_string(), crate::main::BlockMessage::FetchUsageQuota),
            },
            // Add more commands here
        ]
    }

    pub fn init(&self) {
        log::info!("Command palette initialized.");
    }

    pub fn open(&mut self) {
        self.is_open = true;
        self.input_area.set_cursor_line_style(Style::default());
        self.input_area.set_cursor_style(Style::default().bg(Color::White).fg(Color::Black));
        self.update_filtered_commands();
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.input_area.set_lines(vec!["".to_string()]);
        self.selected_index = 0;
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn handle_input(&mut self, input: Input) -> Option<String> {
        match input {
            Input { key: Key::Esc, .. } => {
                self.close();
                None
            },
            Input { key: Key::Enter, .. } => {
                if !self.filtered_commands.is_empty() {
                    let selected_command_id = self.filtered_commands[self.selected_index].id.clone();
                    self.close();
                    Some(selected_command_id)
                } else {
                    None
                }
            },
            Input { key: Key::Up, .. } => {
                if !self.filtered_commands.is_empty() {
                    self.selected_index = self.selected_index.saturating_sub(1);
                }
                None
            },
            Input { key: Key::Down, .. } => {
                if !self.filtered_commands.is_empty() {
                    self.selected_index = (self.selected_index + 1).min(self.filtered_commands.len() - 1);
                }
                None
            },
            _ => {
                self.input_area.input(input);
                self.update_filtered_commands();
                None
            }
        }
    }

    fn update_filtered_commands(&mut self) {
        let query = self.input_area.lines().join("").to_lowercase();
        let candidate_ids: Vec<String> = self.commands.iter().map(|cmd| cmd.id.clone()).collect();
        
        self.filtered_commands = self.fuzzy_matcher.fuzzy_match(&query, &candidate_ids);
        self.selected_index = 0; // Reset selection on filter change
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        if !self.is_open {
            return;
        }

        let popup_area = CommandPalette::centered_rect(60, 40, area); // 60% width, 40% height

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Input area
                Constraint::Min(0),    // Results area
            ])
            .split(popup_area);

        // Input area
        let input_block = Block::default()
            .borders(Borders::ALL)
            .title(Span::styled("Command Palette", Style::default().fg(Color::LightYellow)));
        
        let mut input_widget = self.input_area.widget();
        input_widget = input_widget.block(input_block);
        frame.render_widget(input_widget, chunks[0]);

        // Results area
        let results_block = Block::default()
            .borders(Borders::ALL)
            .title(Span::styled("Results", Style::default().fg(Color::LightGreen)));

        let mut result_lines: Vec<Line> = Vec::new();
        for (i, result) in self.filtered_commands.iter().enumerate() {
            let command_id = &result.id;
            let description = &result.description;
            let line_content = format!("{}: {}", command_id, description);
            
            let mut spans = Vec::new();
            let mut last_idx = 0;
            for &match_idx in &result.indices {
                if match_idx >= line_content.len() { continue; }
                spans.push(Span::raw(&line_content[last_idx..match_idx]));
                spans.push(Span::styled(
                    line_content.chars().nth(match_idx).unwrap().to_string(),
                    Style::default().fg(Color::Cyan).add_modifier(ratatui::style::Modifier::BOLD),
                ));
                last_idx = match_idx + line_content.chars().nth(match_idx).unwrap().len_utf8();
            }
            spans.push(Span::raw(&line_content[last_idx..]));

            let mut line = Line::from(spans);
            if i == self.selected_index {
                line = line.style(Style::default().bg(Color::DarkGray));
            }
            result_lines.push(line);
        }

        let results_paragraph = Paragraph::new(result_lines)
            .block(results_block)
            .wrap(ratatui::widgets::Wrap { trim: false });
        frame.render_widget(results_paragraph, chunks[1]);
    }

    fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }

    pub fn update(&mut self, message: CommandPaletteMessage) -> Option<Message> {
        match message {
            CommandPaletteMessage::InputChanged(value) => {
                self.input_value = value;
                self.filter_commands();
                self.selected_index = self.filtered_commands.first().map(|_| 0).unwrap_or(0);
                None
            }
            CommandPaletteMessage::Submit => {
                if let Some(index) = self.selected_index {
                    if let Some(command) = self.filtered_commands.get(index) {
                        info!("Executing command palette action: {}", command.name);
                        return Some(command.message.clone());
                    }
                }
                None
            }
            CommandPaletteMessage::SelectCommand(index) => {
                self.selected_index = index;
                None
            }
            CommandPaletteMessage::Navigate(direction) => {
                if self.filtered_commands.is_empty() {
                    return None;
                }
                let current_index = self.selected_index;
                let new_index = match direction {
                    Direction::Up => current_index.checked_sub(1).unwrap_or(self.filtered_commands.len() - 1),
                    Direction::Down => (current_index + 1) % self.filtered_commands.len(),
                };
                self.selected_index = new_index;
                None
            }
            CommandPaletteMessage::Close => {
                // This message is typically handled by the parent to close the palette
                self.is_open = false;
                None
            }
        }
    }

    fn filter_commands(&mut self) {
        let query = self.input_value.to_lowercase();
        self.filtered_commands = self.commands.iter()
            .filter(|cmd| {
                cmd.name.to_lowercase().contains(&query) ||
                cmd.description.to_lowercase().contains(&query)
            })
            .cloned()
            .collect();
    }

    pub fn view(&self) -> Element<CommandPaletteMessage> {
        let input = text_input("Search commands...", &self.input_value)
            .on_input(CommandPaletteMessage::InputChanged)
            .on_submit(CommandPaletteMessage::Submit)
            .padding(10)
            .size(18);

        let commands_list: Element<CommandPaletteMessage> = if self.filtered_commands.is_empty() {
            text("No commands found.").size(16).color(Color::from_rgb(0.5, 0.5, 0.5)).into()
        } else {
            scrollable(
                column(
                    self.filtered_commands.iter().enumerate().map(|(i, cmd)| {
                        let is_active = self.selected_index == i;
                        container(
                            column![
                                text(&cmd.name).size(16).style(if is_active { Color::WHITE } else { Color::BLACK }),
                                text(&cmd.description).size(12).style(if is_active { Color::from_rgb(0.8, 0.8, 0.8) } else { Color::from_rgb(0.5, 0.5, 0.5) }),
                            ]
                            .spacing(2)
                        )
                        .width(Length::Fill)
                        .padding(8)
                        .style(move |theme| {
                            if is_active {
                                container::Appearance {
                                    background: Some(iced::Background::Color(theme.palette().primary)),
                                    border_radius: 4.0.into(),
                                    ..Default::default()
                                }
                            } else {
                                container::Appearance {
                                    background: Some(iced::Background::Color(Color::from_rgb8(240, 240, 240))),
                                    border_radius: 4.0.into(),
                                    ..Default::default()
                                }
                            }
                        })
                        .on_press(CommandPaletteMessage::SelectCommand(i))
                        .into()
                    })
                    .collect()
                )
                .spacing(5)
            )
            .height(Length::FillPortion(0.7))
            .into()
        };

        container(
            column![
                input,
                commands_list,
                row![
                    button(text("Close").size(14))
                        .on_press(CommandPaletteMessage::Close)
                        .padding(8)
                ]
                .width(Length::Fill)
                .align_items(alignment::Horizontal::Right)
            ]
            .spacing(10)
        )
        .width(Length::FillPortion(0.6))
        .height(Length::FillPortion(0.7))
        .padding(20)
        .center_x()
        .center_y()
        .style(|theme| container::Appearance {
            background: Some(iced::Background::Color(theme.palette().background)),
            border_radius: 8.0.into(),
            border_width: 1.0,
            border_color: theme.palette().text.scale_alpha(0.3),
            shadow_offset: iced::Vector::new(0.0, 4.0),
            shadow_blur_radius: 8.0,
            ..Default::default()
        })
        .into()
    }
}

pub fn init() {
    info!("Command palette module loaded");
}
