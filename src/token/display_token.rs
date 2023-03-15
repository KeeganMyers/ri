use anyhow::Error as AnyHowError;
use crossterm::event::KeyEvent as Key;
use ropey::Rope;
use std::{convert::TryFrom, iter::Iterator};
use tui::layout::Direction;
use uuid::Uuid;
use tui::layout::Rect;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct WindowChange {
    pub id: Uuid,
    pub x_pos: u16,
    pub y_pos: u16,
    pub title: Option<String>,
    pub page_size: u16,
    pub current_page: u16,
    pub area: Option<Rect>
}

#[derive(Clone)]
pub enum DisplayToken {
    SetHighlight,
    UpdateWindow(WindowChange),
    NewWindow(WindowChange, Option<Direction>),
    DrawViewPort,
    DrawWindow,
    SetTextLayout(Direction),
    CacheWindowContent(Rope),
    AppendCommand(Option<String>),
    CacheCurrentLine(Rope, usize),
    CacheNewLine(Rope, usize),
    RemoveCacheLine(Rope, usize),
    CloseWindow(Uuid),
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

impl TryFrom<&Key> for DisplayToken {
    type Error = AnyHowError;

    fn try_from(key: &Key) -> Result<Self, Self::Error> {
        match key {
            _ => Err(Self::Error::msg(PARSE_FAILURE_ERR)),
        }
    }
}
