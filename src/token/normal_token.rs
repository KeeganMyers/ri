use crate::util::event::Event;
use anyhow::Error as AnyHowError;
use std::{convert::TryFrom, iter::Iterator};
use termion::event::Key;
use actix::prelude::*;

#[derive(Message,Clone,Debug,PartialEq)]
#[rtype(result = "()")]
pub struct WindowLeft {}

#[derive(Message,Clone,Debug,PartialEq)]
#[rtype(result = "()")]
pub struct WindowRight {}

#[derive(Message,Clone,Debug,PartialEq)]
#[rtype(result = "()")]
pub struct WindowUp {}

#[derive(Message,Clone,Debug,PartialEq)]
#[rtype(result = "()")]
pub struct WindowDown {}

#[derive(Message,Clone,Debug,PartialEq)]
#[rtype(result = "()")]
pub struct Up {}

#[derive(Message,Clone,Debug,PartialEq)]
#[rtype(result = "()")]
pub struct First {}

#[derive(Message,Clone,Debug,PartialEq)]
#[rtype(result = "()")]
pub struct FirstNonBlank {}

#[derive(Message,Clone,Debug,PartialEq)]
#[rtype(result = "()")]
pub struct Last {}

#[derive(Message,Clone,Debug,PartialEq)]
#[rtype(result = "()")]
pub struct LastNonBlank {}

#[derive(Message,Clone,Debug,PartialEq)]
#[rtype(result = "()")]
pub struct Left {}

#[derive(Message,Clone,Debug,PartialEq)]
#[rtype(result = "()")]
pub struct Right {}

#[derive(Message,Clone,Debug,PartialEq)]
#[rtype(result = "()")]
pub struct Down {}

#[derive(Message,Clone,Debug,PartialEq)]
#[rtype(result = "()")]
pub struct SwitchToInsert {}

#[derive(Message,Clone,Debug,PartialEq)]
#[rtype(result = "()")]
pub struct SwitchToAppend {}

#[derive(Message,Clone,Debug,PartialEq)]
#[rtype(result = "()")]
pub struct AddNewLineBelow {}

#[derive(Message,Clone,Debug,PartialEq)]
#[rtype(result = "()")]
pub struct AddNewLineAbove {}

#[derive(Message,Clone,Debug,PartialEq)]
#[rtype(result = "()")]
pub struct Paste {}

#[derive(Message,Clone,Debug,PartialEq)]
#[rtype(result = "()")]
pub struct Undo {}

#[derive(Message,Clone,Debug,PartialEq)]
#[rtype(result = "()")]
pub struct Redo {}

#[derive(Message,Clone,Debug,PartialEq)]
#[rtype(result = "()")]
pub struct DeleteLine {}

#[derive(Message,Clone,Debug,PartialEq)]
#[rtype(result = "()")]
pub struct Visual {}

#[derive(Message,Clone,Debug,PartialEq)]
#[rtype(result = "()")]
pub struct VisualLine {}

#[derive(Message,Clone,Debug,PartialEq)]
#[rtype(result = "()")]
pub struct FindNext {
    pub chars: String
}

#[derive(Message,Clone,Debug,PartialEq)]
#[rtype(result = "()")]
pub struct FindLast {
    pub chars: String
}


#[derive(Message,Clone,Debug,PartialEq)]
#[rtype(result = "()")]
pub struct TillNext {
    pub chars: String
}

#[derive(Message,Clone,Debug,PartialEq)]
#[rtype(result = "()")]
pub struct LastLine {}

#[derive(Message,Clone,Debug,PartialEq)]
#[rtype(result = "()")]
pub struct FirstLine {}

#[derive(Message,Clone,Debug,PartialEq)]
#[rtype(result = "()")]
pub struct StartWord {}

#[derive(Message,Clone,Debug,PartialEq)]
#[rtype(result = "()")]
pub struct EndWord {}

#[derive(Message,Clone,Debug,PartialEq)]
#[rtype(result = "()")]
pub struct BackWord {}

#[derive(Message,Clone,Debug,PartialEq)]
#[rtype(result = "()")]
pub struct InnerWord {}

#[derive(Message,Clone,Debug,PartialEq)]
#[rtype(result = "()")]
pub struct Esc {}

#[derive(Message,Clone,Debug,PartialEq)]
#[rtype(result = "()")]
pub struct Enter {}

#[derive(Message,Clone,Debug,PartialEq)]
#[rtype(result = "()")]
pub struct SwitchToCommand {}

#[derive(Message,Clone, Debug,PartialEq)]
#[rtype(result = "()")]
pub enum NormalToken {
    First,
    FirstNonBlank,
    Last,
    LastNonBlank,
    Left(Left),
    Right(Right),
    Up(Up),
    Down(Down),
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
    Esc(Esc),
    Enter,
    SwitchToCommand,
}

pub const PARSE_FAILURE_ERR: &'static str = "Unknown Token";
impl TryFrom<&String> for NormalToken {
    type Error = AnyHowError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        /*
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
        */
        Err(Self::Error::msg(PARSE_FAILURE_ERR))
    }
}

impl TryFrom<&Event<Key>> for NormalToken {
    type Error = AnyHowError;

    fn try_from(key: &Event<Key>) -> Result<Self, Self::Error> {
        match key {
            Event::Input(Key::Up) => Ok(Self::Up(Up {})),
            Event::Input(Key::Down) => Ok(Self::Down(Down {})),
            Event::Input(Key::Left) => Ok(Self::Left( Left{})),
            Event::Input(Key::Right) => Ok(Self::Right(Right {})),
            Event::Input(Key::Esc) => Ok(Self::Esc( Esc {})),
            Event::Input(Key::Backspace) => Ok(Self::Left( Left {})),
            _ => Err(Self::Error::msg(PARSE_FAILURE_ERR)),
        }
    }
}
