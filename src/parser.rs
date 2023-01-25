use actix::prelude::*;
use anyhow::{Error as AnyHowError, Result as AnyHowResult};
use crate::app::Mode;
use crate::{
    token::{
        get_token_from_key, get_token_from_str, Token,
    },
};
use termion::event::Key;

pub struct Parser {
    pub mode: Mode
}

impl Actor for Parser {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = " AnyHowResult<Token>")]
pub struct UserInput {
    pub event:  Key
}

impl Handler<UserInput> for Parser {
    type Result = AnyHowResult<Token>;

    fn handle(&mut self, msg: UserInput , _ctx: &mut Context<Self>) -> Self::Result {
                let event = msg.event;
                let mut token_str = String::new();
                if let Ok(token) = get_token_from_key(&self.mode, &event) {
                    return Ok(token);
                } else if let Key::Char(c) = event {
                    token_str.push_str(&c.to_string());
                    if let Ok(token) = get_token_from_str(&self.mode, &token_str) {
                        token_str.truncate(0);
                        return Ok(token);
                    }
                }
        Err(AnyHowError::msg("No Tokens Found".to_string()))
    }
}
