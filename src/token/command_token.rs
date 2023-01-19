use crate::util::event::Event;
use anyhow::Error as AnyHowError;
use std::{convert::TryFrom, iter::Iterator};
use termion::event::Key;
use uuid::Uuid;
use actix::prelude::*;

#[derive(Message)]
#[rtype(result = "()")]
pub struct Quit {}


#[derive(Message)]
#[rtype(result = "()")]
pub struct TabNew {}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Write {}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Enter {}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Esc {}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Remove {}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Append {
    pub chars: String
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Split {
    pub f_name: Option<String>
}


#[derive(Message)]
#[rtype(result = "()")]
pub struct SetBuffer {
    pub id: Uuid
}


#[derive(Message)]
#[rtype(result = "()")]
pub struct VerticalSplit {
    pub f_name: Option<String>
}

#[derive(Clone, Debug, PartialEq)]
pub enum CommandToken {
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

impl TryFrom<&Event<Key>> for CommandToken {
    type Error = AnyHowError;

    fn try_from(key: &Event<Key>) -> Result<Self, Self::Error> {
        match key {
            Event::Input(Key::Esc) => Ok(Self::Esc),
            Event::Input(Key::Backspace) => Ok(Self::Remove),
            _ => Err(Self::Error::msg(PARSE_FAILURE_ERR)),
        }
    }
}
