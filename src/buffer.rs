use std::sync::{Arc, Mutex};

use arboard::Clipboard;
use ropey::Rope;
use uuid::Uuid;
use crate::{add_safe,mut_add_safe,sub_safe,mut_sub_safe};

#[derive(Clone)]
pub struct Buffer {
    pub id: Uuid,
    pub past_states: Vec<Rope>,
    pub future_states: Vec<Rope>,
    pub file_path: Option<String>,
    pub command_text: Option<String>,
    pub x_pos: u16,
    pub y_pos: u16,
    pub start_select_pos: Option<usize>,
    pub end_select_pos: Option<usize>,
    pub clipboard: Arc<Mutex<Clipboard>>,
    pub text: Rope,
    pub title: String,
    pub page_size: u16,
    pub current_page: u16,
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
            mut_sub_safe(&mut self.y_pos,1);
        }
        if self.y_pos != 0 && (self.y_pos % self.page_size) == 0 {
            mut_sub_safe(&mut self.current_page,self.page_size);
        }
    }

    pub fn on_up_range(&self) -> (usize,usize) {
        let mut new_y_pos = self.y_pos;
        if self.y_pos > 0 {
            mut_sub_safe(&mut new_y_pos,1);
        }
        let line_length = self.text.line(self.y_pos as usize).len_chars();
        (self.text.line_to_char(new_y_pos as usize),(self.text.line_to_char(self.y_pos as usize) + line_length))
    }

    pub fn on_down(&mut self) {
        if self.y_pos <= sub_safe(self.text.len_lines() as u16,1) {
            mut_add_safe(&mut self.y_pos,1);
        }
        if self.y_pos != 0 && (self.y_pos % self.page_size) == 0 {
            mut_add_safe(&mut self.current_page,self.page_size);
        }
    }

    pub fn on_down_range(&self) -> (usize,usize) {
        let mut new_y_pos = self.y_pos;

        if self.y_pos <= sub_safe(self.text.len_lines() as u16, 1) {
            mut_add_safe(&mut new_y_pos,1);
        }
        (self.text.line_to_char(new_y_pos as usize),(self.text.line_to_char(self.y_pos as usize)))
    }

    pub fn current_line_len(&self) -> usize {
        self
            .text
            .get_line(self.y_pos as usize)
            .map(|l| l.len_chars())
            .unwrap_or_default()
    }

    pub fn end_of_current_line(&self) -> usize {
        self.start_of_current_line() + self.current_line_len()
    }

    pub fn start_of_current_line(&self) -> usize {
        self.text.line_to_char(self.y_pos as usize)
    }

    pub fn current_line_chars(&self) -> Vec<char> {
        self.text
            .line(self.y_pos as usize)
            .chars()
            .filter(|c| c != &'\n')
            .collect()
    }

    pub fn on_right(&mut self) {
        let chars: u16 = self.current_line_len() as u16;
        if self.x_pos < sub_safe(chars,1) {
            mut_add_safe(&mut self.x_pos, 1);
        } else {
            self.on_down();
        }
    }

    pub fn on_right_range(&self) -> (usize,usize) {
        let mut new_x_pos = self.x_pos;
        let chars: u16 = self.current_line_len() as u16;

        if self.x_pos < sub_safe(chars,1) {
            mut_add_safe(&mut new_x_pos,1);
        }
        (self.text.line_to_char(new_x_pos as usize),(self.text.line_to_char(self.x_pos as usize)))
    }

    pub fn on_left(&mut self) {
        if self.x_pos > 0 {
            mut_sub_safe(&mut self.x_pos,1);
            if self.x_pos == 0 {
                self.on_up()
            }
        } else {
            self.on_up()
        }
    }

    pub fn on_left_range(&self) -> (usize,usize) {
        let mut new_x_pos = self.x_pos;
        let chars: u16 = self.current_line_len() as u16;

        if self.x_pos < sub_safe(chars,1) {
            mut_sub_safe(&mut new_x_pos,1);
        }
        (self.text.line_to_char(new_x_pos as usize),(self.text.line_to_char(self.x_pos as usize)))
    }

    pub fn recenter(&mut self) {
        if self.y_pos <= 0_u16 {
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
            let start_idx = sub_safe(end_idx as u16,1) as usize;
            self.future_states = vec![];
            self.past_states.push(self.text.clone());
            let _ = self.text.try_remove(start_idx..end_idx);
            self.on_left()
        }
    }

    /*
    pub fn get_selected_range(&self) -> Option<(usize, usize)> {
        let offset;
        let end_idx;
        if let Some(end) = self.end_select_pos {
            offset = 0;
            end_idx = end;
        } else {
            offset = 1;
            end_idx = self.get_cursor_idx();
        }
        if let Some(start_idx) = self.start_select_pos {
            if start_idx > end_idx {
                return Some((end_idx, start_idx + offset));
            } else {
                return Some((start_idx, end_idx + offset));
            }
        }
        None
    }
    */

    pub fn end_current_word(&self) -> u16 {
        let line_chars = self.current_line_chars();
        let mut chars_cursor = line_chars[self.x_pos as usize..].iter();
        let mut end_current_word = self.x_pos.clone();

        while let Some(c) = chars_cursor.next() {
            if !c.is_alphabetic() {
                mut_add_safe(&mut end_current_word,1);
                break;
            }
            mut_add_safe(&mut end_current_word,1);
        }
        end_current_word
    }

    pub fn find_next_word(&self) -> u16 {
        let line_chars = self.current_line_chars();
        let end_current_word = self.end_current_word();

        if end_current_word != self.x_pos {
            let mut chars_end_word = line_chars[end_current_word as usize..].iter();
            let mut start_next_word = end_current_word.clone();
            while let Some(c) = chars_end_word.next() {
                if c.is_alphabetic() {
                    mut_add_safe(&mut start_next_word,1);
                    break;
                } else {
                    mut_add_safe(&mut start_next_word,1);
                }
            }
            start_next_word
        } else {
            end_current_word
        }
    }

    pub fn find_last_word(&self) -> u16 {
        let line_chars = self.current_line_chars();
        let mut chars_cursor = line_chars[..=self.x_pos as usize].iter().rev();
        let mut start_current_word = self.x_pos.clone();

        while let Some(c) = chars_cursor.next() {
            if !c.is_alphabetic() {
                mut_sub_safe(&mut start_current_word,1);
                break;
            }
            mut_sub_safe(&mut start_current_word,1);
        }

        if start_current_word != self.x_pos {
            let mut chars_end_word = line_chars[..=start_current_word as usize].iter().rev();
            let mut start_last_word = start_current_word.clone();
            while let Some(c) = chars_end_word.next() {
                if c.is_alphabetic() {
                    mut_sub_safe(&mut start_last_word,1);
                    break;
                } else {
                    mut_sub_safe(&mut start_last_word,1);
                }
            }
            start_last_word
        } else {
            start_current_word
        }
    }

    pub fn add_newline_above(&mut self) {
        unimplemented!()
    }

    pub fn yank_line_range(&self, start_idx: usize, end_idx: usize) {
        if let (Ok(mut clipboard),Some(rope_slice)) = (self.clipboard.lock(),self.text.get_slice(start_idx..end_idx as usize)) {
        clipboard.set_text(rope_slice)
                .expect("Could not set value to system clipboard");
        }
    }

    pub fn yank_lines(&self, start_idx: usize, end_idx: usize) {
        //account for the chance that the range is reversed
        let mut str_vec = vec![];
        let line_count = self.text.len_lines() as usize;
        if start_idx < 1 || end_idx < 1 {
            return ();
        }

        if end_idx > line_count || start_idx > line_count {
            return ();
        }

        if end_idx < start_idx {
            for line in sub_safe(end_idx as u16,1) as usize..start_idx {
                str_vec.push(self.text.line(line).to_string())
            }
        } else {
            for line in sub_safe(start_idx as u16,1) as usize..end_idx {
                str_vec.push(self.text.line(line).to_string())
            }
        }

        if let Ok(mut clipboard) = self.clipboard.lock() {
            clipboard
                .set_text(str_vec.join(""))
                .expect("Could not set value to system clipboard");
        }
    }

    pub fn move_to_last_line(&mut self) {
        let line_count = self.text.len_lines() as u16;
        self.x_pos = 0;
        self.y_pos = sub_safe(line_count,1);
        if line_count > self.page_size {
            self.current_page = sub_safe(sub_safe(line_count,self.page_size),1);
        } else {
            self.current_page = line_count;
        }
    }

    pub fn move_to_first_line(&mut self) {
        self.y_pos = 0;
        self.current_page = 0;
        self.x_pos = 0;
    }

    pub fn move_to_line_number(&mut self, line_number: usize) {
        let line_count = self.text.len_lines() as usize;
        if line_number < line_count {
            self.y_pos = sub_safe(line_number as u16,1);
            self.x_pos = 0;
            if line_number >= self.page_size as usize {
                self.current_page = sub_safe(sub_safe(line_number as u16,(line_number % self.page_size as usize) as u16)
                   ,self.page_size)
            } else {
                self.current_page = 0
            }
        }
    }

    pub fn select_line(&mut self) {
        let idx = self.get_cursor_idx();
        self.start_select_pos = Some(idx);
        self.end_select_pos = Some(self.end_of_current_line());
    }

    pub fn add_newline_below(&mut self) {
        let char_idx = self.end_of_current_line();
        self.past_states.push(self.text.clone());
        self.future_states = vec![];
        self.x_pos = 0;
        mut_add_safe(&mut self.y_pos,1);
        let _ = self.text.try_insert_char(char_idx, '\n');
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
        if let Ok(mut clipboard) = self.clipboard.lock() {
            let coppied_text = clipboard
                .get_text()
                .expect("Could not set value to system clipboard");
            let char_idx = self.get_cursor_idx();
            self.past_states.push(self.text.clone());
            self.future_states = vec![];
            let _ = self.text.try_insert(char_idx, &coppied_text);
        }
    }

    pub fn delete_line(&mut self) {
        self.future_states = vec![];
        self.past_states.push(self.text.clone());
        let _ = self
            .text
            .try_remove(self.start_of_current_line()..self.end_of_current_line());
        self.recenter();
    }

    pub fn delete_line_range(&mut self,start_idx: usize, end_idx: usize) {
        self.future_states = vec![];
        self.past_states.push(self.text.clone());

        let _ = self
            .text
            .try_remove(start_idx..end_idx);
        self.recenter();
    }

    pub fn set_states(&mut self) {
        self.future_states = vec![];
        self.past_states.push(self.text.clone());
    }

    pub fn delete_line_direct(&mut self) {
        let _ = self
            .text
            .try_remove(self.start_of_current_line()..self.end_of_current_line());
        self.recenter();
    }

    pub fn insert_return(&mut self) {
        let char_idx = self.get_cursor_idx();
        self.past_states.push(self.text.clone());
        self.future_states = vec![];
        let _ = self.text.try_insert_char(char_idx, '\n');
        self.x_pos = 0;
        self.on_down();
    }
    pub fn insert_chars(&mut self, chars: &str) {
        self.past_states.push(self.text.clone());
        self.future_states = vec![];
        let char_idx = self.get_cursor_idx();
        if self.text.try_insert(char_idx, &chars).is_ok() {
            mut_add_safe(&mut self.x_pos, chars.len() as u16);
        }
    }
    pub fn append_return(&mut self) {
        let char_idx = add_safe(self.get_cursor_idx() as u16,1);
        self.past_states.push(self.text.clone());
        self.future_states = vec![];
        if self.text.try_insert_char(char_idx as usize, '\n').is_ok() {
            mut_add_safe(&mut self.y_pos, 1);
            self.x_pos = 0;
        } else if self.text.try_insert_char(sub_safe(char_idx,1) as usize, '\n').is_ok() {
            mut_add_safe(&mut self.y_pos, 1);
            self.x_pos = 0;
        }
    }

    pub fn append_chars(&mut self, chars: &str) {
        self.past_states.push(self.text.clone());
        self.future_states = vec![];
        let char_idx = add_safe(self.get_cursor_idx() as u16,1);
        if self.text.try_insert(char_idx as usize, &chars).is_ok() {
            mut_add_safe(&mut self.x_pos,chars.len() as u16);
        } else if self.text.try_insert(sub_safe(char_idx,1) as usize, &chars).is_ok() {
            mut_add_safe(&mut self.x_pos,chars.len() as u16);
        }
    }

    pub fn new(file_name: Option<String>) -> Result<Self, std::io::Error> {
        match file_name {
            Some(file_path) => {
                let rope = if std::path::Path::new(&file_path.trim()).exists() {
                    let file = std::fs::File::open(&file_path.trim())?;
                    let buf_reader = std::io::BufReader::new(file);
                    Rope::from_reader(buf_reader)?
                } else {
                    Rope::new()
                };

                Ok(Self {
                    id: Uuid::new_v4(),
                    title: file_path.clone(),
                    clipboard: Arc::new(Mutex::new(Clipboard::new().unwrap())),
                    start_select_pos: None,
                    end_select_pos: None,
                    past_states: vec![],
                    future_states: vec![],
                    x_pos: 0,
                    y_pos: 0,
                    file_path: Some(file_path.trim().to_owned()),
                    text: rope,
                    command_text: None,
                    current_page: 0,
                    page_size: 10,
                })
            }
            None => Ok(Self {
                id: Uuid::new_v4(),
                title: "Ri".to_string(),
                clipboard: Arc::new(Mutex::new(Clipboard::new().unwrap())),
                start_select_pos: None,
                end_select_pos: None,
                past_states: vec![],
                future_states: vec![],
                x_pos: 0,
                y_pos: 0,
                file_path: None,
                text: Rope::new(),
                command_text: None,
                current_page: 0,
                page_size: 10,
            }),
        }
    }
}
