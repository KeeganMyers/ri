mod app;
mod buffer;
pub mod token;
mod ui;
mod util;
mod window;

use crate::{
    app::{App, Mode},
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
use postage::{broadcast, prelude::Stream};
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli: Cli = argh::from_env();
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut highlight_cache: Arc<HashMap<Uuid, Vec<Spans>>> = Arc::new(HashMap::new());
    let mut line_num_cache: Arc<HashMap<Uuid, Spans>> = Arc::new(HashMap::new());
    let _ = setup_logger();
    let mut terminal = Terminal::new(backend)?;
    let config = Config {
        tick_rate: Duration::from_millis(250),
        ..Config::default()
    };
    let events = Events::with_config(config.clone());
    let mut app = Arc::new(App::new(cli.file_name)?);
    let mut token_str = String::new();
    let (mut tx, rx) = broadcast::channel::<Token>(50);
    tokio::task::spawn(print_messages(rx, tx));
    loop {
        let event = events.next()?;
        if let Ok(token) = get_token_from_key(&app, &event) {
            token_str.truncate(0);
        } else if let Event::Input(Key::Char(c)) = event {
            token_str.push_str(&c.to_string());
            if let Ok(token) = get_token_from_str(&app, &token_str) {
                token_str.truncate(0);
            }
        }
        if !app.should_quit {
            terminal.draw(|f| {
                ui::draw(
                    f,
                    Arc::get_mut(&mut app).unwrap(),
                    Arc::get_mut(&mut highlight_cache).unwrap(),
                    Arc::get_mut(&mut line_num_cache).unwrap(),
                );
            })?;
        }

        Arc::get_mut(&mut app).unwrap().on_event(event, &config);
        if app.should_quit {
            break;
        }
    }
    Ok(())
}

async fn print_messages(
    mut rx: impl Stream<Item = Token> + Unpin,
    mut tx: postage::broadcast::Sender<Token>,
) {
    while let Some(message) = rx.recv().await {
        println!("got a message: {:?}", message);
    }
}
