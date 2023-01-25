mod app;
mod buffer;
pub mod token;
mod ui;
mod window;
mod parser;

use std::io;
use std::io::Write;
use std::thread;
use std::time;

use termion;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use std::time::Duration;
use actix::prelude::*;
use crate::{
    parser::{Parser,UserInput},
    app::{App, Mode},
    buffer::Buffer,
    token::Token,
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
        //let ui = Ui::new().unwrap().start();
        let parser = Parser {mode: Mode::Normal}.start();
        let _ = io::stdout().into_raw_mode().unwrap();
        let mut stdin = termion::async_stdin().keys();
        loop {
            let input = stdin.next();
            if let Some(Ok(event)) = input {
                if let Ok(Ok(token)) = parser.send(UserInput {event}).await {
                    log::info!("sending {:?} to app", token);
                    let _ = app.send(token).await;
                }
               () 
            }
            thread::sleep(time::Duration::from_millis(50));
         }
    };
        let arbiter = Arbiter::new();
        arbiter.spawn(execution);
        let _ = system.run();

    /*
    let (app_tx, app_rx) = flume::unbounded::<Token>();
    let (ui_tx, ui_rx) = flume::unbounded::<Token>();
    let ui_handler = async_std::task::spawn(Ui::receive_tokens(app_rx.clone(), ui_tx.clone()));
    let app_handler = async_std::task::spawn(App::receive_tokens(
        cli.file_name,
        ui_rx.clone(),
        app_tx.clone(),
    ));
    let _ = ui_handler.await;
    let _ = app_handler.await;
    */
    Ok(())
}
