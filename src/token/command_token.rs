use crate::{app::Mode};
use anyhow::Error as AnyHowError;
use crossterm::event::{KeyCode, KeyEvent as Key};
use std::{convert::TryFrom, iter::Iterator};
use uuid::Uuid;

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
    GoToLine(usize)
}

pub const PARSE_FAILURE_ERR: &'static str = "Unknown Token";
impl TryFrom<&String> for CommandToken {
    type Error = AnyHowError;
    fn try_from(value: &String) -> Result<Self, Self::Error> {
        let command_slice = &*(value.chars().collect::<Vec<char>>());
        let command_token = match command_slice {
            [':', 'q', ..] => Ok(Self::Quit),
            [':', 'w', ..] => Ok(Self::Write),
            ['\n', ..] => Ok(Self::Enter),
            [':', 'v', 's', rest @ ..] => Ok(Self::VerticalSplit(Some(rest.iter().collect::<String>()))),
            [':', 's', 'p', rest @ ..] => Ok(Self::Split(Some(rest.iter().collect::<String>()))),
            [':',rest @ ..] if rest.iter().collect::<String>().trim().parse::<usize>().is_ok() => Ok(Self::GoToLine(rest.iter().collect::<String>().trim().parse::<usize>().unwrap_or_default())),
            [rest @ ..] => Ok(Self::Append(rest.iter().collect::<String>())),
        };
        if command_token.is_err() &&value.starts_with(":") && value.contains(',') {

        }
        command_token
    }
}

impl TryFrom<&Key> for CommandToken {
    type Error = AnyHowError;

    fn try_from(key: &Key) -> Result<Self, Self::Error> {
        match key.code {
            KeyCode::Enter => Ok(Self::Enter),
            KeyCode::Esc => Ok(Self::Esc),
            KeyCode::Backspace => Ok(Self::Remove),
            _ => Err(Self::Error::msg(PARSE_FAILURE_ERR)),
        }
    }
}
