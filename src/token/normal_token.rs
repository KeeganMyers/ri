use anyhow::Error as AnyHowError;
use crossterm::event::{KeyCode, KeyEvent as Key};
use std::{convert::TryFrom, iter::Iterator};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum NormalToken {
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
    YankLine,
    DeleteLine,
    Visual,
    VisualLine,
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
            ['y', 'y', ..] => Ok(Self::YankLine),
            ['d', 'd', ..] => Ok(Self::DeleteLine),
            ['u', ..] => Ok(Self::Undo),
            ['r', ..] => Ok(Self::Redo),
            ['o', ..] => Ok(Self::AddNewLineBelow),
            ['O', ..] => Ok(Self::AddNewLineAbove),
            ['p', ..] => Ok(Self::Paste),
            ['i', ..] => Ok(Self::SwitchToInsert),
            ['v', ..] => Ok(Self::Visual),
            ['V', ..] => Ok(Self::VisualLine),
            ['\n', ..] => Ok(Self::Enter),
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
            KeyCode::Esc => Ok(Self::Esc),
            _ => Err(Self::Error::msg(PARSE_FAILURE_ERR)),
        }
    }
}
