use anyhow::Error as AnyHowError;
use std::{convert::TryFrom, iter::Iterator};
use termion::event::Key;

#[derive(Clone, Debug, PartialEq)]
pub enum InsertToken {
    Append(String),
    Esc,
    Enter,
    Remove,
}

pub const PARSE_FAILURE_ERR: &'static str = "Unknown Token";
impl TryFrom<&String> for InsertToken {
    type Error = AnyHowError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        match &*(value.chars().collect::<Vec<char>>()) {
            ['\n', ..] => Ok(Self::Enter),
            [rest @ ..] => Ok(Self::Append(rest.iter().collect::<String>())),
        }
    }
}

impl TryFrom<&Key> for InsertToken {
    type Error = AnyHowError;

    fn try_from(key: &Key) -> Result<Self, Self::Error> {
        match key {
            Key::Esc => Ok(Self::Esc),
            Key::Backspace => Ok(Self::Remove),
            _ => Err(Self::Error::msg(PARSE_FAILURE_ERR)),
        }
    }
}
