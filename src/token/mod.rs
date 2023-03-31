pub mod append_token;
pub mod command_token;
pub mod display_token;
pub mod insert_token;
pub mod normal_token;
pub mod operator_token;
pub mod motion_token;
use anyhow::{Error as AnyHowError, Result as AnyHowResult};
use crossterm::event::KeyEvent as Key;
use std::convert::TryFrom;

use crate::Mode;
pub use append_token::*;
pub use motion_token::*;
pub use command_token::*;
pub use display_token::*;
pub use insert_token::*;
pub use normal_token::*;
pub use operator_token::*;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Token {
    Append(AppendToken),
    Motion(MotionToken),
    Command(CommandToken),
    Insert(InsertToken),
    Normal(NormalToken),
    Operator(OperatorToken),
    Display(DisplayToken),
}

#[derive(Clone, Debug)]
pub struct GetState {}

pub fn get_tokens_from_chars(mode: &Mode, input: &Vec<char>) -> Vec<Token> {
    let mut skip_to = 0;
    let mut tokens = vec![];
    for idx in 0..input.len() {
        let unmatched = &input[skip_to..=idx];
        let token_result = match mode {
            Mode::Normal => {
                if let Ok(normal) = NormalToken::try_from(unmatched) {
                    Some(Token::Normal(normal))
                } else if let Ok(operator) = OperatorToken::try_from(unmatched) {
                    Some(Token::Operator(operator))
                } else if let Ok(motion) = MotionToken::try_from(unmatched) {
                    Some(Token::Motion(motion))
                } else {
                    None
                }
            }
            Mode::Command => {
                if let Ok(token) = CommandToken::try_from(unmatched) {
                    Some(Token::Command(token))
                } else {
                    None
                }
            }
            Mode::Insert => {
                if let Ok(token) = InsertToken::try_from(unmatched) {
                    Some(Token::Insert(token))
                } else {
                    None
                }
            }
            Mode::Append => {
                if let Ok(token) = AppendToken::try_from(unmatched) {
                    Some(Token::Append(token))
                } else {
                    None
                }
            }
            _ => None,
        };
        if let Some(token) = token_result {
            tokens.push(token);
            skip_to = idx + 1;
        }
    }
    tokens
}

pub fn get_token_from_chars(mode: &Mode, input: &Vec<char>) -> AnyHowResult<Token> {
    match mode {
        Mode::Command => Ok(Token::Command(CommandToken::try_from(input)?)),
        _ => Err(AnyHowError::msg("No Tokens Found".to_string())),
    }
}

pub fn get_token_from_key(mode: &Mode, event: &Key) -> AnyHowResult<Token> {
    match mode {
        Mode::Normal => {
            if let Ok(normal) = NormalToken::try_from(event) {
                Ok(Token::Normal(normal))
            } else if let Ok(operator) = OperatorToken::try_from(event) {
                Ok(Token::Operator(operator))
            } else if let Ok(motion) = MotionToken::try_from(event) {
                Ok(Token::Motion(motion))
            } else {
                Err(AnyHowError::msg("No Tokens Found".to_string()))
            }
        }
        Mode::Command => Ok(Token::Command(CommandToken::try_from(event)?)),
        Mode::Insert => Ok(Token::Insert(InsertToken::try_from(event)?)),
        Mode::Append => Ok(Token::Append(AppendToken::try_from(event)?)),
        _ => Err(AnyHowError::msg("No Tokens Found".to_string())),
    }
}
