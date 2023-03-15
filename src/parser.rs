use crate::app::Mode;
use crate::token::{get_token_from_key, get_token_from_str, Token};
use anyhow::{Error as AnyHowError, Result as AnyHowResult};
use crossterm::event::{KeyCode, KeyEvent as Key};

pub struct Parser {
    pub token_str: String
}


pub struct UserInput {
    pub event: Key,
}

impl Parser {
    pub fn new () -> Self {
        Self {
            token_str: String::new()
        }
    }

    pub fn handle_event(&mut self,msg: UserInput, mode: &Mode) -> AnyHowResult<Token> {
        let event = msg.event;
        if let Ok(token) = get_token_from_key(mode, &event) {
            return Ok(token);
        } else if let KeyCode::Char(c) = event.code {
            self.token_str.push_str(&c.to_string());
            if let Ok(token) = get_token_from_str(mode, &self.token_str) {
                self.token_str.truncate(0);
                return Ok(token);
            }
        }
        Err(AnyHowError::msg("No Tokens Found".to_string()))
    }
}
