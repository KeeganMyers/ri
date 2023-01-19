use crate::app::Mode;
use actix::prelude::*;
use crate::token::{
    command_token::*,
    display_token::{DisplayToken, WindowChange},
    NormalToken, OperatorToken,
    RangeToken, 
};
use crate::token::{display_token::*, command_token::*,normal_token::*, Token};

use anyhow::{Error as AnyHowError, Result as AnyHowResult};
use arboard::Clipboard;
use log::trace;
use ropey::Rope;
use uuid::Uuid;

impl Actor for Buffer {
    type Context = Context<Self>;
}


impl Handler<Quit> for Buffer {
    type Result = ();

    fn handle(&mut self, _msg: Quit, _ctx: &mut Context<Self>) {
                self.set_normal_mode();
                unimplemented!();
                /*
                Ok(vec![
                    Token::Display(DisplayToken::CloseWindow(self.id)),
                    Token::Display(DisplayToken::DrawViewPort),
                ])
                */
    }
}


impl Handler<Write> for Buffer {
    type Result = ();

    fn handle(&mut self, _msg: Write, _ctx: &mut Context<Self>) {
                let _ = self.on_save();
                unimplemented!();
                //Ok(vec![Token::Display(DisplayToken::DrawViewPort)])
    }
}


impl Handler<Split> for Buffer {
    type Result = ();

    fn handle(&mut self, _msg: Split, _ctx: &mut Context<Self>) {
            self.set_normal_mode();
    }
}


impl Handler<VerticalSplit> for Buffer {
    type Result = ();

    fn handle(&mut self, _msg: VerticalSplit, _ctx: &mut Context<Self>) {
            self.set_normal_mode();
    }
}


impl Handler<crate::token::normal_token::Esc> for Buffer {
    type Result = ();

    fn handle(&mut self, _msg: crate::token::normal_token::Esc, _ctx: &mut Context<Self>) {
                self.start_select_pos = None;
                self.set_normal_mode();
                unimplemented!();
                //Ok(vec![Token::Display(DisplayToken::DrawViewPort)])
                /*
                if mode == Mode::Insert {
                    self.start_select_pos = None;
                    self.set_normal_mode();
                    //self.standard_normal_response()
                }
                */
    }
}

impl Handler<Remove> for Buffer {
    type Result = ();

    fn handle(&mut self, _msg: Remove, _ctx: &mut Context<Self>) {
                self.remove_char();
                if self.mode == Mode::Insert {
                    //self.standard_insert_response()
                }
    }
}

impl Handler<Append> for Buffer {
    type Result = ();

    fn handle(&mut self, msg: Append, _ctx: &mut Context<Self>) {
                match self.mode {
                    Mode::Append => {
                    let chars = msg.chars;
                    self.past_states.push(self.text.clone());
                    self.future_states = vec![];
                    let char_idx = self.get_cursor_idx() + 1;
                    if self.text.try_insert(char_idx, &chars).is_ok() {
                        self.x_pos += chars.len() as u16;
                    } else if self.text.try_insert(char_idx - 1, &chars).is_ok() {
                        self.x_pos += chars.len() as u16;
                    }
                },
                Mode::Insert => {
                    let chars = msg.chars;
                    self.past_states.push(self.text.clone());
                    self.future_states = vec![];
                    let char_idx = self.get_cursor_idx();
                    if self.text.try_insert(char_idx, &chars).is_ok() {
                        self.x_pos += chars.len() as u16;
                    }
                    unimplemented!();
                    //self.standard_insert_response()
                },
                 Mode::Command => {
                    self.command_text = self.command_text.clone().map(|mut t| {
                        t.push_str(&msg.chars);
                        t
                    });
                    unimplemented!();
                    /*
                    Ok(vec![
                        Token::Display(DisplayToken::AppendCommand(
                            self.id,
                            self.command_text.clone(),
                        )),
                        Token::Display(DisplayToken::DrawViewPort),
                    ])
                    */
                },
                _ => ()
    }
}
}


impl Handler<crate::token::normal_token::Enter> for Buffer {
    type Result = ();

    fn handle(&mut self, msg: crate::token::normal_token::Enter, _ctx: &mut Context<Self>) {
                match self.mode {
                    Mode::Append => {
                    let char_idx = self.get_cursor_idx() + 1;
                    self.past_states.push(self.text.clone());
                    self.future_states = vec![];
                    if self.text.try_insert_char(char_idx, '\n').is_ok() {
                        self.y_pos += 1;
                        self.x_pos = 0;
                    } else if self.text.try_insert_char(char_idx - 1, '\n').is_ok() {
                        self.y_pos += 1;
                        self.x_pos = 0;
                    }
                },
                Mode::Insert => {
                    let char_idx = self.get_cursor_idx();
                    self.past_states.push(self.text.clone());
                    self.future_states = vec![];
                    let _ = self.text.try_insert_char(char_idx, '\n');
                    self.x_pos = 0;
                    self.on_down();
                    unimplemented!();
                    /*
                    Ok(vec![
                        Token::Display(DisplayToken::CacheNewLine(
                            self.id,
                            self.text.clone(),
                            self.y_pos as usize,
                        )),
                        Token::Display(DisplayToken::UpdateWindow(WindowChange {
                            id: self.id,
                            x_pos: self.x_pos,
                            y_pos: self.y_pos,
                            mode: self.mode.clone(),
                            title: Some(self.title.clone()),
                            page_size: self.page_size,
                            current_page: self.current_page,
                            ..WindowChange::default()
                        })),
                        Token::Display(DisplayToken::DrawViewPort),
                    ])
                    */
                }
                Mode::Normal => {
                    self.command_text = self.command_text.clone().map(|mut t| {
                        t.truncate(t.len() - 1);
                        t
                    });
                    unimplemented!();
                    /*
                    Ok(vec![
                        Token::Display(DisplayToken::AppendCommand(
                            self.id,
                            self.command_text.clone(),
                        )),
                        Token::Display(DisplayToken::DrawViewPort),
                    ])
                    */
                },
                _ => ()
                };
    }
}


impl Handler<Up> for Buffer {
    type Result = ();

    fn handle(&mut self, _msg: Up, _ctx: &mut Context<Self>) {
                self.on_up();
                //self.standard_normal_response()
    }
}

impl Handler<Down> for Buffer {
    type Result = ();

    fn handle(&mut self, _msg: Down, _ctx: &mut Context<Self>) {
                self.on_down();
                //self.standard_normal_response()
    }
}

impl Handler<Left> for Buffer {
    type Result = ();

    fn handle(&mut self, _msg: Left, _ctx: &mut Context<Self>) {
                self.on_left();
                //self.standard_normal_response()
    }
}

impl Handler<Right> for Buffer {
    type Result = ();

    fn handle(&mut self, _msg: Right, _ctx: &mut Context<Self>) {
                self.on_right();
                //self.standard_normal_response()
    }
}

impl Handler<SwitchToCommand> for Buffer {
    type Result = ();

    fn handle(&mut self, _msg: SwitchToCommand, _ctx: &mut Context<Self>) {
                self.command_text = Some("".to_string());
                self.set_command_mode();
                //self.standard_normal_response()
    }
}

impl Handler<SwitchToInsert> for Buffer {
    type Result = ();

    fn handle(&mut self, _msg: SwitchToInsert, _ctx: &mut Context<Self>) {
                self.set_insert_mode();
                //self.standard_normal_response()
    }
}

impl Handler<SwitchToAppend> for Buffer {
    type Result = ();

    fn handle(&mut self, _msg: SwitchToAppend, _ctx: &mut Context<Self>) {
                self.set_append_mode();
                //self.standard_normal_response()
    }
}


impl Handler<AddNewLineBelow> for Buffer {
    type Result = ();

    fn handle(&mut self, _msg: AddNewLineBelow, _ctx: &mut Context<Self>) {
                self.add_newline_below();
                /*
                Ok(vec![
                    Token::Display(DisplayToken::CacheNewLine(
                        self.id,
                        self.text.clone(),
                        self.y_pos as usize,
                    )),
                    Token::Display(DisplayToken::UpdateWindow(WindowChange {
                        id: self.id,
                        x_pos: self.x_pos,
                        y_pos: self.y_pos,
                        mode: self.mode.clone(),
                        title: Some(self.title.clone()),
                        page_size: self.page_size,
                        current_page: self.current_page,
                        ..WindowChange::default()
                    })),
                    Token::Display(DisplayToken::DrawViewPort),
                ])
                */
    }
}

impl Handler<AddNewLineAbove> for Buffer {
    type Result = ();

    fn handle(&mut self, _msg: AddNewLineAbove, _ctx: &mut Context<Self>) {
                self.add_newline_above();
                //self.standard_normal_response()
    }
}

impl Handler<Paste> for Buffer {
    type Result = ();

    fn handle(&mut self, _msg: Paste, _ctx: &mut Context<Self>) {
                self.paste_text();
                //self.standard_normal_response()
    }
}


impl Handler<Undo> for Buffer {
    type Result = ();

    fn handle(&mut self, _msg: Undo, _ctx: &mut Context<Self>) {
                self.undo();
                //self.standard_normal_response()
    }
}

impl Handler<Redo> for Buffer {
    type Result = ();

    fn handle(&mut self, _msg: Redo, _ctx: &mut Context<Self>) {
                self.redo();
                //self.standard_normal_response()
    }
}

impl Handler<DeleteLine> for Buffer {
    type Result = ();

    fn handle(&mut self, _msg: DeleteLine, _ctx: &mut Context<Self>) {
                let removed_line_index = self.y_pos;
                self.delete_line();
                unimplemented!();
                /*
                Ok(vec![
                    Token::Display(DisplayToken::RemoveCacheLine(
                        self.id,
                        self.text.clone(),
                        removed_line_index as usize,
                    )),
                    Token::Display(DisplayToken::UpdateWindow(WindowChange {
                        id: self.id,
                        x_pos: self.x_pos,
                        y_pos: self.y_pos,
                        mode: self.mode.clone(),
                        title: Some(self.title.clone()),
                        page_size: self.page_size,
                        current_page: self.current_page,
                        ..WindowChange::default()
                    })),
                    Token::Display(DisplayToken::DrawViewPort),
                ])
                */
    }
}

impl Handler<Visual> for Buffer {
    type Result = ();

    fn handle(&mut self, _msg: Visual, _ctx: &mut Context<Self>) {
                self.set_visual_mode();
                //self.standard_normal_response()
    }
}

impl Handler<VisualLine> for Buffer {
    type Result = ();

    fn handle(&mut self, _msg: VisualLine, _ctx: &mut Context<Self>) {
                self.select_line();
                //self.standard_normal_response()
    }
}

impl Handler<Last> for Buffer {
    type Result = ();

    fn handle(&mut self, _msg: Last, _ctx: &mut Context<Self>) {
                self.x_pos = (self.current_line_len() - 2) as u16;
                //self.standard_normal_response()
    }
}

impl Handler<LastNonBlank> for Buffer {
    type Result = ();

    fn handle(&mut self, _msg: LastNonBlank, _ctx: &mut Context<Self>) {
                self.x_pos = (self.current_line_len() - 2) as u16;
                //self.standard_normal_response()
    }
}

impl Handler<First> for Buffer {
    type Result = ();

    fn handle(&mut self, _msg: First, _ctx: &mut Context<Self>) {
                self.x_pos = 0 as u16;
                //self.standard_normal_response()
    }
}

impl Handler<FirstNonBlank> for Buffer {
    type Result = ();

    fn handle(&mut self, _msg: FirstNonBlank, _ctx: &mut Context<Self>) {
                self.x_pos = 0 as u16;
                //self.standard_normal_response()
    }
}

impl Handler<StartWord> for Buffer {
    type Result = ();

    fn handle(&mut self, _msg: StartWord, _ctx: &mut Context<Self>) {
                self.x_pos = self.find_next_word();
                //self.standard_normal_response()
    }
}

pub struct Buffer {
    pub id: Uuid,
    pub past_states: Vec<Rope>,
    pub future_states: Vec<Rope>,
    pub file_path: Option<String>,
    pub command_text: Option<String>,
    pub operator: Option<OperatorToken>,
    pub x_pos: u16,
    pub y_pos: u16,
    pub start_select_pos: Option<usize>,
    pub end_select_pos: Option<usize>,
    pub mode: Mode,
    pub clipboard: Clipboard,
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
            self.y_pos -= 1;
        }
        if self.y_pos != 0 && (self.y_pos % self.page_size) == 0 {
            self.current_page -= self.page_size;
        }
    }

    pub fn on_down(&mut self) {
        if self.y_pos < self.text.len_lines() as u16 - 1 {
            self.y_pos += 1;
        }
        if self.y_pos != 0 && (self.y_pos % self.page_size) == 0 {
            self.current_page += self.page_size;
        }
    }

    pub fn current_line_len(&self) -> usize {
        let len = self
            .text
            .get_line(self.y_pos as usize)
            .map(|l| l.len_chars())
            .unwrap_or_default();
        trace!("line_length {:?}", len);
        len
    }

    pub fn end_of_current_line(&self) -> usize {
        let len = self.start_of_current_line() + self.current_line_len();
        trace!("end of current line {:?}", len);
        len
    }

    pub fn start_of_current_line(&self) -> usize {
        let len = self.text.line_to_char(self.y_pos as usize);
        trace!("start of current line {:?}", len);
        len
    }

    pub fn current_line_chars(&self) -> Vec<char> {
        self.text
            .line(self.y_pos as usize)
            .chars()
            .filter(|c| c != &'\n')
            .collect()
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
            let start_idx = end_idx - 1;
            self.future_states = vec![];
            self.past_states.push(self.text.clone());
            let _ = self.text.try_remove(start_idx..end_idx);
            self.on_left()
        }
    }

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

    pub fn find_next_word(&self) -> u16 {
        let line_chars = self.current_line_chars();
        trace!("line chars {:?}", line_chars);
        let mut chars_cursor = line_chars[self.x_pos as usize..].iter();
        let mut end_current_word = self.x_pos.clone();

        while let Some(c) = chars_cursor.next() {
            if !c.is_alphabetic() {
                end_current_word += 1;
                break;
            }
            end_current_word += 1;
        }

        if end_current_word != self.x_pos {
            let mut chars_end_word = line_chars[end_current_word as usize..].iter();
            let mut start_next_word = end_current_word.clone();
            while let Some(c) = chars_end_word.next() {
                if c.is_alphabetic() {
                    start_next_word += 1;
                    break;
                } else {
                    start_next_word += 1;
                }
            }
            start_next_word
        } else {
            end_current_word
        }
    }

    pub fn find_range(&self, range: &RangeToken) -> Option<(u16, u16)> {
        match range {
            RangeToken::StartWord => Some((self.x_pos, self.find_next_word())),
            _ => None,
        }
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

    pub fn yank_range(&mut self, range_start: usize, range_end: usize) {
        if let Some(slice) = self.text.get_slice(range_start..range_end) {
            self.clipboard
                .set_text(slice.as_str().unwrap_or_default().to_owned())
                .expect("Could not set value to system clipboard");
        }
    }

    pub fn paste_text(&mut self) {
        let coppied_text = self
            .clipboard
            .get_text()
            .expect("Could not set value to system clipboard");
        let char_idx = self.get_cursor_idx();
        self.past_states.push(self.text.clone());
        self.future_states = vec![];
        let _ = self.text.try_insert(char_idx, &coppied_text);
    }

    pub fn delete_line(&mut self) {
        self.future_states = vec![];
        self.past_states.push(self.text.clone());
        let _ = self
            .text
            .try_remove(self.start_of_current_line()..self.end_of_current_line());
        self.recenter();
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

    pub fn new(file_name: Option<String>) -> Result<Self, std::io::Error> {
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
                    id: Uuid::new_v4(),
                    title: file_path.clone(),
                    clipboard: Clipboard::new().unwrap(),
                    mode: Mode::Normal,
                    start_select_pos: None,
                    end_select_pos: None,
                    past_states: vec![],
                    future_states: vec![],
                    x_pos: 0,
                    y_pos: 0,
                    file_path: Some(file_path),
                    text: rope,
                    command_text: None,
                    operator: None::<OperatorToken>,
                    current_page: 0,
                    page_size: 10,
                })
            }
            None => Ok(Self {
                id: Uuid::new_v4(),
                title: "Ri".to_string(),
                clipboard: Clipboard::new().unwrap(),
                mode: Mode::Normal,
                start_select_pos: None,
                end_select_pos: None,
                past_states: vec![],
                future_states: vec![],
                x_pos: 0,
                y_pos: 0,
                file_path: None,
                text: Rope::new(),
                command_text: None,
                operator: None,
                current_page: 0,
                page_size: 10,
            }),
        }
    }

    fn standard_insert_response(&self) -> AnyHowResult<Vec<Token>> {
        Ok(vec![
            Token::Display(DisplayToken::CacheCurrentLine(
                self.id,
                self.text.clone(),
                self.y_pos as usize,
            )),
            Token::Display(DisplayToken::UpdateWindow(WindowChange {
                id: self.id,
                x_pos: self.x_pos,
                y_pos: self.y_pos,
                mode: self.mode.clone(),
                title: Some(self.title.clone()),
                page_size: self.page_size,
                current_page: self.current_page,
                ..WindowChange::default()
            })),
            Token::Display(DisplayToken::DrawViewPort),
        ])
    }

    fn standard_normal_response(&self) -> AnyHowResult<Vec<Token>> {
        Ok(vec![
            Token::Display(DisplayToken::UpdateWindow(WindowChange {
                id: self.id,
                x_pos: self.x_pos,
                y_pos: self.y_pos,
                mode: self.mode.clone(),
                title: Some(self.title.clone()),
                page_size: self.page_size,
                current_page: self.current_page,
                ..WindowChange::default()
            })),
            Token::Display(DisplayToken::DrawViewPort),
        ])
    }

        /*
    pub fn handle_visual_token(&mut self, token: Token) -> AnyHowResult<Vec<Token>> {
               match c {
                   'y' => {
                       self.mode = Mode::Normal;
                       if let Some((start_idx, end_idx)) = self.get_selected_range() {
                           if let Some(selected_text) = self.text.slice(start_idx..end_idx).as_str() {
                               self.clipboard
                                   .set_text(selected_text.to_owned())
                                   .expect("Could not set value to system clipboard");
                           }
                       }
                       self.start_select_pos = None;
                       self.end_select_pos = None;
                   }
                   'p' => {
                       self.mode = Mode::Normal;
                       if let Some((start_idx, end_idx)) = self.get_selected_range() {
                           let coppied_text = self
                               .clipboard
                               .get_text()
                               .expect("Could not set value to system clipboard");
                           self.past_states.push(self.text.clone());
                           self.future_states = vec![];
                           let _ = self.text.try_remove(start_idx..end_idx);
                           let _ = self.text.try_insert(start_idx, &coppied_text);
                       }
                   }
                   'd' => {
                       self.mode = Mode::Normal;
                       if let Some((start_idx, end_idx)) = self.get_selected_range() {
                           self.future_states = vec![];
                           self.past_states.push(self.text.clone());
                           let _ = self.text.try_remove(start_idx..end_idx);
                           self.recenter();
                       }
                       self.start_select_pos = None;
                       self.end_select_pos = None;
                   }
                   _ => (),
               }
        Ok(vec![])
    }
        */

        /*
    pub fn handle_operator_token(&mut self, token: OperatorToken) -> AnyHowResult<Vec<Token>> {
        if self.operator.is_none() {
            self.operator = Some(token);
            Ok(vec![])
        } else {
            Err(AnyHowError::msg("No Tokens Found".to_string()))
        }

        Err(AnyHowError::msg("No Tokens Found".to_string()))
    }
        */

        /*
    pub fn handle_range_token(&mut self, token: RangeToken) -> AnyHowResult<Vec<Token>> {
           if let Some(range_command) = RangeToken::parse(RangeToken::tokenize(c.to_string()))
               .unwrap_or_default()
            mut    .get(0)
           {
               if let Some((start_range, end_range)) = self.find_range(range_command) {
                   match operator {
                       OperatorToken::Yank => {
                           self.yank_range(start_range.into(), end_range.into())
                       }
                       _ => (),
                   }
               }
           }
           self.operator = None;

                   if self.mode == self::Mode::Command {
                       self.set_normal_mode();
                   }
        Ok(vec![])
    }
        */

}
