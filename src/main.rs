mod app;
mod buffer;
mod parser;
pub mod reflow;
pub mod token;
mod ui;
mod window;

use crate::{
    app::{App, Mode},
    buffer::Buffer,
    parser::{Parser, UserInput},
    token::{
        display_token::{DisplayToken},
        Token,
    },
    ui::Ui,
    window::Window,
};
use crossterm;
use crossterm::event::{poll, read, Event};
use std::time::Duration;

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

fn main() -> Result<(), Box<dyn Error>> {
    let cli: Cli = argh::from_env();
    let _ = setup_logger();
    let mut app = App::new(cli.file_name)?;
    app.handle_token(Token::Display(DisplayToken::DrawViewPort));
    loop {
        if let Ok(true) = poll(Duration::from_millis(250)) {
            if let Ok(Event::Key(event)) = read() {
                if let Ok(token) = Parser::handle_event(UserInput { event }, &app.mode) {
                    app.handle_token(token); 
                    if app.should_quit {
                        break;
                    }
                }
                ()
            }
        }
    }
    Ok(())
}
