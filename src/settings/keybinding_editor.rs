use iced::{
    widget::{column, row, text, button, text_input, scrollable},
    Element, Length, Color, alignment,
};
use std::collections::HashMap;
use crate::config::preferences::{KeyBindings, KeyBinding, Modifier, Action};
use log::info;

#[derive(Debug, Clone)]
pub enum KeybindingEditorMessage {
    AddBinding,
    RemoveBinding(String), // Keybinding ID
    KeyInputChanged(String, String), // (Keybinding ID, new key input)
    ModifierToggled(String, Modifier), // (Keybinding ID, Modifier)
    ActionSelected(String, Action), // (Keybinding ID, Action)
    WhenInputChanged(String, String), // (Keybinding ID, new when input)
    Save,
    Cancel,
}

#[derive(Debug, Clone)]
pub struct KeybindingEditor {
    keybindings: KeyBindings,
    // Temporary state for new/edited binding
    new_binding_id: String,
    new_binding_key: String,
    new_binding_modifiers: Vec<Modifier>,
    new_binding_action: Action,
    new_binding_when: String,
}

impl KeybindingEditor {
    pub fn new(keybindings: KeyBindings) -> Self {
        Self {
            keybindings,
            new_binding_id: String::new(),
            new_binding_key: String::new(),
            new_binding_modifiers: Vec::new(),
            new_binding_action: Action::Command("".to_string()), // Default action
            new_binding_when: String::new(),
        }
    }

    pub fn update(&mut self, message: KeybindingEditorMessage) {
        match message {
            KeybindingEditorMessage::AddBinding => {
                let id = format!("custom_binding_{}", self.keybindings.bindings.len());
                let new_binding = KeyBinding {
                    key: self.new_binding_key.clone(),
                    modifiers: self.new_binding_modifiers.clone(),
                    action: self.new_binding_action.clone(),
                    when: if self.new_binding_when.is_empty() { None } else { Some(self.new_binding_when.clone()) },
                };
                self.keybindings.bindings.insert(id.clone(), new_binding);
                info!("Added new keybinding: {}", id);
                // Reset new binding fields
                self.new_binding_id.clear();
                self.new_binding_key.clear();
                self.new_binding_modifiers.clear();
                self.new_binding_action = Action::Command("".to_string());
                self.new_binding_when.clear();
            }
            KeybindingEditorMessage::RemoveBinding(id) => {
                self.keybindings.bindings.remove(&id);
                info!("Removed keybinding: {}", id);
            }
            KeybindingEditorMessage::KeyInputChanged(id, new_key) => {
                if id == self.new_binding_id {
                    self.new_binding_key = new_key;
                } else if let Some(binding) = self.keybindings.bindings.get_mut(&id) {
                    binding.key = new_key;
                }
            }
            KeybindingEditorMessage::ModifierToggled(id, modifier) => {
                let modifiers_list = if id == self.new_binding_id {
                    &mut self.new_binding_modifiers
                } else if let Some(binding) = self.keybindings.bindings.get_mut(&id) {
                    &mut binding.modifiers
                } else {
                    return;
                };

                if modifiers_list.contains(&modifier) {
                    modifiers_list.retain(|&m| m != modifier);
                } else {
                    modifiers_list.push(modifier);
                }
            }
            KeybindingEditorMessage::ActionSelected(id, action) => {
                if id == self.new_binding_id {
                    self.new_binding_action = action;
                } else if let Some(binding) = self.keybindings.bindings.get_mut(&id) {
                    binding.action = action;
                }
            }
            KeybindingEditorMessage::WhenInputChanged(id, new_when) => {
                if id == self.new_binding_id {
                    self.new_binding_when = new_when;
                } else if let Some(binding) = self.keybindings.bindings.get_mut(&id) {
                    binding.when = if new_when.is_empty() { None } else { Some(new_when) };
                }
            }
            KeybindingEditorMessage::Save => {
                info!("Keybindings saved (mock).");
                // In a real app, you'd persist self.keybindings
            }
            KeybindingEditorMessage::Cancel => {
                info!("Keybinding changes cancelled (mock).");
                // In a real app, you'd revert changes or reload original
            }
        }
    }

    pub fn view(&self) -> Element<KeybindingEditorMessage> {
        let header = text("Keybinding Editor").size(24).color(Color::BLACK);

        let new_binding_section = column![
            text("Add New Keybinding").size(18),
            row![
                text_input("Key (e.g., 'a', 'Enter', 'F1')", &self.new_binding_key)
                    .on_input(|s| KeybindingEditorMessage::KeyInputChanged(self.new_binding_id.clone(), s))
                    .width(Length::FillPortion(0.3)),
                row![
                    Self::modifier_button(Modifier::Ctrl, &self.new_binding_modifiers, self.new_binding_id.clone()),
                    Self::modifier_button(Modifier::Alt, &self.new_binding_modifiers, self.new_binding_id.clone()),
                    Self::modifier_button(Modifier::Shift, &self.new_binding_modifiers, self.new_binding_id.clone()),
                    Self::modifier_button(Modifier::Super, &self.new_binding_modifiers, self.new_binding_id.clone()),
                ].spacing(5).width(Length::FillPortion(0.4)),
                Self::action_dropdown(&self.new_binding_action, self.new_binding_id.clone()),
                text_input("When (optional context)", &self.new_binding_when)
                    .on_input(|s| KeybindingEditorMessage::WhenInputChanged(self.new_binding_id.clone(), s))
                    .width(Length::FillPortion(0.2)),
                button(text("Add"))
                    .on_press(KeybindingEditorMessage::AddBinding)
                    .padding(8)
            ].spacing(10).align_items(alignment::Horizontal::Center),
        ].spacing(10);

        let existing_bindings: Vec<Element<KeybindingEditorMessage>> = self.keybindings.bindings.iter()
            .map(|(id, binding)| {
                row![
                    text_input("", &binding.key)
                        .on_input(|s| KeybindingEditorMessage::KeyInputChanged(id.clone(), s))
                        .width(Length::FillPortion(0.2)),
                    row![
                        Self::modifier_button(Modifier::Ctrl, &binding.modifiers, id.clone()),
                        Self::modifier_button(Modifier::Alt, &binding.modifiers, id.clone()),
                        Self::modifier_button(Modifier::Shift, &binding.modifiers, id.clone()),
                        Self::modifier_button(Modifier::Super, &binding.modifiers, id.clone()),
                    ].spacing(5).width(Length::FillPortion(0.3)),
                    Self::action_dropdown(&binding.action, id.clone()),
                    text_input("", binding.when.as_deref().unwrap_or(""))
                        .on_input(|s| KeybindingEditorMessage::WhenInputChanged(id.clone(), s))
                        .width(Length::FillPortion(0.2)),
                    button(text("Remove"))
                        .on_press(KeybindingEditorMessage::RemoveBinding(id.clone()))
                        .padding(5)
                ].spacing(10).align_items(alignment::Horizontal::Center).into()
            })
            .collect();

        let controls = row![
            button(text("Save")).on_press(KeybindingEditorMessage::Save).padding(10),
            button(text("Cancel")).on_press(KeybindingEditorMessage::Cancel).padding(10),
        ].spacing(10);

        column![
            header,
            new_binding_section,
            scrollable(column(existing_bindings).spacing(5)).height(Length::Fill),
            controls,
        ]
        .spacing(20)
        .padding(20)
        .into()
    }

    fn modifier_button(modifier: Modifier, active_modifiers: &[Modifier], id: String) -> Element<'static, KeybindingEditorMessage> {
        let is_active = active_modifiers.contains(&modifier);
        let text_str = format!("{:?}", modifier);
        button(text(text_str).size(14))
            .on_press(KeybindingEditorMessage::ModifierToggled(id, modifier))
            .style(if is_active {
                iced::widget::button::primary::Style::Primary
            } else {
                iced::widget::button::text::Style::Text
            })
            .padding(5)
            .into()
    }

    fn action_dropdown(current_action: &Action, id: String) -> Element<'static, KeybindingEditorMessage> {
        let actions = vec![
            Action::NewTab, Action::CloseTab, Action::NextTab, Action::PreviousTab,
            Action::SplitHorizontal, Action::SplitVertical, Action::CloseSplit,
            Action::Copy, Action::Paste, Action::Cut, Action::SelectAll,
            Action::Find, Action::FindNext, Action::FindPrevious,
            Action::ScrollUp, Action::ScrollDown, Action::ScrollToTop, Action::ScrollToBottom,
            Action::ToggleFullscreen, Action::ToggleSettings, Action::Quit,
            Action::Command("".to_string()), // Placeholder for custom command
        ];

        let current_action_text = match current_action {
            Action::Command(cmd) => format!("Command: {}", cmd),
            _ => format!("{:?}", current_action),
        };

        iced::widget::pick_list(
            actions,
            Some(current_action.clone()),
            move |action| KeybindingEditorMessage::ActionSelected(id.clone(), action),
        )
        .placeholder(&current_action_text)
        .width(Length::FillPortion(0.3))
        .into()
    }
}

pub fn init() {
    info!("Keybinding editor module loaded");
}
