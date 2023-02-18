mod app;
mod buffer;
pub mod token;
mod ui;
mod window;
mod parser;

use crossterm::{execute, terminal::{ScrollUp, SetSize, size}};
use crossterm;
use crossterm::event::{poll, read, Event};
use std::time::Duration;
use actix::prelude::*;
use crate::{
    parser::{Parser,UserInput},
    app::{App, Mode},
    buffer::Buffer,
    token::{CommandToken,Token},
    ui::Ui,
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
use std::error::Error;
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
    let system = System::new();
    let execution = async {
        let app = App::new(cli.file_name).unwrap().start();
        let parser = Parser {mode: Mode::Normal}.start();
        let _ = app.send(Token::Command(CommandToken::NoOp)).await;
        loop {
             if let Ok(_) = poll(Duration::from_millis(500)) {
                let input = read();
                if let Ok(Event::Key(event)) = input {
                    if let Ok(Ok(token)) = parser.send(UserInput {event}).await {
                        let _ = app.send(token).await;
                    }
                   () 
                }
             }
         }
    };
        let arbiter = Arbiter::new();
        arbiter.spawn(execution);
        let _ = system.run();
    Ok(())
}

