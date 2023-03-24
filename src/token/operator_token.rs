use anyhow::Error as AnyHowError;
use crossterm::event::{KeyCode, KeyEvent as Key};
use std::convert::TryFrom;

#[derive(Clone, PartialEq, Eq, Debug)]
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
}

impl TryFrom<&[char]> for OperatorToken {
    type Error = AnyHowError;

    fn try_from(value: &[char]) -> Result<Self, Self::Error> {
        match value {
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
            _ => Err(Self::Error::msg(PARSE_FAILURE_ERR)),
        }
    }
}

pub const PARSE_FAILURE_ERR: &'static str = "Unknown Token";
impl TryFrom<&Vec<char>> for OperatorToken {
    type Error = AnyHowError;

    fn try_from(value: &Vec<char>) -> Result<Self, Self::Error> {
        Self::try_from(&value[..])
    }
}

impl TryFrom<&Key> for OperatorToken {
    type Error = AnyHowError;

    fn try_from(key: &Key) -> Result<Self, Self::Error> {
        match key.code {
            KeyCode::Esc => Ok(Self::Esc),
            KeyCode::Backspace => Ok(Self::Remove),
            _ => Err(Self::Error::msg(PARSE_FAILURE_ERR)),
        }
    }
}
