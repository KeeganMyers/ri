pub mod append_token;
pub mod command_token;
pub mod display_token;
pub mod insert_token;
pub mod normal_token;
pub mod operator_token;
pub mod range_token;
use anyhow::{Error as AnyHowError, Result as AnyHowResult};
use std::convert::TryFrom;
use termion::event::Key;

use crate::Mode;
pub use append_token::*;
pub use command_token::*;
pub use display_token::*;
pub use insert_token::*;
pub use normal_token::*;
pub use operator_token::*;
pub use range_token::*;
use actix::prelude::*;

#[derive(Message)]
#[rtype(result = "()")]
#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Append(AppendToken),
    Command(CommandToken),
    Insert(InsertToken),
    Normal(NormalToken),
    Operator(OperatorToken),
    Range(RangeToken),
    Display(DisplayToken),
}

pub fn get_token_from_str(mode: &Mode, input: &String) -> AnyHowResult<Token> {
    match mode {
        Mode::Normal => {
            if let Ok(normal) = NormalToken::try_from(input) {
                Ok(Token::Normal(normal))
            } else if let Ok(operator) = OperatorToken::try_from(input) {
                Ok(Token::Operator(operator))
            } else if let Ok(range) = RangeToken::try_from(input) {
                Ok(Token::Range(range))
            } else {
                Err(AnyHowError::msg("No Tokens Found".to_string()))
            }
        }
        Mode::Command => Ok(Token::Command(CommandToken::try_from(input)?)),
        Mode::Insert => Ok(Token::Insert(InsertToken::try_from(input)?)),
        Mode::Append => Ok(Token::Append(AppendToken::try_from(input)?)),
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
            } else if let Ok(range) = RangeToken::try_from(event) {
                Ok(Token::Range(range))
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
