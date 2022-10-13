use super::Command;
use anyhow::{Error as AnyHowError, Result as AnyHowResult};
use std::{convert::TryFrom, iter::Iterator};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum OperatorCommand {
    Yank,
    Delete,
    Change,
    Indent,
    UnIndent,
    Uppercase,
    Lowercase,
    ToggleCase,
    Shell,
    Format,
}

pub const PARSE_FAILURE_ERR: &'static str = "Unknown Command";
impl TryFrom<&String> for OperatorCommand {
    type Error = AnyHowError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "y" => Ok(Self::Yank),
            "d" => Ok(Self::Delete),
            "c" => Ok(Self::Change),
            ">" => Ok(Self::Indent),
            "<" => Ok(Self::UnIndent),
            "gU" => Ok(Self::Uppercase),
            "gu" => Ok(Self::Lowercase),
            "~" => Ok(Self::ToggleCase),
            "!" => Ok(Self::Shell),
            "=" => Ok(Self::Format),
            _ => Err(Self::Error::msg(PARSE_FAILURE_ERR)),
        }
    }
}

impl Command for OperatorCommand {
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
