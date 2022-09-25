use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Span, Spans},
    widgets::canvas::{Canvas, Line, Map, MapResolution, Rectangle},
    widgets::{
        Axis, BarChart, Block, Borders, Cell, Chart, Dataset, Gauge, LineGauge, List, ListItem,
        Paragraph, Row, Sparkline, Table, Tabs, Wrap,
    },
    Frame,
};

#[derive(Debug, Copy, Clone)]
pub struct WindowPosition {
    pub y: u32,
    pub x: u32,
}

pub struct Window {
    pub title: String,
    pub should_quit: bool,
    pub current_percent_size: u16,
    pub buffer_idx: u16,
}

/*
#[derive(Debug, Copy, Clone)]
pub struct WindowSize {
    pub height: u32,
    pub width: u32,
}

pub trait Window {
    fn get_size(&self) -> WindowSize;
    fn refresh(&self);
    fn append_str(&self, s: &str);
    fn save_cursor_pos(&self);
    fn restore_cursor_pos(&self);
}

pub trait Layout {
    fn create_view_window(&self) -> impl Window;
    fn create_new_status_bar_window(&self) -> impl Window;
}
*/

impl Window {
    pub fn new(buffer_idx: u16) -> Self {
        Self {
            title: "".to_string(),
            should_quit: false,
            current_percent_size: 50,
            buffer_idx,
        }
    }
}
