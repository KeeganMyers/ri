use crate::{Command,NormalCommand,util::event::{Config, Event, Events}};
use anyhow::{Result as AnyHowResult, Error as AnyHowError};
use crate::app::Mode;
use arboard::Clipboard;
use ropey::Rope;
use termion::{event::Key,input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};

pub struct Buffer {
    pub should_quit: bool,
    pub past_states: Vec<Rope>,
    pub future_states: Vec<Rope>,
    pub file_path: Option<String>,
    pub command_text: Option<String>,
    pub last_char: Option<char>,
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
    pub title: String
}

impl Buffer {
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
    
    pub fn start_of_current_line(&self) -> usize {
         self.text.line_to_char(self.y_pos as usize)
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

    pub fn recenter(&mut self) {
        if self.y_pos < 0_u16 {
            self.on_down()
        }
        if self.y_pos < self.text.len_lines() as u16 {
            self.on_up()
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
        let mut offset = 0;
        let mut end_idx = 0;
        if let Some(end) = self.end_select_pos {
            offset = 0;
            end_idx = end;
        } else {
            offset = 1;
            end_idx = self.get_cursor_idx();
        }
        if let Some(start_idx) = self.start_select_pos {
            if start_idx > end_idx {
                return Some((end_idx,start_idx+offset));
            } else {
                return Some((start_idx,end_idx+offset));
            }
        }
        None
    }

    pub fn execute_normal(&mut self, c: char,config: &Config) -> AnyHowResult<()> {
        let tokens = NormalCommand::tokenize(c.to_string());     
        self.command_text = Some(format!("{:?}",NormalCommand::parse(tokens.clone())));
        for command in NormalCommand::parse(tokens)? {
            match command {
                NormalCommand::Left => self.on_left(),
                NormalCommand::Right => self.on_right(),
                NormalCommand::Up => self.on_up(),
                NormalCommand::Down => self.on_down(),
                NormalCommand::Insert => self.set_insert_mode(),
                NormalCommand::Append => self.set_insert_mode(),
                NormalCommand::AddNewLineBelow => self.add_newline_below(),
                NormalCommand::AddNewLineAbove => self.add_newline_above(),
                NormalCommand::Paste => self.paste_text(),
                NormalCommand::Undo => self.undo(),
                NormalCommand::Redo => self.redo(),
                NormalCommand::DeleteLine => self.delete_line(),
                NormalCommand::Visual => self.set_visual_mode(),
                NormalCommand::VisualLine => self.select_line(),
                _ => ()
            }
        }
        Ok(())
    }

    pub fn add_newline_above(&mut self) {
        unimplemented!()
    }

    pub fn select_line(&mut self) {
        self.mode = Mode::Visual;
        let idx = self.get_cursor_idx();
        self.start_select_pos = Some(idx);
        self.end_select_pos = Some(self.end_of_current_line());
    }

    pub fn add_newline_below(&mut self) {
        self.set_insert_mode();
        let char_idx = self.end_of_current_line();
        self.past_states.push(self.text.clone());
        self.future_states = vec![];
        self.x_pos = 0;
        self.y_pos += 1;
        let _ = self.text.try_insert_char(char_idx,'\n');
    }

    pub fn undo(&mut self) {
        if let Some(past_state) = self.past_states.pop() {
            self.future_states.push(self.text.clone());
            self.text = past_state;
            self.recenter();
        }
    }

    pub fn redo(&mut self) {
        if let Some(future_state) = self.future_states.pop() {
            self.text = future_state;
        }
    }

    pub fn paste_text(&mut self) {
        let coppied_text = self.clipboard.get_text().expect("Could not set value to system clipboard");
        let char_idx = self.get_cursor_idx();
        self.past_states.push(self.text.clone());
        self.future_states = vec![];
        let _ = self.text.try_insert(char_idx,&coppied_text);
    }

    pub fn delete_line(&mut self) {
        self.future_states = vec![];
        self.past_states.push(self.text.clone());
        let _ = self.text.try_remove(self.start_of_current_line()..self.end_of_current_line());
        self.recenter();
    }

    pub fn on_key(&mut self, c: char,config: &Config) {
        match self.mode {
            Mode::Normal => self.execute_normal(c, config),
            _ => Ok(())
        };
        /*
        match c {
            'y' if self.mode == Mode::Visual => {
                self.mode = Mode::Normal;
                if let Some((start_idx,end_idx)) = self.get_selected_range() {
                    if let Some(selected_text) = self.text.slice(start_idx..end_idx).as_str() {
                        self.clipboard.set_text(selected_text.to_owned()).expect("Could not set value to system clipboard");
                    }
                }
                self.start_select_pos = None;
                self.end_select_pos = None;
            },
            'p' if self.mode == Mode::Visual => {
                self.mode = Mode::Normal;
                if let Some((start_idx,end_idx)) = self.get_selected_range() {
                    let coppied_text = self.clipboard.get_text().expect("Could not set value to system clipboard");
                    self.past_states.push(self.text.clone());
                    self.future_states = vec![];
                    let _ = self.text.try_remove(start_idx..end_idx);
                    let _ = self.text.try_insert(start_idx,&coppied_text);
                }
            },
            'd' if self.mode == Mode::Visual => {
                self.mode = Mode::Normal;
                if let Some((start_idx,end_idx)) = self.get_selected_range() {
                    self.future_states = vec![];
                    self.past_states.push(self.text.clone());
                    let _ = self.text.try_remove(start_idx..end_idx);
                    self.recenter();
                }
                self.start_select_pos = None;
                self.end_select_pos = None;
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
            _ if self.mode == Mode::Command => {
                self.command_text =  self.command_text.clone().map(|mut t| {
                    t.push_str(&c.to_string()); 
                    t});
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
        }*/
    }

    pub fn set_command_mode(&mut self) {
        self.mode = Mode::Command
    }

    pub fn set_insert_mode(&mut self) {
        self.mode = Mode::Insert
    }

    pub fn set_visual_mode(&mut self) {
        self.mode = Mode::Visual;
        let idx = self.get_cursor_idx();
        self.start_select_pos = Some(idx);
    }

    pub fn set_append_mode(&mut self) {
        self.mode = Mode::Append
    }

    pub fn set_normal_mode(&mut self) {
        self.mode = Mode::Normal
    }

    pub fn new(file_name: Option<String>) -> Result<Self,std::io::Error> {
        match file_name {
            Some(file_path) => {
                let rope = if std::path::Path::new(&file_path).exists() {
                    let file = std::fs::File::open(&file_path)?;
                    let buf_reader = std::io::BufReader::new(file);
                    Rope::from_reader(buf_reader)?
                } else {
                  Rope::new()
                };

                Ok(Self {
                    title: file_path.clone(),
                    should_quit: false,
                    clipboard: Clipboard::new().unwrap(),
                    mode: Mode::Normal,
                    start_select_pos: None,
                    end_select_pos: None,
                    char_pos: 0,
                    past_states: vec![],
                    future_states: vec![],
                    x_pos: 0,
                    y_pos: 0,
                    x_offset: 0,
                    y_offset: 0,
                    file_path: Some(file_path),
                    text: rope,
                    command_text: None,
                    last_char: None,
                })
            },
            None => {
                Ok(Self {
                    title: "Ri".to_string(),
                    should_quit: false,
                    clipboard: Clipboard::new().unwrap(),
                    mode: Mode::Normal,
                    start_select_pos: None,
                    end_select_pos: None,
                    char_pos: 0,
                    past_states: vec![],
                    future_states: vec![],
                    x_pos: 0,
                    y_pos: 0,
                    x_offset: 0,
                    y_offset: 0,
                    file_path: None,
                    text: Rope::new(),
                    command_text: None,
                    last_char: None,
                })
            }
        }
    }

    pub fn on_event(&mut self,event: Event<Key>,config: &Config) {
        match event {
            Event::Input(key) => match key {
                Key::Up => {
                    self.on_up();
                }
                Key::Backspace if self.mode == self::Mode::Insert || self.mode == self::Mode::Append  => {
                    self.remove_char();
                }
                Key::Backspace if self.mode == self::Mode::Normal => {
                    self.on_left();
                }
                Key::Down => {
                    self.on_down();
                }
                Key::Left => {
                    self.on_left();
                }
                Key::Right => {
                    self.on_right();
                }
                Key::Esc => {
                    if self.mode == self::Mode::Insert || self.mode == self::Mode::Append || self.mode == self::Mode::Visual {
                        self.start_select_pos = None;
                        self.set_normal_mode();
                    }
                    if self.mode == self::Mode::Command {
                        self.set_normal_mode();
                    }
                }
                Key::Char(c) if c == ':' => {
                    self.command_text = Some("".to_string());
                    self.set_command_mode();
                    self.on_key(c, &config);
                },
                Key::Char(c) => {
                    self.on_key(c, &config);
                    if self.mode == self::Mode::Normal {
                        self.last_char = Some(c);
                    } else {
                        self.last_char = None;
                    }
                }
                _ => {}
            _ => ()
            },
            Event::Tick => {
                ()
            }
        }
  }
}