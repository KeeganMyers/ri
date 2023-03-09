use crate::app::Mode;
use crate::token::{get_token_from_key, get_token_from_str, Token, CommandToken};
use actix::prelude::*;
use anyhow::{Error as AnyHowError, Result as AnyHowResult};
use crossterm::event::{KeyCode, KeyEvent as Key};

pub struct Parser {
    pub mode: Mode,
}

impl Actor for Parser {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = " AnyHowResult<Token>")]
pub struct UserInput {
    pub event: Key,
}

impl Handler<UserInput> for Parser {
    type Result = AnyHowResult<Token>;

    fn handle(&mut self, msg: UserInput, _ctx: &mut Context<Self>) -> Self::Result {
        let event = msg.event;
        let mut token_str = String::new();
        log::debug!("current mode {:?}", self.mode);
        if let Ok(token) = get_token_from_key(&self.mode, &event) {
            return Ok(token);
        } else if let KeyCode::Char(c) = event.code {
            token_str.push_str(&c.to_string());
            if let Ok(token) = get_token_from_str(&self.mode, &token_str) {
                token_str.truncate(0);
                return Ok(token);
            }
        }
        Err(AnyHowError::msg("No Tokens Found".to_string()))
    }
}


impl Handler<Token> for Parser {
    type Result = ();

    fn handle(&mut self, msg: Token, _ctx: &mut Context<Self>) -> Self::Result {
        log::debug!("parser setting mode");
        match msg {
            Token::Command(CommandToken::SetMode(mode)) => {
                self.mode = mode
            },
            _ => ()
        }
    }
}
