use actix::prelude::*;
use crate::util::event::{Config, Event, Events};
use anyhow::{Error as AnyHowError, Result as AnyHowResult};
use crate::app::Mode;
use crate::{
    token::{
        command_token::*,
        display_token::{DisplayToken, WindowChange},
        get_token_from_key, get_token_from_str, CommandToken, Token,
    },
    Buffer,
};
use termion::event::Key;

pub struct Parser {
    mode: Mode
}

impl Actor for Parser {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct UserInput {
    event:  Event<Key>
}

impl Handler<UserInput> for Parser {
    type Result = ();

    fn handle(&mut self, msg: UserInput , _ctx: &mut Context<Self>) {
                let event = msg.event;
                let mut token_str = String::new();
                if let Ok(token) = get_token_from_key(&self.mode, &event) {
                    println!("{:?}", token);
                    //handle event
                } else if let Event::Input(Key::Char(c)) = event {
                    token_str.push_str(&c.to_string());
                    if let Ok(token) = get_token_from_str(&self.mode, &token_str) {
                        //app_events.push(token.clone());
                        //draw_events.push(token.clone());
                        token_str.truncate(0);
                        //handle_event
                    }
                }
    }
}
