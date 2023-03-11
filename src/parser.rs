use crate::app::Mode;
use crate::token::{get_token_from_key, get_token_from_str, Token};
use anyhow::{Error as AnyHowError, Result as AnyHowResult};
use crossterm::event::{KeyCode, KeyEvent as Key};

pub struct Parser {
}


pub struct UserInput {
    pub event: Key,
}

impl Parser {
    pub fn handle_event(msg: UserInput, mode: &Mode) -> AnyHowResult<Token> {
        let event = msg.event;
        let mut token_str = String::new();
        log::debug!("current mode {:?}", mode);
        if let Ok(token) = get_token_from_key(mode, &event) {
            return Ok(token);
        } else if let KeyCode::Char(c) = event.code {
            token_str.push_str(&c.to_string());
            if let Ok(token) = get_token_from_str(mode, &token_str) {
                token_str.truncate(0);
                return Ok(token);
            }
        }
        Err(AnyHowError::msg("No Tokens Found".to_string()))
    }
}
