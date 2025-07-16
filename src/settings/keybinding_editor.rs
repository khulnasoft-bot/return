use iced::{
    widget::{column, row, text, button, text_input, scrollable},
    Element, Length, Color, alignment, Command,
};
use std::collections::HashMap;
use crate::config::preferences::KeybindingPreferences;
use log::info;

#[derive(Debug, Clone)]
pub enum KeybindingEditorMessage {
    BindingChanged(String, String), // (action, new_key_combination)
    AddBinding,
    RemoveBinding(String), // (action)
    Save,
    Cancel,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct KeybindingEditor {
    pub keybindings: KeybindingPreferences,
    original_keybindings: KeybindingPreferences, // To allow reverting changes
    error_message: Option<String>,
}

impl KeybindingEditor {
    pub fn new(keybindings: KeybindingPreferences) -> Self {
        Self {
            keybindings: keybindings.clone(),
            original_keybindings: keybindings,
            error_message: None,
        }
    }

    pub fn update(&mut self, message: KeybindingEditorMessage) -> Command<KeybindingEditorMessage> {
        self.error_message = None; // Clear previous errors
        match message {
            KeybindingEditorMessage::BindingChanged(action, new_key_combination) => {
                self.keybindings.bindings.insert(action, new_key_combination);
            }
            KeybindingEditorMessage::AddBinding => {
                let new_action_key = format!("new_action_{}", uuid::Uuid::new_v4().to_string()[..4].to_string());
                self.keybindings.bindings.insert(new_action_key, "".to_string());
            }
            KeybindingEditorMessage::RemoveBinding(action) => {
                self.keybindings.bindings.remove(&action);
            }
            KeybindingEditorMessage::Save => {
                self.original_keybindings = self.keybindings.clone();
                info!("Keybindings saved (mock). In a real app, this would persist.");
                // In a real app, you'd trigger a save via ConfigManager here
            }
            KeybindingEditorMessage::Cancel => {
                self.keybindings = self.original_keybindings.clone();
                info!("Keybinding changes cancelled.");
            }
            KeybindingEditorMessage::Error(msg) => {
                self.error_message = Some(msg);
            }
        }
        Command::none()
    }

    pub fn view(&self) -> Element<KeybindingEditorMessage> {
        let header = text("Keybinding Editor").size(24).color(Color::BLACK);

        let keybinding_fields: Vec<Element<KeybindingEditorMessage>> = self.keybindings.bindings.iter()
            .map(|(action, key_combination)| {
                row![
                    text(action).width(Length::Fixed(150.0)),
                    text_input("Key Combination", key_combination)
                        .on_input(move |s| KeybindingEditorMessage::BindingChanged(action.clone(), s))
                        .width(Length::Fill),
                    button(text("X"))
                        .on_press(KeybindingEditorMessage::RemoveBinding(action.clone()))
                        .padding(5)
                ].spacing(10).align_items(alignment::Horizontal::Center).into()
            })
            .collect();

        let error_display = if let Some(msg) = &self.error_message {
            text(msg).color(Color::RED).size(14).into()
        } else {
            column![].into()
        };

        let controls = row![
            button(text("Add New Binding")).on_press(KeybindingEditorMessage::AddBinding).padding(10),
            button(text("Save")).on_press(KeybindingEditorMessage::Save).padding(10),
            button(text("Cancel")).on_press(KeybindingEditorMessage::Cancel).padding(10),
        ].spacing(10);

        scrollable(
            column![
                header,
                column(keybinding_fields).spacing(5),
                error_display,
                controls,
            ]
            .spacing(20)
            .padding(20)
        )
        .into()
    }
}

pub fn init() {
    info!("Keybinding editor module loaded");
}
