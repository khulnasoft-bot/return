use iced::{
    widget::{column, row, text, checkbox, slider, pick_list, text_input},
    Element, Length, Color, alignment,
};
use crate::config::preferences::{UiPreferences, InputType, InputPosition};
use log::info;

#[derive(Debug, Clone)]
pub enum AppearanceSettingsMessage {
    SyncWithOsThemeToggled(bool),
    AppIconChanged(String),
    OpenNewWindowsCustomSizeToggled(bool),
    WindowOpacityChanged(f32),
    WindowBlurRadiusChanged(f32),
    InputTypeChanged(InputType),
    InputPositionChanged(InputPosition),
    DimInactivePanesToggled(bool),
    FocusFollowsMouseToggled(bool),
    // Add messages for other UI preferences
}

#[derive(Debug, Clone)]
pub struct AppearanceSettings {
    preferences: UiPreferences,
}

impl AppearanceSettings {
    pub fn new(preferences: UiPreferences) -> Self {
        Self { preferences }
    }

    pub fn update(&mut self, message: AppearanceSettingsMessage) {
        match message {
            AppearanceSettingsMessage::SyncWithOsThemeToggled(value) => {
                self.preferences.sync_with_os_theme = value;
                info!("Sync with OS theme: {}", value);
            }
            AppearanceSettingsMessage::AppIconChanged(value) => {
                self.preferences.app_icon = value;
                info!("App icon changed to: {}", self.preferences.app_icon);
            }
            AppearanceSettingsMessage::OpenNewWindowsCustomSizeToggled(value) => {
                self.preferences.open_new_windows_custom_size = value;
                info!("Open new windows custom size: {}", value);
            }
            AppearanceSettingsMessage::WindowOpacityChanged(value) => {
                self.preferences.window_opacity = value;
                info!("Window opacity: {}", value);
            }
            AppearanceSettingsMessage::WindowBlurRadiusChanged(value) => {
                self.preferences.window_blur_radius = value;
                info!("Window blur radius: {}", value);
            }
            AppearanceSettingsMessage::InputTypeChanged(value) => {
                self.preferences.input_type = value;
                info!("Input type: {:?}", value);
            }
            AppearanceSettingsMessage::InputPositionChanged(value) => {
                self.preferences.input_position = value;
                info!("Input position: {:?}", value);
            }
            AppearanceSettingsMessage::DimInactivePanesToggled(value) => {
                self.preferences.dim_inactive_panes = value;
                info!("Dim inactive panes: {}", value);
            }
            AppearanceSettingsMessage::FocusFollowsMouseToggled(value) => {
                self.preferences.focus_follows_mouse = value;
                info!("Focus follows mouse: {}", value);
            }
        }
    }

    pub fn view(&self) -> Element<AppearanceSettingsMessage> {
        let header = text("Appearance Settings").size(24).color(Color::BLACK);

        let sync_os_theme_toggle = checkbox(
            "Sync with OS Theme",
            self.preferences.sync_with_os_theme,
            AppearanceSettingsMessage::SyncWithOsThemeToggled,
        );

        let app_icon_input = row![
            text("App Icon:").width(Length::Fixed(150.0)),
            text_input("Default, Custom1, etc.", &self.preferences.app_icon)
                .on_input(AppearanceSettingsMessage::AppIconChanged)
                .width(Length::Fill)
        ].spacing(10).align_items(alignment::Horizontal::Center);

        let custom_size_toggle = checkbox(
            "Open New Windows with Custom Size",
            self.preferences.open_new_windows_custom_size,
            AppearanceSettingsMessage::OpenNewWindowsCustomSizeToggled,
        );

        let opacity_slider = row![
            text(format!("Window Opacity: {:.2}", self.preferences.window_opacity)).width(Length::Fixed(150.0)),
            slider(0.0..=1.0, self.preferences.window_opacity, AppearanceSettingsMessage::WindowOpacityChanged)
                .step(0.01)
                .width(Length::Fill)
        ].spacing(10).align_items(alignment::Horizontal::Center);

        let blur_slider = row![
            text(format!("Window Blur Radius: {:.1}", self.preferences.window_blur_radius)).width(Length::Fixed(150.0)),
            slider(0.0..=10.0, self.preferences.window_blur_radius, AppearanceSettingsMessage::WindowBlurRadiusChanged)
                .step(0.1)
                .width(Length::Fill)
        ].spacing(10).align_items(alignment::Horizontal::Center);

        let input_type_picker = row![
            text("Input Type:").width(Length::Fixed(150.0)),
            pick_list(
                &[InputType::Universal, InputType::Classic][..],
                Some(self.preferences.input_type),
                AppearanceSettingsMessage::InputTypeChanged,
            )
            .width(Length::Fill)
        ].spacing(10).align_items(alignment::Horizontal::Center);

        let input_position_picker = row![
            text("Input Position:").width(Length::Fixed(150.0)),
            pick_list(
                &[InputPosition::PinToBottom][..], // Only one option for now
                Some(self.preferences.input_position),
                AppearanceSettingsMessage::InputPositionChanged,
            )
            .width(Length::Fill)
        ].spacing(10).align_items(alignment::Horizontal::Center);

        let dim_inactive_panes_toggle = checkbox(
            "Dim Inactive Panes",
            self.preferences.dim_inactive_panes,
            AppearanceSettingsMessage::DimInactivePanesToggled,
        );

        let focus_follows_mouse_toggle = checkbox(
            "Focus Follows Mouse",
            self.preferences.focus_follows_mouse,
            AppearanceSettingsMessage::FocusFollowsMouseToggled,
        );

        column![
            header,
            sync_os_theme_toggle,
            app_icon_input,
            custom_size_toggle,
            opacity_slider,
            blur_slider,
            input_type_picker,
            input_position_picker,
            dim_inactive_panes_toggle,
            focus_follows_mouse_toggle,
        ]
        .spacing(15)
        .padding(20)
        .into()
    }
}

pub fn init() {
    info!("Appearance settings module loaded");
}
