use crate::app::Mode;
use anyhow::Error as AnyHowError;
use crossterm::event::{KeyCode, KeyEvent as Key};
use std::{convert::TryFrom, iter::Iterator};
use uuid::Uuid;

#[derive(Clone, PartialEq, Eq, Debug)]
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
    YankLines(usize, usize),
    DeleteLines(usize, usize),
}

pub const PARSE_FAILURE_ERR: &'static str = "Unknown Token";

impl TryFrom<&[char]> for CommandToken {
    type Error = AnyHowError;
    fn try_from(value: &[char]) -> Result<Self, Self::Error> {
        let value_str = value.iter().collect::<String>();
        Ok(Self::Append(value_str.clone()))
    }
}

impl TryFrom<&Vec<char>> for CommandToken {
    type Error = AnyHowError;
    fn try_from(value: &Vec<char>) -> Result<Self, Self::Error> {
        let command_token = match &value[..] {
            [':', 'q', ..] => Ok(Self::Quit),
            [':', 'w', ..] => Ok(Self::Write),
            ['\n', ..] => Ok(Self::Enter),
            [':', 'v', 's', rest @ ..] => {
                Ok(Self::VerticalSplit(Some(rest.iter().collect::<String>())))
            }
            [':', 's', 'p', rest @ ..] => Ok(Self::Split(Some(rest.iter().collect::<String>()))),
            [':', rest @ ..]
                if rest
                    .iter()
                    .collect::<String>()
                    .trim()
                    .parse::<usize>()
                    .is_ok() =>
            {
                Ok(Self::GoToLine(
                    rest.iter()
                        .collect::<String>()
                        .trim()
                        .parse::<usize>()
                        .unwrap_or_default(),
                ))
            }
            _ => Err(Self::Error::msg(PARSE_FAILURE_ERR)),
        };
        let value_str = value.iter().collect::<String>();
        if command_token.is_err()
            && value_str.starts_with(":")
            && value_str.contains(',')
            && (value_str.ends_with("d") || value_str.ends_with("y"))
        {
            let last_char = value.iter().nth(value.len() - 1).unwrap();
            let command_vec = value_str
                .trim_start_matches(":")
                .trim_end_matches("d")
                .trim_end_matches("y")
                .split(',')
                .collect::<Vec<&str>>();
            return match &command_vec[..] {
                [start_index, end_index]
                    if *last_char == 'y'
                        && start_index.parse::<usize>().is_ok()
                        && end_index.parse::<usize>().is_ok() =>
                {
                    return Ok(Self::YankLines(
                        start_index.parse::<usize>().unwrap(),
                        end_index.parse::<usize>().unwrap(),
                    ))
                }
                [start_index, end_index]
                    if *last_char == 'd'
                        && start_index.parse::<usize>().is_ok()
                        && end_index.parse::<usize>().is_ok() =>
                {
                    return Ok(Self::DeleteLines(
                        start_index.parse::<usize>().unwrap(),
                        end_index.parse::<usize>().unwrap(),
                    ))
                }
                _ => Err(Self::Error::msg(PARSE_FAILURE_ERR)),
            };
        } else if command_token.is_err() {
            return Ok(Self::Append(value_str.clone()));
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
