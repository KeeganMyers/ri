use crate::app::Mode;
use tui::layout::Rect;
use uuid::Uuid;

#[derive(Debug, Copy, Clone)]
pub struct WindowPosition {
    pub y: u32,
    pub x: u32,
}

#[derive(Default, Clone, Debug)]
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
    pub outer_areas: Vec<Option<Rect>>,
    pub area: Option<Rect>,
    pub command_text: Option<String>,
    pub bottom: Option<u16>,
    pub right: Option<u16>,
    pub window_left: Option<Uuid>,
    pub window_right: Option<Uuid>,
    pub window_up: Option<Uuid>,
    pub window_down: Option<Uuid>,
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

    pub fn get_origin(&self) -> Option<(u16, u16)> {
        if let Some(area) = self.area {
            return Some((area.x, area.y));
        }
        None
    }
}
