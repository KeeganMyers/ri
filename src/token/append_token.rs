use anyhow::Error as AnyHowError;
use crossterm::event::{KeyCode, KeyEvent as Key};
use std::{convert::TryFrom, iter::Iterator};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum AppendToken {
    Append(String),
    Esc,
    Enter,
    Remove,
}

pub const PARSE_FAILURE_ERR: &'static str = "Unknown Token";

impl TryFrom<&[char]> for AppendToken {
    type Error = AnyHowError;

    fn try_from(value: &[char]) -> Result<Self, Self::Error> {
        match value {
            ['\n', ..] => Ok(Self::Enter),
            [rest @ ..] => Ok(Self::Append(rest.iter().collect::<String>())),
        }
    }
}

impl TryFrom<&Vec<char>> for AppendToken {
    type Error = AnyHowError;

    fn try_from(value: &Vec<char>) -> Result<Self, Self::Error> {
        Self::try_from(&value[..])
    }
}

impl TryFrom<&Key> for AppendToken {
    type Error = AnyHowError;

    fn try_from(key: &Key) -> Result<Self, Self::Error> {
        match key.code {
            KeyCode::Esc => Ok(Self::Esc),
            KeyCode::Backspace => Ok(Self::Remove),
            _ => Err(Self::Error::msg(PARSE_FAILURE_ERR)),
        }
    }
}
