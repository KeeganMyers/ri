mod app;
mod buffer;
pub mod command;
mod ui;
mod util;
mod window;

use crate::{app::App, buffer::Buffer, window::Window};
use argh::FromArgs;
pub use command::{
    normal_command::NormalCommand, operator_command::OperatorCommand, range_command::RangeCommand,
    Command,
};
use anyhow::Result as AnyhowResult;
use std::sync::Arc;
use std::{error::Error, io, time::Duration};
use termion::{input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{backend::TermionBackend, Terminal};
use util::event::{Config, Events};
use log::{debug, error, info, trace, warn, LevelFilter, SetLoggerError};
use log4rs::{
    append::{
        console::{ConsoleAppender, Target},
        file::FileAppender,
    },
    config::{Appender, Config as LogConfig, Root},
    encode::pattern::PatternEncoder,
    filter::threshold::ThresholdFilter,
};
#[macro_use]
extern crate serde_derive;
#[macro_use]
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

        // Build a stderr logger.
        let stderr = ConsoleAppender::builder().target(Target::Stderr).build();

        // Logging to log file.
        let logfile = FileAppender::builder()
            // Pattern: https://docs.rs/log4rs/*/log4rs/encode/pattern/index.html
            .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
            .build(file_path)
            .unwrap();

        // Log Trace level output to file where trace is the default level
        // and the programmatically specified level to stderr.
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

        // Use this to change log levels at runtime.
        // This means you can change the default log level to trace
        // if you are trying to debug an issue and need more logs on then turn it off
        // once you are done.
        let _handle = log4rs::init_config(config)?;
        Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli: Cli = argh::from_env();
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    setup_logger();
    let mut terminal = Terminal::new(backend)?;
    let config = Config {
        tick_rate: Duration::from_millis(250),
        ..Config::default()
    };

    let events = Events::with_config(config.clone());
    let mut app = Arc::new(App::new(cli.file_name)?);
    loop {
        if !app.should_quit {
            terminal.draw(|f| {
                ui::draw(f, Arc::get_mut(&mut app).unwrap());
            })?;
        }

        Arc::get_mut(&mut app)
            .unwrap()
            .on_event(events.next()?, &config);
        if app.should_quit {
            break;
        }
    }
    Ok(())
}
