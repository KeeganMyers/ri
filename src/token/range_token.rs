use anyhow::Error as AnyHowError;
use crossterm::event::{KeyCode, KeyEvent as Key};
use std::{convert::TryFrom, iter::Iterator};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum RangeToken {
    First,
    FirstNonBlank,
    Left,
    Last,
    LastNonBlank,
    Right,
    Up,
    Down,
    FindNext(String),
    FindLast(String),
    TillNext(String),
    TillLast(String),
    LastLine,
    FirstLine,
    StartWord,
    EndWord,
    BackWord,
    InnerWord,
    Esc,
    Remove,
    Enter,
}

impl TryFrom<&[char]> for RangeToken {
    type Error = AnyHowError;

    fn try_from(value: &[char]) -> Result<Self, Self::Error> {
        match value {
            ['^', ..] => Ok(Self::FirstNonBlank),
            ['$', ..] => Ok(Self::Last),
            ['h', ..] => Ok(Self::Left),
            ['l', ..] => Ok(Self::Right),
            ['j', ..] => Ok(Self::Up),
            ['k', ..] => Ok(Self::Down),
            ['g', '_', ..] => Ok(Self::LastNonBlank),
            ['g', 'g', ..] => Ok(Self::LastLine),
            ['G', ..] => Ok(Self::FirstLine),
            ['w', ..] => Ok(Self::StartWord),
            ['e', ..] => Ok(Self::EndWord),
            ['b', ..] => Ok(Self::BackWord),
            ['i', 'w', ..] => Ok(Self::InnerWord),
            ['\n', ..] => Ok(Self::Enter),
            ['f', rest @ ..] => Ok(Self::FindNext(rest.iter().collect::<String>())),
            ['F', rest @ ..] => Ok(Self::FindLast(rest.iter().collect::<String>())),
            ['t', rest @ ..] => Ok(Self::TillNext(rest.iter().collect::<String>())),
            ['T', rest @ ..] => Ok(Self::TillLast(rest.iter().collect::<String>())),
            _ => Err(Self::Error::msg(PARSE_FAILURE_ERR)),
        }
    }
}

pub const PARSE_FAILURE_ERR: &'static str = "Unknown Token";
impl TryFrom<&Vec<char>> for RangeToken {
    type Error = AnyHowError;

    fn try_from(value: &Vec<char>) -> Result<Self, Self::Error> {
        Self::try_from(&value[..])
    }
}

impl TryFrom<&Key> for RangeToken {
    type Error = AnyHowError;

    fn try_from(key: &Key) -> Result<Self, Self::Error> {
        match key.code {
            KeyCode::Esc => Ok(Self::Esc),
            KeyCode::Backspace => Ok(Self::Remove),
            _ => Err(Self::Error::msg(PARSE_FAILURE_ERR)),
        }
    }
}
