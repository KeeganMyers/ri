pub mod normal_command;
pub mod operator_command;
pub mod range_command;
use anyhow::Result as AnyHowResult;

pub trait Command {
    fn tokenize(command: String) -> Vec<String> {
        command
            .split("")
            .map(|s| s.to_string())
            .filter(|s| s.len() >= 1)
            .collect()
    }

    fn parse(tokens: Vec<String>) -> AnyHowResult<Vec<Self>>
    where
        Self: Sized;
}
