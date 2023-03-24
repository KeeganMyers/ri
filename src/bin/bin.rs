use crossterm;
use crossterm::event::{poll, read, Event};
use ri::{
    app::App,
    parser::{Parser, UserInput},
    token::{display_token::DisplayToken, Token},
};
use ri::{setup_logger, Cli};
use std::time::Duration;

use std::error::Error;
extern crate log;

fn main() -> Result<(), Box<dyn Error>> {
    let cli: Cli = argh::from_env();
    let _ = setup_logger();
    let mut app = App::new(cli.file_name)?;
    app.handle_tokens(vec![Token::Display(DisplayToken::DrawViewPort)]);
    let mut parser = Parser::new();
    loop {
        if let Ok(true) = poll(Duration::from_millis(250)) {
            if let Ok(Event::Key(event)) = read() {
                let tokens = parser.handle_event(UserInput { event }, &app.mode);
                app.handle_tokens(tokens);
                if app.should_quit {
                    break;
                }
                ()
            }
        }
    }
    Ok(())
}
