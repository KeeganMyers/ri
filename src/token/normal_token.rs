use crate::util::event::Event;
use anyhow::Error as AnyHowError;
use std::{convert::TryFrom, iter::Iterator};
use termion::event::Key;

#[derive(Clone, Debug, PartialEq)]
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
}

pub const PARSE_FAILURE_ERR: &'static str = "Unknown Token";
impl TryFrom<&String> for NormalToken {
    type Error = AnyHowError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        match &*(value.chars().collect::<Vec<char>>()) {
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
            ['z', 'h', ..] => Ok(Self::WindowLeft),
            ['l', ..] => Ok(Self::Right),
            ['z', 'l', ..] => Ok(Self::WindowRight),
            ['j', ..] => Ok(Self::Up),
            ['z', 'k', ..] => Ok(Self::WindowUp),
            ['k', ..] => Ok(Self::Down),
            ['z', 'j', ..] => Ok(Self::WindowDown),
            ['g', '_', ..] => Ok(Self::LastNonBlank),
            ['g', 'g', ..] => Ok(Self::LastLine),
            ['G', ..] => Ok(Self::FirstLine),
            ['w', ..] => Ok(Self::StartWord),
            ['e', ..] => Ok(Self::EndWord),
            ['b', ..] => Ok(Self::BackWord),
            ['v', ..] => Ok(Self::Visual),
            ['V', ..] => Ok(Self::VisualLine),
            ['\n', ..] => Ok(Self::Enter),
            ['f', rest @ ..] => Ok(Self::FindNext(rest.iter().collect::<String>())),
            ['F', rest @ ..] => Ok(Self::FindLast(rest.iter().collect::<String>())),
            ['t', rest @ ..] => Ok(Self::TillNext(rest.iter().collect::<String>())),
            ['T', rest @ ..] => Ok(Self::TillLast(rest.iter().collect::<String>())),
            _ => Err(Self::Error::msg(PARSE_FAILURE_ERR)),
        }
    }
}

impl TryFrom<&Event<Key>> for NormalToken {
    type Error = AnyHowError;

    fn try_from(key: &Event<Key>) -> Result<Self, Self::Error> {
        match key {
            Event::Input(Key::Up) => Ok(Self::Up),
            Event::Input(Key::Down) => Ok(Self::Down),
            Event::Input(Key::Left) => Ok(Self::Left),
            Event::Input(Key::Right) => Ok(Self::Right),
            Event::Input(Key::Esc) => Ok(Self::Esc),
            Event::Input(Key::Backspace) => Ok(Self::Left),
            _ => Err(Self::Error::msg(PARSE_FAILURE_ERR)),
        }
    }
}
