use anyhow::Error as AnyHowError;
use crossterm::event::{KeyCode, KeyEvent as Key};
use std::{convert::TryFrom, iter::Iterator};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum NormalToken {
    First,
    FirstNonBlank,
    Last,
    LastNonBlank,
    Left,
    Right,
    Up,
    Down,
    WindowLeft,
    WindowRight,
    WindowUp,
    WindowDown,
    SwitchToInsert,
    SwitchToAppend,
    AddNewLineBelow,
    AddNewLineAbove,
    Paste,
    Undo,
    Redo,
    DeleteLine,
    Visual,
    VisualLine,
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
    Enter,
    SwitchToCommand,
    SetWindow(usize),
}

impl TryFrom<&[char]> for NormalToken {
    type Error = AnyHowError;
    fn try_from(value: &[char]) -> Result<Self, Self::Error> {
        match value {
            ['a', ..] => Ok(Self::SwitchToAppend),
            [':', ..] => Ok(Self::SwitchToCommand),
            ['d', 'd', ..] => Ok(Self::DeleteLine),
            ['u', ..] => Ok(Self::Undo),
            ['r', ..] => Ok(Self::Redo),
            ['o', ..] => Ok(Self::AddNewLineBelow),
            ['O', ..] => Ok(Self::AddNewLineAbove),
            ['p', ..] => Ok(Self::Paste),
            ['i', 'w', ..] => Ok(Self::InnerWord),
            ['i', ..] => Ok(Self::SwitchToInsert),
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
            ['v', ..] => Ok(Self::Visual),
            ['V', ..] => Ok(Self::VisualLine),
            ['\n', ..] => Ok(Self::Enter),
            ['f', rest @ ..] => Ok(Self::FindNext(rest.iter().collect::<String>())),
            ['z', rest @ ..]
                if rest
                    .iter()
                    .collect::<String>()
                    .trim()
                    .parse::<usize>()
                    .is_ok() =>
            {
                Ok(Self::SetWindow(
                    rest.iter()
                        .collect::<String>()
                        .trim()
                        .parse::<usize>()
                        .unwrap_or_default(),
                ))
            }
            ['F', rest @ ..] => Ok(Self::FindLast(rest.iter().collect::<String>())),
            ['t', rest @ ..] => Ok(Self::TillNext(rest.iter().collect::<String>())),
            ['T', rest @ ..] => Ok(Self::TillLast(rest.iter().collect::<String>())),
            _ => Err(Self::Error::msg(PARSE_FAILURE_ERR)),
        }
    }
}

pub const PARSE_FAILURE_ERR: &'static str = "Unknown Token";
impl TryFrom<&Vec<char>> for NormalToken {
    type Error = AnyHowError;

    fn try_from(value: &Vec<char>) -> Result<Self, Self::Error> {
        Self::try_from(&value[..])
    }
}

impl TryFrom<&Key> for NormalToken {
    type Error = AnyHowError;

    fn try_from(key: &Key) -> Result<Self, Self::Error> {
        match key.code {
            KeyCode::Up => Ok(Self::Up),
            KeyCode::Down => Ok(Self::Down),
            KeyCode::Left => Ok(Self::Left),
            KeyCode::Right => Ok(Self::Right),
            KeyCode::Esc => Ok(Self::Esc),
            KeyCode::Backspace => Ok(Self::Left),
            _ => Err(Self::Error::msg(PARSE_FAILURE_ERR)),
        }
    }
}
