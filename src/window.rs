pub use self::termion::TermionLayout;

#[derive(Debug, Copy, Clone)]
pub struct WindowPosition {
    pub y: u32,
    pub x: u32,
}

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

/*
pub trait Layout {
    fn create_view_window(&self) -> impl Window;
    fn create_new_status_bar_window(&self) -> impl Window;
}
*/
