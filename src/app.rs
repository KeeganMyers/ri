use ropey::Rope;
use arboard::Clipboard;

#[derive(Eq,PartialEq)]
pub enum Mode {
    Insert,
    Append,
    Select,
    Normal
}

use crate::util::event::{Config};
pub struct App {
    pub title: String,
    pub progress: f64,
    pub enhanced_graphics: bool,
    pub x_pos: u16,
    pub y_pos: u16,
    pub start_select_pos: Option<usize>,
    pub char_pos: usize,
    pub mode: Mode,
    pub clipboard: Clipboard,
    pub text: Rope,
    pub should_quit: bool,
    pub past_states: Vec<Rope>,
    pub future_states: Vec<Rope>,
    pub file_path: Option<String>,
    pub command_text: Option<String>
}

impl App {
    pub fn new(enhanced_graphics: bool, file_name: Option<String>) -> Result<App,std::io::Error> {
        match file_name {
            Some(file_path) => {
                let rope = if std::path::Path::new(&file_path).exists() {
                    let file = std::fs::File::open(&file_path)?;
                    let buf_reader = std::io::BufReader::new(file);
                    Rope::from_reader(buf_reader)?
                } else {
                  Rope::new()
                };

                Ok(App {
                    title: file_path.clone(),
                    should_quit: false,
                    progress: 0.0,
                    clipboard: Clipboard::new().unwrap(),
                    enhanced_graphics,
                    mode: Mode::Normal,
                    start_select_pos: None,
                    char_pos: 0,
                    past_states: vec![],
                    future_states: vec![],
                    x_pos: 0,
                    y_pos: 0,
                    file_path: Some(file_path),
                    text: rope,
                    command_text: None,
                })
            },
            None => {
                Ok(App {
                    title: "Ri".to_string(),
                    should_quit: false,
                    progress: 0.0,
                    clipboard: Clipboard::new().unwrap(),
                    enhanced_graphics,
                    mode: Mode::Normal,
                    start_select_pos: None,
                    char_pos: 0,
                    past_states: vec![],
                    future_states: vec![],
                    x_pos: 0,
                    y_pos: 0,
                    file_path: None,
                    text: Rope::new(),
                    command_text: None,
                })
            }
        }
    }

    pub fn on_save(&mut self) -> Result<(), std::io::Error> {
       if let Some(file_path) = &self.file_path {
            let file = if std::path::Path::new(&file_path).exists() {
                std::fs::OpenOptions::new()
                            .read(true)
                            .write(true)
                            .truncate(true)
                            .open(&file_path)?
            } else {
                std::fs::File::create(&file_path)?
            };
           self.text.write_to(std::io::BufWriter::new(file))?;
           return Ok(());
       }
       Ok(())
    }

    pub fn on_up(&mut self) {
        if self.y_pos > 0 {
            self.y_pos -= 1;
        }
    }

    pub fn on_down(&mut self) {
        if self.y_pos < self.text.len_lines() as u16 - 1{
            self.y_pos += 1;
        }
    }

    pub fn current_line_len(&self) -> usize {
        self.text.line(self.y_pos as usize).len_chars()
    }

    pub fn end_of_current_line(&self) -> usize {
         self.text.line_to_char(self.y_pos as usize) + self.current_line_len()
    }

    pub fn on_right(&mut self) {
        let chars = self.current_line_len();
        if self.x_pos < chars as u16 - 1 {
            self.x_pos += 1;
        } else {
            self.on_down();
        }
    }

    pub fn on_left(&mut self) {
        if self.x_pos > 0 {
            self.x_pos -= 1;
            if self.x_pos == 0 {
                self.on_up()
            }
        }
    }

    pub fn get_cursor_idx(&self) -> usize {
         self.text.line_to_char(self.y_pos as usize) + self.x_pos as usize
    }

    pub fn remove_char(&mut self) {
        let end_idx = self.get_cursor_idx();
        if end_idx > 0 {
            let start_idx = end_idx - 1;
            self.future_states = vec![];
            self.past_states.push(self.text.clone());
            let _ = self.text.try_remove(start_idx..end_idx);
            self.on_left()
        }
    }

    pub fn get_selected_range(&self) -> Option<(usize,usize)> {
        let end_idx = self.get_cursor_idx();
        if let Some(start_idx) = self.start_select_pos {
            if start_idx > end_idx {
                return Some((end_idx,start_idx+1));
            } else {
                return Some((start_idx,end_idx+1));
            }
        }
        None
    }

    pub fn on_key(&mut self, c: char,_config: &Config) {
        match c {
            'q'  if self.mode == Mode::Normal => {
                self.should_quit = true;
            },
            'w'  if self.mode == Mode::Normal => {
                let _ = self.on_save();
            },
            'a' if self.mode == Mode::Normal => {
                self.mode = Mode::Append;
            },
            'i' if self.mode == Mode::Normal => {
                self.mode = Mode::Insert;
            },
            'h' if self.mode == Mode::Normal => {
                self.on_left();
            },
            'l' if self.mode == Mode::Normal => {
                self.on_right();
            },
            'j' if self.mode == Mode::Normal => {
                self.on_down();
            },
            'k' if self.mode == Mode::Normal => {
                self.on_up();
            },
            'o' if self.mode == Mode::Normal => {
                self.mode = Mode::Insert;
                let char_idx = self.end_of_current_line();
                self.past_states.push(self.text.clone());
                self.future_states = vec![];
                self.x_pos = 0;
                self.y_pos += 1;
                let _ = self.text.try_insert_char(char_idx,'\n');
            },
            'y' if self.mode == Mode::Select => {
                self.mode = Mode::Normal;
                if let Some((start_idx,end_idx)) = self.get_selected_range() {
                    if let Some(selected_text) = self.text.slice(start_idx..end_idx).as_str() {
                        self.clipboard.set_text(selected_text.to_owned()).expect("Could not set value to system clipboard");
                    }
                }
            },
            'p' if self.mode == Mode::Select => {
                self.mode = Mode::Normal;
                if let Some((start_idx,end_idx)) = self.get_selected_range() {
                    let coppied_text = self.clipboard.get_text().expect("Could not set value to system clipboard");
                    self.past_states.push(self.text.clone());
                    self.future_states = vec![];
                    let _ = self.text.try_remove(start_idx..end_idx);
                    let _ = self.text.try_insert(start_idx,&coppied_text);
                }
            },
            'p' if self.mode == Mode::Normal  => {
                let coppied_text = self.clipboard.get_text().expect("Could not set value to system clipboard");
                let char_idx = self.get_cursor_idx();
                self.past_states.push(self.text.clone());
                self.future_states = vec![];
                let _ = self.text.try_insert(char_idx,&coppied_text);
            },
            'v' if self.mode == Mode::Normal => {
                self.mode = Mode::Select;
                let idx = self.get_cursor_idx();
                self.start_select_pos = Some(idx);
            },
            'u' if self.mode == Mode::Normal => {
                if let Some(past_state) = self.past_states.pop() {
                    self.future_states.push(self.text.clone());
                    self.text = past_state;
                }
            }
            'r' if self.mode == Mode::Normal => {
                if let Some(future_state) = self.future_states.pop() {
                    self.text = future_state;
                }
            },
            'd' if self.mode == Mode::Select => {
                self.mode = Mode::Normal;
                if let Some((start_idx,end_idx)) = self.get_selected_range() {
                    self.future_states = vec![];
                    self.past_states.push(self.text.clone());
                    let _ = self.text.try_remove(start_idx..end_idx);
                }
                self.start_select_pos = None;
            },
            '\n' if self.mode == Mode::Insert => {
                let char_idx = self.get_cursor_idx();
                self.past_states.push(self.text.clone());
                self.future_states = vec![];
                let _ = self.text.try_insert_char(char_idx,c);
                self.x_pos = 0;
                self.on_down();
            },
            '\n' if self.mode == Mode::Append => {
                let char_idx = self.get_cursor_idx() + 1;
                self.past_states.push(self.text.clone());
                self.future_states = vec![];
                if self.text.try_insert_char(char_idx,c).is_ok() {
                    self.y_pos += 1;
                    self.x_pos = 0;
                } else if self.text.try_insert_char(char_idx - 1,c).is_ok() {
                    self.y_pos += 1;
                    self.x_pos = 0;
                }
            },
            _ if self.mode == Mode::Insert => {
                self.past_states.push(self.text.clone());
                self.future_states = vec![];
                let char_idx = self.get_cursor_idx();
                if  self.text.try_insert_char(char_idx,c).is_ok() {
                    self.x_pos += 1;
                }
            },
            _ if self.mode == Mode::Append => {
                self.past_states.push(self.text.clone());
                self.future_states = vec![];
                let char_idx = self.get_cursor_idx() + 1;
                if  self.text.try_insert_char(char_idx,c).is_ok() {
                    self.x_pos += 1;
                } else if self.text.try_insert_char(char_idx - 1,c).is_ok() {
                    self.x_pos += 1;
                }
            }
            _ => {}
        }
    }

    pub fn on_tick(&mut self) {
    }
}
