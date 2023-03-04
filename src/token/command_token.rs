use anyhow::Error as AnyHowError;
use std::{convert::TryFrom, iter::Iterator};
use crossterm::event::{KeyCode,KeyEvent as Key};
use uuid::Uuid;
use crate::{Token,app::Mode};
use actix::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub enum CommandToken {
    NoOp,
    SetMode(Mode),
    Quit,
    Write,
    TabNew,
    Append(String),
    Remove,
    VerticalSplit(Option<String>),
    Split(Option<String>),
    Tab,
    Esc,
    Enter,
    SetBuffer(Uuid),
    SetBufferWindow(Recipient<Token>)
}

pub const PARSE_FAILURE_ERR: &'static str = "Unknown Token";
impl TryFrom<&String> for CommandToken {
    type Error = AnyHowError;
    fn try_from(value: &String) -> Result<Self, Self::Error> {
        match &*(value.chars().collect::<Vec<char>>()) {
            [':', 'q', ..] => Ok(Self::Quit),
            [':', 'w', ..] => Ok(Self::Write),
            ['\n', ..] => Ok(Self::Enter),
            [':', 'v', 's', rest @ ..] => {
                Ok(Self::VerticalSplit(Some(rest.iter().collect::<String>())))
            }
            [':', 's', 'p', rest @ ..] => Ok(Self::Split(Some(rest.iter().collect::<String>()))),
            [rest @ ..] => Ok(Self::Append(rest.iter().collect::<String>())),
        }
    }
}

impl TryFrom<&Key> for CommandToken {
    type Error = AnyHowError;

    fn try_from(key: &Key) -> Result<Self, Self::Error> {
        match key.code {
            KeyCode::Esc => Ok(Self::Esc),
            KeyCode::Backspace => Ok(Self::Remove),
            _ => Err(Self::Error::msg(PARSE_FAILURE_ERR)),
        }
    }
}
