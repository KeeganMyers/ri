mod app;
mod buffer;
pub mod token;
mod ui;
mod util;
mod window;

use crate::{
    app::{App, Mode},
    ui::Ui,
    buffer::Buffer,
    token::{get_token_from_key, get_token_from_str, Token},
    window::Window,
};

use anyhow::Result as AnyhowResult;
use argh::FromArgs;
use log::LevelFilter;
use log4rs::{
    append::{
        console::{ConsoleAppender, Target},
        file::FileAppender,
    },
    config::{Appender, Config as LogConfig, Root},
    encode::pattern::PatternEncoder,
    filter::threshold::ThresholdFilter,
};
use flume::unbounded;
use std::collections::HashMap;
use std::sync::Arc;
use std::{error::Error, io, time::Duration};
use termion::event::Key;
use termion::{input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::text::Spans;
use tui::{backend::TermionBackend, Terminal};
use util::event::{Config, Event, Events};
use uuid::Uuid;
#[macro_use]
extern crate serde_derive;
extern crate log;

#[derive(Debug, FromArgs)]
#[argh(description = "")]
struct Cli {
    ///file name to open in the first tab
    #[argh(positional)]
    file_name: Option<String>,
}

fn setup_logger() -> AnyhowResult<()> {
    let level = log::LevelFilter::Info;
    let file_path = "run_log";
    let stderr = ConsoleAppender::builder().target(Target::Stderr).build();

    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
        .build(file_path)
        .unwrap();

    let config = LogConfig::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(level)))
                .build("stderr", Box::new(stderr)),
        )
        .build(
            Root::builder()
                .appender("logfile")
                .appender("stderr")
                .build(LevelFilter::Trace),
        )
        .unwrap();

    let _handle = log4rs::init_config(config)?;
    Ok(())
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli: Cli = argh::from_env();
    let _ = setup_logger();
    let (tx, rx) = flume::unbounded::<Token>();
    let ui_handler = async_std::task::spawn({
        Ui::receive_tokens(rx.clone(), tx.clone())
    });
    let app_handler = async_std::task::spawn({
        App::receive_tokens(cli.file_name,rx.clone(), tx.clone())
    });
    let _ = ui_handler.await;
    let _ = app_handler.await;
    Ok(())
}
