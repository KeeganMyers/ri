use super::Command;
use anyhow::{Error as AnyHowError, Result as AnyHowResult};
use std::{convert::TryFrom, iter::Iterator};

#[derive(Debug, PartialEq)]
pub enum RangeCommand {
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
}

pub const PARSE_FAILURE_ERR: &'static str = "Unknown Command";
impl TryFrom<&String> for RangeCommand {
    type Error = AnyHowError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        let cmd = match value.as_str() {
            "^" => Ok(Self::FirstNonBlank),
            "$" => Ok(Self::Last),
            "h" => Ok(Self::Left),
            "l" => Ok(Self::Right),
            "j" => Ok(Self::Up),
            "k" => Ok(Self::Down),
            "g_" => Ok(Self::LastNonBlank),
            "gg" => Ok(Self::LastLine),
            "G" => Ok(Self::FirstLine),
            "w" => Ok(Self::StartWord),
            "e" => Ok(Self::EndWord),
            "b" => Ok(Self::BackWord),
            "iw" => Ok(Self::InnerWord),
            _ => Err(Self::Error::msg(PARSE_FAILURE_ERR)),
        };
        if cmd.is_err() {
            return match value.chars().collect::<Vec<char>>().as_slice() {
                ['f', rest] => Ok(Self::FindNext(rest.to_string())),
                ['F', rest] => Ok(Self::FindLast(rest.to_string())),
                ['t', rest] => Ok(Self::TillNext(rest.to_string())),
                ['T', rest] => Ok(Self::TillLast(rest.to_string())),
                _ => Err(Self::Error::msg(PARSE_FAILURE_ERR)),
            };
        }
        cmd
    }
}

impl Command for RangeCommand {
    fn parse(mut tokens: Vec<String>) -> AnyHowResult<Vec<Self>> {
        let mut p_token = tokens.pop();
        let mut commands = Vec::new();

        while let Some(ref p_token_char) = p_token {
            if let Ok(command) = Self::try_from(p_token_char) {
                commands.push(command);
                p_token = tokens.pop();
            } else if let Some(added_token_char) = tokens.pop() {
                p_token = p_token.map(|s| format!("{}{}", s, added_token_char));
            } else {
                //return Err(AnyHowError::msg(PARSE_FAILURE_ERR));
                return Err(AnyHowError::msg(format!("{:?}", p_token_char)));
            }
        }
        Ok(commands)
    }
}
