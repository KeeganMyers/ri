pub struct Split {
    pub title: String,
    pub y_offset: u16,
    pub x_offset: u16,
    pub x_pos: u16,
    pub y_pos: u16,
    pub start_select_pos: Option<usize>,
    pub end_select_pos: Option<usize>,
    pub char_pos: usize,
    pub mode: Mode,
    pub clipboard: Clipboard,
    pub text: Rope,
    pub should_quit: bool,
    pub past_states: Vec<Rope>,
    pub future_states: Vec<Rope>,
    pub file_path: Option<String>,
    pub command_text: Option<String>,
    pub last_char: Option<char>
}

impl Split {
}
