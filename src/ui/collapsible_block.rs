use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;
use iced::{
    widget::{column, container, row, text, button},
    Element, Length, alignment,
};
use log::info;

#[derive(Debug, Clone)]
pub enum CollapsibleBlockMessage {
    Toggle,
    // Add other messages specific to the block
}

#[derive(Debug, Clone)]
pub struct CollapsibleBlock<'a, Message> {
    title: String,
    content: Element<'a, Message>,
    is_collapsed: bool,
    on_toggle: Box<dyn Fn(bool) -> Message + 'a>,
    is_active: bool,
}

impl<'a, Message> CollapsibleBlock<'a, Message> {
    pub fn new<F>(title: String, content: Element<'a, Message>, is_collapsed: bool, on_toggle: F) -> Self
    where
        F: Fn(bool) -> Message + 'a,
    {
        Self {
            title,
            content,
            is_collapsed,
            on_toggle: Box::new(on_toggle),
            is_active: false,
        }
    }

    pub fn view(self) -> Element<'a, Message> {
        let header = row![
            button(text(if self.is_collapsed { "▶" } else { "▼" }))
                .on_press((self.on_toggle)(!self.is_collapsed))
                .style(iced::widget::button::text::Style::Text),
            text(self.title).size(18).width(Length::Fill),
        ]
        .align_items(alignment::Horizontal::Center)
        .spacing(10);

        let content_view = if self.is_collapsed {
            column![].into()
        } else {
            self.content
        };

        container(
            column![
                header,
                content_view,
            ]
            .spacing(5)
        )
        .padding(10)
        .style(|theme| container::Appearance {
            background: Some(iced::Background::Color(Color::WHITE)),
            border_radius: 5.0,
            border_width: 1.0,
            border_color: theme.palette().text.scale_alpha(0.2),
            ..Default::default()
        })
        .into()
    }

    pub fn set_active(&mut self, active: bool) {
        self.is_active = active;
    }
}

pub fn init() {
    info!("Collapsible block module loaded");
}

impl CollapsibleBlock<'static, CollapsibleBlockMessage> {
    pub fn new_tui(title: String, content: Vec<Line<'static>>) -> Self {
        Self {
            title,
            content: Paragraph::new(content).into(),
            is_collapsed: false,
            on_toggle: Box::new(|is_collapsed| CollapsibleBlockMessage::Toggle),
            is_active: false,
        }
    }

    pub fn toggle_collapse(&mut self) {
        self.is_collapsed = !self.is_collapsed;
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let border_style = if self.is_active {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::White)
        };

        let title_text = if self.is_collapsed {
            format!("{} [▶]", self.title)
        } else {
            format!("{} [▼]", self.title)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(Span::styled(title_text, Style::default().fg(Color::LightGreen)));

        if self.is_collapsed {
            frame.render_widget(block, area);
        } else {
            let inner_area = block.inner(area);
            frame.render_widget(block, area);

            let paragraph = Paragraph::new(self.content.clone())
                .wrap(ratatui::widgets::Wrap { trim: false });
            frame.render_widget(paragraph, inner_area);
        }
    }
}
