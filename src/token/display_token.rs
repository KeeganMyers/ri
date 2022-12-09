use crate::app::Mode;
use crate::util::event::Event;
use anyhow::Error as AnyHowError;
use ropey::Rope;
use std::{convert::TryFrom, iter::Iterator};
use termion::event::Key;
use uuid::Uuid;
use tui::layout::Direction;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct WindowChange {
    pub id: Uuid,
    pub x_pos: u16,
    pub y_pos: u16,
    pub command_text: Option<String>,
    pub mode: Mode,
    pub title: Option<String>,
    pub page_size: u16,
    pub current_page: u16,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DisplayToken {
    SetHighlight,
    UpdateWindow(WindowChange),
    NewVerticalWindow(WindowChange),
    NewHorizontalWindow(WindowChange),
    DropWindow(Uuid),
    DrawViewPort,
    SetTextLayout(Direction),
    DrawWindow(Uuid),
    CacheWindowContent(Uuid, Rope),
    AppendCommand(Uuid, Option<String>),
    CacheCurrentLine(Uuid, Rope, usize),
    CacheNewLine(Uuid, Rope, usize),
    RemoveCacheLine(Uuid, Rope, usize),
}

pub const PARSE_FAILURE_ERR: &'static str = "Unknown Token";
impl TryFrom<&String> for DisplayToken {
    type Error = AnyHowError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        match &*(value.chars().collect::<Vec<char>>()) {
            _ => Err(Self::Error::msg(PARSE_FAILURE_ERR)),
        }
    }
}

impl TryFrom<&Event<Key>> for DisplayToken {
    type Error = AnyHowError;

    fn try_from(key: &Event<Key>) -> Result<Self, Self::Error> {
        match key {
            _ => Err(Self::Error::msg(PARSE_FAILURE_ERR)),
        }
    }
}
