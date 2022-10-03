mod app;
mod window;
mod buffer;
mod ui;
mod util;
pub mod command;

pub use command::{Command,normal_command::NormalCommand};
use std::sync::Arc;
use argh::FromArgs;
use termion::{event::Key,input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use crate::{app::App,buffer::Buffer,window::Window};
use util::event::{Config, Event, Events};
use std::{error::Error, io, time::Duration};
use tui::{backend::TermionBackend, Terminal};
extern crate strum;
#[macro_use]
extern crate strum_macros;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;

use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::{ThemeSet, Style};
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

#[derive(Debug, FromArgs)]
#[argh(description = "")]
struct Cli {
    /// time in ms between two ticks.
    #[argh(option, default = "250")]
    tick_rate: u64,
    /// whether unicode symbols are used to improve the overall look of the app
    #[argh(option, default = "true")]
    enhanced_graphics: bool,
    ///file name to open in the first tab
    #[argh(positional)]
    file_name: Option<String>
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli: Cli = argh::from_env();
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let config = Config {
        tick_rate: Duration::from_millis(250),
        ..Config::default()
    };
    // Set max_log_level to Trace
    tui_logger::init_logger(log::LevelFilter::Trace).unwrap();

    // Set default level for unknown targets to Trace
    tui_logger::set_default_level(log::LevelFilter::Trace);
    let events = Events::with_config(config.clone());
    let mut app = Arc::new(App::new(cli.file_name)?);
    //env_logger::init();
    loop {
        if !app.should_quit {
            terminal.draw(|f| {
                ui::draw(f, Arc::get_mut(&mut app).unwrap());
            })?;
        }

        Arc::get_mut(&mut app).unwrap().on_event(events.next()?,&config);
        if app.should_quit {
            break;
        }
    }
    Ok(())
}
