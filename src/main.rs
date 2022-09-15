mod app;
mod tab;
mod window;
mod buffer;
mod command_parser;
mod ui;
mod util;

use argh::FromArgs;
use termion::{event::Key,input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use crate::{app::App};
use util::event::{Config, Event, Events};
use std::{error::Error, io, time::Duration};
use tui::{backend::TermionBackend, Terminal};

/// Termion demo
#[derive(Debug, FromArgs)]
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
    let events = Events::with_config(config.clone());
    let mut app = App::new(true, cli.file_name)?;
    //env_logger::init();
    loop {
        if !app.should_quit {
            terminal.draw(|f| {
                ui::draw(f, &mut app);
                f.set_cursor(app.x_pos,app.y_pos + app.y_offset)
            })?;
        }
        match events.next()? {
            Event::Input(key) => match key {
                Key::Up => {
                    app.on_up();
                }
                Key::Backspace if app.mode == app::Mode::Insert || app.mode == app::Mode::Append  => {
                    app.remove_char();
                }
                Key::Backspace if app.mode == app::Mode::Normal => {
                    app.on_left();
                }
                Key::Down => {
                    app.on_down();
                }
                Key::Left => {
                    app.on_left();
                }
                Key::Right => {
                    app.on_right();
                }
                Key::Esc => {
                    if app.mode == app::Mode::Insert || app.mode == app::Mode::Append || app.mode == app::Mode::Select {
                        app.start_select_pos = None;
                        app.set_normal_mode();
                    }
                    if app.mode == app::Mode::Command {
                        app.set_normal_mode();
                    }
                }
                Key::Char(c) if c == ':' => {
                    app.command_text = Some("".to_string());
                    app.set_command_mode();
                    app.on_key(c, &config);
                },
                Key::Char(c) => {
                    app.on_key(c, &config);
                    if app.mode == app::Mode::Normal {
                        app.last_char = Some(c);
                    } else {
                        app.last_char = None;
                    }
                }
                _ => {}
            },
            Event::Tick => {
                app.on_tick();
            }
        }
        if app.should_quit {
            break;
        }
    }
    Ok(())
}
