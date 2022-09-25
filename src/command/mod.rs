pub mod normal_command;
use anyhow::{Result as AnyHowResult, Error as AnyHowError};

pub trait Command {
 fn tokenize(command: String) -> Vec<String> where Self: Sized;
 fn parse(tokens: Vec<String>) -> AnyHowResult<Vec<Self>> where Self: Sized;
}
