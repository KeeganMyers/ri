use crate::app::Mode;
use tui::layout::Rect;
use uuid::Uuid;

#[derive(Debug, Copy, Clone)]
pub struct WindowPosition {
    pub y: u32,
    pub x: u32,
}

#[derive(Default, Debug)]
pub struct Window {
    pub id: Uuid,
    pub title: String,
    pub current_percent_size: u16,
    pub buffer_id: Uuid,
    pub y_offset: u16,
    pub x_offset: u16,
    pub x_pos: u16,
    pub y_pos: u16,
    pub mode: Mode,
    pub page_size: u16,
    pub current_page: u16,
    pub area: Rect,
    pub command_text: Option<String>,
}

impl Window {
    pub fn new(_buffer_idx: u16) -> Self {
        Self::default()
    }

    pub fn display_x_pos(&self) -> u16 {
        self.x_pos + self.x_offset
    }

    pub fn display_y_pos(&self) -> u16 {
        (self.y_pos + self.y_offset) - self.current_page
    }
}
