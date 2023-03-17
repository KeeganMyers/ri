use crate::{app::Mode};
use anyhow::Error as AnyHowError;
use crossterm::event::{KeyCode, KeyEvent as Key};
use std::{convert::TryFrom, iter::Iterator};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq)]
pub enum CommandToken {
    NoOp,
    SetMode(Mode),
    Quit,
    Write,
    TabNew,
    Append(String),
    Remove,
    VerticalSplit(Option<String>),
    Split(Option<String>),
    Tab,
    Esc,
    Enter,
    SetBuffer(Uuid),
    GoToLine(usize),
    YankLines(usize,usize),
    DeleteLines(usize,usize)
}

pub const PARSE_FAILURE_ERR: &'static str = "Unknown Token";
impl TryFrom<&String> for CommandToken {
    type Error = AnyHowError;
    fn try_from(value: &String) -> Result<Self, Self::Error> {
        let command_slice = &*(value.chars().collect::<Vec<char>>());
        let command_token = match command_slice {
            [':', 'q', ..] => Ok(Self::Quit),
            [':', 'w', ..] => Ok(Self::Write),
            ['\n', ..] => Ok(Self::Enter),
            [':', 'v', 's', rest @ ..] => Ok(Self::VerticalSplit(Some(rest.iter().collect::<String>()))),
            [':', 's', 'p', rest @ ..] => Ok(Self::Split(Some(rest.iter().collect::<String>()))),
            [':',rest @ ..] if rest.iter().collect::<String>().trim().parse::<usize>().is_ok() => Ok(Self::GoToLine(rest.iter().collect::<String>().trim().parse::<usize>().unwrap_or_default())),
             _ => Err(Self::Error::msg(PARSE_FAILURE_ERR)),
        };

        if command_token.is_err() && value.starts_with(":") && value.contains(',') && (value.ends_with("d") || value.ends_with("y")) {
           let last_char = value.chars().nth(value.len() -1).unwrap();
           let command_vec = value.trim_start_matches(":").trim_end_matches("d").trim_end_matches("y").split(',').collect::<Vec<&str>>();
           return match  &command_vec[..] {
               [start_index,end_index] if last_char == 'y' && start_index.parse::<usize>().is_ok() && end_index.parse::<usize>().is_ok()  => return Ok(Self::YankLines(start_index.parse::<usize>().unwrap(),end_index.parse::<usize>().unwrap())),
               [start_index,end_index] if last_char == 'd' && start_index.parse::<usize>().is_ok() && end_index.parse::<usize>().is_ok()  => return Ok(Self::DeleteLines(start_index.parse::<usize>().unwrap(),end_index.parse::<usize>().unwrap())),
             _ => Err(Self::Error::msg(PARSE_FAILURE_ERR)),
           };
        } else if command_token.is_err() {
            return Ok(Self::Append(value.clone()));
        }
        command_token
    }
}

impl TryFrom<&Key> for CommandToken {
    type Error = AnyHowError;

    fn try_from(key: &Key) -> Result<Self, Self::Error> {
        match key.code {
            KeyCode::Enter => Ok(Self::Enter),
            KeyCode::Esc => Ok(Self::Esc),
            KeyCode::Backspace => Ok(Self::Remove),
            _ => Err(Self::Error::msg(PARSE_FAILURE_ERR)),
        }
    }
}
