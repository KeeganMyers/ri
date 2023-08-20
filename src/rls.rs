use std::process::{ChildStdin, ChildStdout};
use anyhow::{Error as AnyhowError,Result as AnyhowResult};

pub type IoTuple = (ChildStdin, ChildStdout);
pub fn embed_rls() -> AnyhowResult<IoTuple> {
    let child_process = std::process::Command::new("rust-analyzer")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()?;
    if let (Some(stdin), Some(stdout)) = (child_process.stdin, child_process.stdout) {
        return Ok((stdin, stdout));
    }
    Err(AnyhowError::msg(
        "failed to start child process".to_string(),
    ))
}
