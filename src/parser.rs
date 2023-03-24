use crate::app::Mode;
use crate::token::{get_token_from_key, get_tokens_from_chars, Token};
use crossterm::event::{KeyCode, KeyEvent as Key};

pub struct Parser {
    pub token_str: Vec<char>,
}

pub struct UserInput {
    pub event: Key,
}

impl Parser {
    pub fn new() -> Self {
        Self { token_str: vec![] }
    }

    pub fn handle_event(&mut self, msg: UserInput, mode: &Mode) -> Vec<Token> {
        let event = msg.event;
        if let Ok(token) = get_token_from_key(mode, &event) {
            return vec![token];
        } else if let KeyCode::Char(c) = event.code {
            self.token_str.push(c);
            let tokens = get_tokens_from_chars(mode, &self.token_str);
            if !tokens.is_empty() {
                self.token_str.truncate(0);
                return tokens;
            }
        }
        vec![]
    }
}
