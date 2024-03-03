use anyhow::{Error as AnyhowError,Result as AnyhowResult};
use lsp_types::{ClientCapabilities, InitializeParams, ServerCapabilities};
use lsp_server::{Connection, Message, Request, RequestId, Response,IoThreads};
use tokio::{
    io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
    process::{Child, Command,ChildStdout,ChildStdin},
    sync::{
        mpsc::{channel, UnboundedReceiver, UnboundedSender},
        Notify, OnceCell,
    },
};

pub type IoTuple = (ChildStdin, ChildStdout);
pub async fn embed_rls() -> AnyhowResult<()> {
   let server_capabilities = serde_json::to_value(&ServerCapabilities::default()).unwrap();
    let mut process = Command::new("rust-analyzer")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()?;
    let init = serde_json::json!({
    "jsonrpc": "2.0",
    "id": "1",
    "method": "initialize",
    "params": {
        "capabilities": {},
    }
    }).to_string();
    let mut writer = BufWriter::new(process.stdin.take().expect("Failed to open stdin"));
    let mut reader = BufReader::new(process.stdout.take().expect("Failed to open stdout"));
    let mut stderr = BufReader::new(process.stderr.take().expect("Failed to open stderr"));
    
    write_message(&mut writer, &init).await?;
    log::debug!("read_buff {:?}", read_message(&mut reader).await?);
    /*
    let (server_rx, server_tx, initialize_notify) =
        Transport::start(reader, writer, stderr, id, name.clone());
    Err(AnyhowError::msg(
        "failed to start child process".to_string(),
    ))
    */
    Ok(())
}

pub async fn write_message(writer: &mut BufWriter<ChildStdin>,message: &str) -> AnyhowResult<()> {
   writer 
        .write_all(format!("Content-Length: {}\r\n\r\n", message.len()).as_bytes())
        .await?;
    writer.write_all(message.as_bytes()).await?;
    writer.flush().await?;
    Ok(())
}

pub async fn read_message(reader: &mut BufReader<ChildStdout>) -> AnyhowResult<String> {
        let mut buffer: String = "".to_string();
        let mut content_length = None;
        loop {
            buffer.truncate(0);
            if reader.read_line(&mut buffer).await? == 0 {
                return Err(AnyhowError::msg("stream closed"));
            };

            if buffer == "\r\n" {
                break;
            }

            let header = buffer.trim();

            let parts = header.split_once(": ");

            match parts {
                Some(("Content-Length", value)) => {
                    content_length = Some(value.parse()?);
                }
                Some((_, _)) => {}
                None => {
                }
            }
        }

        let content_length = content_length.unwrap_or_default();
        let mut content = vec![0; content_length];
        reader.read_exact(&mut content).await?;
        let msg = std::str::from_utf8(&content)?;
        Ok(msg.to_owned())
}
