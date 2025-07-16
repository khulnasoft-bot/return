use iced::{
    widget::{column, row, text, checkbox, container, button, horizontal_space},
    Element, Length, Color, alignment,
};
use crate::config::preferences::IndexingPreferences;
use log::info;

#[derive(Debug, Clone, PartialEq)]
pub enum IndexingStatus {
    Discovering(u32), // Number of files to index
    Synced,
    Failed(String), // Error message
    TooLarge(String), // Message about file limit
    Paused,
}

#[derive(Debug, Clone)]
pub struct IndexedFolder {
    pub path: String,
    pub status: IndexingStatus,
}

#[derive(Debug, Clone)]
pub enum IndexingSettingsMessage {
    IndexNewFoldersToggled(bool),
    FolderAction(String, FolderAction), // (folder_path, action)
}

#[derive(Debug, Clone)]
pub enum FolderAction {
    TogglePause,
    Refresh,
    Delete,
}

#[derive(Debug, Clone)]
pub struct IndexingSettings {
    pub preferences: IndexingPreferences,
    pub indexed_folders: Vec<IndexedFolder>,
}

impl IndexingSettings {
    pub fn new(preferences: IndexingPreferences) -> Self {
        // Hardcoded sample data based on the image
        let indexed_folders = vec![
            IndexedFolder {
                path: "~/warp-server".to_string(),
                status: IndexingStatus::Discovering(45),
            },
            IndexedFolder {
                path: "~/warp-internal".to_string(),
                status: IndexingStatus::Synced,
            },
            IndexedFolder {
                path: "~/certs".to_string(),
                status: IndexingStatus::Failed("Sync failed: We couldn't read the .git directory — it may be corrupted or incomplete. Try re-cloning the repo and syncing again.".to_string()),
            },
            IndexedFolder {
                path: "~/warp-terraform".to_string(),
                status: IndexingStatus::TooLarge("This codebase exceeds your plan's 20,000 file limit. Talk to sales to index larger codebases.".to_string()),
            },
        ];

        Self {
            preferences,
            indexed_folders,
        }
    }

    pub fn update(&mut self, message: IndexingSettingsMessage) {
        match message {
            IndexingSettingsMessage::IndexNewFoldersToggled(value) => {
                self.preferences.index_new_folders_by_default = value;
                info!("Index new folders by default: {}", value);
            }
            IndexingSettingsMessage::FolderAction(path, action) => {
                if let Some(folder) = self.indexed_folders.iter_mut().find(|f| f.path == path) {
                    match action {
                        FolderAction::TogglePause => {
                            folder.status = match folder.status {
                                IndexingStatus::Discovering(count) => IndexingStatus::Paused,
                                IndexingStatus::Paused => IndexingStatus::Discovering(0), // Resume, reset count for simplicity
                                _ => folder.status.clone(), // No change for other statuses
                            };
                            info!("Toggled pause for folder: {}", path);
                        }
                        FolderAction::Refresh => {
                            // Simulate refresh: reset to discovering or synced
                            folder.status = IndexingStatus::Discovering(100); // Simulate re-indexing
                            info!("Refreshed folder: {}", path);
                        }
                        FolderAction::Delete => {
                            self.indexed_folders.retain(|f| f.path != path);
                            info!("Deleted folder: {}", path);
                        }
                    }
                }
            }
        }
    }

    pub fn view(&self) -> Element<IndexingSettingsMessage> {
        let header = text("Codebase indexing")
            .size(24)
            .color(Color::WHITE);

        let description = text(
            "Warp can automatically index code repositories as you navigate them, helping agents quickly understand context and provide solutions. Code is never stored on the server. If a codebase is unable to be indexed, Warp can still navigate your codebase and gain insights via grep and find tool calling."
        )
        .size(14)
        .color(Color::from_rgb(0.7, 0.7, 0.7)); // Lighter grey for description

        let toggle_section = column![
            checkbox(
                "Index new folders by default",
                self.preferences.index_new_folders_by_default,
                IndexingSettingsMessage::IndexNewFoldersToggled,
            )
            .text_color(Color::WHITE),
            text("When set to true, Warp will automatically index code repositories as you navigate them - helping agents quickly understand context and provide targeted solutions.")
                .size(12)
                .color(Color::from_rgb(0.6, 0.6, 0.6)),
        ]
        .spacing(5);

        let folder_list: Element<IndexingSettingsMessage> = self.indexed_folders.iter().fold(
            column![],
            |col, folder| {
                col.push(self.folder_item_view(folder))
            }
        )
        .spacing(10)
        .into();

        column![
            header,
            description,
            horizontal_space(Length::Fill), // Spacer
            toggle_section,
            horizontal_space(Length::Fill), // Spacer
            folder_list,
        ]
        .spacing(20)
        .padding(20)
        .into()
    }

    fn folder_item_view(&self, folder: &IndexedFolder) -> Element<IndexingSettingsMessage> {
        let path_text = text(&folder.path).size(16).color(Color::WHITE);

        let (status_icon, status_text, status_color, extra_message) = match &folder.status {
            IndexingStatus::Discovering(count) => ("⏸️", format!("Discovering files to index - {}", count), Color::WHITE, None),
            IndexingStatus::Synced => ("✅", "Synced".to_string(), Color::from_rgb(0.2, 0.8, 0.2), None), // Green
            IndexingStatus::Failed(msg) => ("⚠️", "Failed".to_string(), Color::from_rgb(0.8, 0.2, 0.2), Some(msg.clone())), // Red
            IndexingStatus::TooLarge(msg) => ("⚠️", "Codebase too large".to_string(), Color::from_rgb(0.8, 0.5, 0.0), Some(msg.clone())), // Orange
            IndexingStatus::Paused => ("▶️", "Paused".to_string(), Color::WHITE, None),
        };

        let status_line = row![
            text(status_icon).size(14).color(status_color),
            text(status_text).size(14).color(status_color),
        ].spacing(5).align_items(alignment::Vertical::Center);

        let mut actions = row![];
        match folder.status {
            IndexingStatus::Discovering(_) | IndexingStatus::Paused => {
                actions = actions.push(
                    button(text(if matches!(folder.status, IndexingStatus::Paused) { "▶️" } else { "⏸️" }))
                        .on_press(IndexingSettingsMessage::FolderAction(folder.path.clone(), FolderAction::TogglePause))
                        .style(iced::widget::button::text::Style::Text)
                        .padding(5)
                );
            },
            _ => {
                actions = actions.push(
                    button(text("🔄"))
                        .on_press(IndexingSettingsMessage::FolderAction(folder.path.clone(), FolderAction::Refresh))
                        .style(iced::widget::button::text::Style::Text)
                        .padding(5)
                );
            }
        }
        actions = actions.push(
            button(text("🗑️"))
                .on_press(IndexingSettingsMessage::FolderAction(folder.path.clone(), FolderAction::Delete))
                .style(iced::widget::button::text::Style::Text)
                .padding(5)
        );

        let mut content = column![
            path_text,
            status_line,
        ].spacing(2);

        if let Some(msg) = extra_message {
            content = content.push(
                text(msg).size(12).color(Color::from_rgb(0.6, 0.6, 0.6))
            );
        }

        container(
            row![
                content.width(Length::Fill),
                actions.spacing(5).align_items(alignment::Vertical::Center),
            ]
            .align_items(alignment::Vertical::Center)
            .spacing(10)
        )
        .padding(10)
        .style(iced::widget::container::Appearance {
            background: Some(iced::Background::Color(Color::from_rgb(0.15, 0.15, 0.15))), // Darker background for items
            border_radius: 5.0,
            border_width: 0.0, // No border for items
            border_color: Color::TRANSPARENT,
            ..Default::default()
        })
        .into()
    }
}

pub fn init() {
    info!("Indexing settings module loaded");
}
