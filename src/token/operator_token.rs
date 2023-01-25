use anyhow::Error as AnyHowError;
use std::{convert::TryFrom, iter::Iterator};
use termion::event::Key;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum OperatorToken {
    Yank,
    Delete,
    Change,
    Indent,
    UnIndent,
    Uppercase,
    Lowercase,
    ToggleCase,
    Shell,
    Format,
    Esc,
    Remove,
    Enter,
}

pub const PARSE_FAILURE_ERR: &'static str = "Unknown Token";
impl TryFrom<&String> for OperatorToken {
    type Error = AnyHowError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        match &*(value.chars().collect::<Vec<char>>()) {
            ['y', ..] => Ok(Self::Yank),
            ['d', ..] => Ok(Self::Delete),
            ['c', ..] => Ok(Self::Change),
            ['>', ..] => Ok(Self::Indent),
            ['<', ..] => Ok(Self::UnIndent),
            ['g', 'U', ..] => Ok(Self::Uppercase),
            ['g', 'u', ..] => Ok(Self::Lowercase),
            ['~', ..] => Ok(Self::ToggleCase),
            ['!', ..] => Ok(Self::Shell),
            ['=', ..] => Ok(Self::Format),
            ['\n', ..] => Ok(Self::Enter),
            _ => Err(Self::Error::msg(PARSE_FAILURE_ERR)),
        }
    }
}

impl TryFrom<&Key> for OperatorToken {
    type Error = AnyHowError;

    fn try_from(key: &Key) -> Result<Self, Self::Error> {
        match key {
            Key::Esc => Ok(Self::Esc),
            Key::Backspace => Ok(Self::Remove),
            _ => Err(Self::Error::msg(PARSE_FAILURE_ERR)),
        }
    }
}
