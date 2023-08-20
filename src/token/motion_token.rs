use anyhow::Error as AnyHowError;
use crossterm::event::{KeyCode, KeyEvent as Key};
use std::{convert::TryFrom, iter::Iterator};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum MotionToken {
    First,
    FirstNonBlank,
    Last,
    LastNonBlank,
    Left,
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
}

impl TryFrom<&[char]> for MotionToken {
    type Error = AnyHowError;
    fn try_from(value: &[char]) -> Result<Self, Self::Error> {
        match value {
            ['i', 'w', ..] => Ok(Self::InnerWord),
            ['0', ..] => Ok(Self::First),
            ['^', ..] => Ok(Self::FirstNonBlank),
            ['$', ..] => Ok(Self::Last),
            ['h', ..] => Ok(Self::Left),
            ['l', ..] => Ok(Self::Right),
            ['k', ..] => Ok(Self::Up),
            ['j', ..] => Ok(Self::Down),
            ['g', '_', ..] => Ok(Self::LastNonBlank),
            ['g', 'g', ..] => Ok(Self::FirstLine),
            ['G', ..] => Ok(Self::LastLine),
            ['w', ..] => Ok(Self::StartWord),
            ['e', ..] => Ok(Self::EndWord),
            ['b', ..] => Ok(Self::BackWord),
            ['f', rest @ ..] => Ok(Self::FindNext(rest.iter().collect::<String>())),
            ['F', rest @ ..] => Ok(Self::FindLast(rest.iter().collect::<String>())),
            ['t', rest @ ..] => Ok(Self::TillNext(rest.iter().collect::<String>())),
            ['T', rest @ ..] => Ok(Self::TillLast(rest.iter().collect::<String>())),
            _ => Err(Self::Error::msg(PARSE_FAILURE_ERR)),
        }
    }
}

pub const PARSE_FAILURE_ERR: &'static str = "Unknown Token";
impl TryFrom<&Vec<char>> for MotionToken {
    type Error = AnyHowError;

    fn try_from(value: &Vec<char>) -> Result<Self, Self::Error> {
        Self::try_from(&value[..])
    }
}

impl TryFrom<&Key> for MotionToken {
    type Error = AnyHowError;

    fn try_from(key: &Key) -> Result<Self, Self::Error> {
        match key.code {
            KeyCode::Up => Ok(Self::Up),
            KeyCode::Down => Ok(Self::Down),
            KeyCode::Left => Ok(Self::Left),
            KeyCode::Right => Ok(Self::Right),
            KeyCode::Backspace => Ok(Self::Left),
            _ => Err(Self::Error::msg(PARSE_FAILURE_ERR)),
        }
    }
}
